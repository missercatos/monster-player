// Package main 实现一个最小化的塞壬唱片 API 缓存网关（BFF/代理）。
//
// 演示要点：
//   - 与上游保持一致的路径与响应包装（{code,msg,data}），
//     因此任何客户端（包括 tools/siren-ref）把 base URL 指向本网关即可，无需改动代码；
//   - 内存 TTL 缓存，对应目标架构中的 Redis 角色；
//   - /healthz 与 /stats，对应可观测性与计数角色；
//   - 结构化 JSON 日志（log/slog）。
//
// 生产环境将使用 Gin/Echo + Redis + 限流 + 熔断 + gRPC 内部通信，本文件只验证最小闭环。
package main

import (
	"encoding/json"
	"flag"
	"io"
	"log/slog"
	"net/http"
	"os"
	"strconv"
	"sync"
	"time"
)

// entry 是一条缓存记录。
type entry struct {
	body        []byte
	contentType string
	expiresAt   time.Time
}

// gateway 是网关的全部状态。
type gateway struct {
	baseURL string
	client  *http.Client
	ttl     time.Duration
	started time.Time

	mu     sync.Mutex
	cache  map[string]entry
	hits   int
	misses int
}

func newGateway(baseURL string, ttl time.Duration) *gateway {
	return &gateway{
		baseURL: baseURL,
		client:  &http.Client{Timeout: 15 * time.Second},
		ttl:     ttl,
		started: time.Now(),
		cache:   make(map[string]entry),
	}
}

// routes 注册全部路由并套上日志中间件。
func (g *gateway) routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", g.handleHealth)
	mux.HandleFunc("GET /stats", g.handleStats)
	// 与上游完全一致的代理路径
	mux.HandleFunc("GET /api/albums", g.handleProxy)
	mux.HandleFunc("GET /api/songs", g.handleProxy)
	mux.HandleFunc("GET /api/news", g.handleProxy)
	mux.HandleFunc("GET /api/search", g.handleProxy)
	mux.HandleFunc("GET /api/album/{cid}/detail", g.handleProxy)
	mux.HandleFunc("GET /api/song/{cid}", g.handleProxy)
	return g.withLogging(mux)
}

func (g *gateway) handleHealth(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{
		"status":   "ok",
		"upstream": g.baseURL,
		"time":     time.Now().UTC().Format(time.RFC3339),
	})
}

func (g *gateway) handleStats(w http.ResponseWriter, _ *http.Request) {
	g.mu.Lock()
	hits, misses, size := g.hits, g.misses, len(g.cache)
	g.mu.Unlock()
	writeJSON(w, http.StatusOK, map[string]any{
		"cache": map[string]any{
			"entries":     size,
			"hits":        hits,
			"misses":      misses,
			"ttl_seconds": int(g.ttl.Seconds()),
		},
		"uptime_seconds": int(time.Since(g.started).Seconds()),
	})
}

// handleProxy 是透明代理：命中缓存直接返回，否则回源并写入缓存。
func (g *gateway) handleProxy(w http.ResponseWriter, r *http.Request) {
	key := r.URL.RequestURI() // 含查询串，作为缓存键

	if e, ok := g.lookup(key); ok {
		w.Header().Set("X-Cache", "HIT")
		w.Header().Set("Content-Type", e.contentType)
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write(e.body)
		return
	}

	req, err := http.NewRequestWithContext(r.Context(), http.MethodGet, g.baseURL+key, nil)
	if err != nil {
		writeProxyError(w, http.StatusInternalServerError, "invalid upstream request")
		return
	}
	req.Header.Set("Accept", "application/json")

	resp, err := g.client.Do(req)
	if err != nil {
		slog.Error("upstream request failed", "path", key, "error", err.Error())
		writeProxyError(w, http.StatusBadGateway, "upstream request failed")
		return
	}
	defer func() { _ = resp.Body.Close() }()

	body, err := io.ReadAll(io.LimitReader(resp.Body, 16<<20))
	if err != nil {
		writeProxyError(w, http.StatusBadGateway, "reading upstream response failed")
		return
	}

	contentType := resp.Header.Get("Content-Type")
	if contentType == "" {
		contentType = "application/json"
	}

	if resp.StatusCode == http.StatusOK && json.Valid(body) {
		g.store(key, entry{body: body, contentType: contentType, expiresAt: time.Now().Add(g.ttl)})
		w.Header().Set("X-Cache", "MISS")
	} else {
		w.Header().Set("X-Cache", "BYPASS")
	}
	w.Header().Set("Content-Type", contentType)
	w.WriteHeader(resp.StatusCode)
	_, _ = w.Write(body)
}

func (g *gateway) lookup(key string) (entry, bool) {
	g.mu.Lock()
	defer g.mu.Unlock()
	e, ok := g.cache[key]
	if !ok {
		g.misses++
		return entry{}, false
	}
	if time.Now().After(e.expiresAt) {
		delete(g.cache, key)
		g.misses++
		return entry{}, false
	}
	g.hits++
	return e, true
}

func (g *gateway) store(key string, e entry) {
	g.mu.Lock()
	defer g.mu.Unlock()
	g.cache[key] = e
}

// withLogging 输出结构化 JSON 访问日志。
func (g *gateway) withLogging(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		rec := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
		next.ServeHTTP(rec, r)
		slog.LogAttrs(r.Context(), slog.LevelInfo, "http",
			slog.String("method", r.Method),
			slog.String("path", r.URL.Path),
			slog.String("query", r.URL.RawQuery),
			slog.Int("status", rec.status),
			slog.String("cache", rec.Header().Get("X-Cache")),
			slog.Duration("duration", time.Since(start)),
		)
	})
}

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (r *statusRecorder) WriteHeader(code int) {
	r.status = code
	r.ResponseWriter.WriteHeader(code)
}

func writeJSON(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}

func writeProxyError(w http.ResponseWriter, status int, msg string) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("X-Cache", "BYPASS")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]any{"code": -1, "msg": msg, "data": nil})
}

func envOr(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func envInt(key string, fallback int) int {
	if v := os.Getenv(key); v != "" {
		if n, err := strconv.Atoi(v); err == nil {
			return n
		}
	}
	return fallback
}

func main() {
	port := flag.String("port", envOr("PORT", "8080"), "listen port")
	baseURL := flag.String("upstream", envOr("SIREN_BASE_URL", "https://monster-siren.hypergryph.com"), "upstream base URL")
	ttlSeconds := flag.Int("ttl", envInt("SIREN_CACHE_TTL", 300), "cache TTL in seconds")
	flag.Parse()

	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))

	g := newGateway(*baseURL, time.Duration(*ttlSeconds)*time.Second)
	addr := ":" + *port
	slog.Info("gateway listening", "addr", addr, "upstream", *baseURL, "cache_ttl_seconds", *ttlSeconds)
	if err := http.ListenAndServe(addr, g.routes()); err != nil {
		slog.Error("server exited", "error", err.Error())
		os.Exit(1)
	}
}
