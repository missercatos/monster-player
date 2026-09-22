package main

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"
	"time"
)

func TestProxyCachesUpstreamResponses(t *testing.T) {
	var upstreamHits atomic.Int64
	upstream := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		upstreamHits.Add(1)
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"code":0,"msg":"","data":[{"cid":"1","name":"demo"}]}`))
	}))
	defer upstream.Close()

	g := newGateway(upstream.URL, time.Minute)
	srv := httptest.NewServer(g.routes())
	defer srv.Close()

	for i, want := range []string{"MISS", "HIT", "HIT"} {
		resp, err := http.Get(srv.URL + "/api/albums")
		if err != nil {
			t.Fatalf("call %d: %v", i+1, err)
		}
		_ = resp.Body.Close()
		if got := resp.Header.Get("X-Cache"); got != want {
			t.Errorf("call %d: X-Cache = %q, want %q", i+1, got, want)
		}
	}
	if got := upstreamHits.Load(); got != 1 {
		t.Errorf("upstream hits = %d, want 1 (cache should absorb repeats)", got)
	}
}

func TestProxyPassesThroughQueryStrings(t *testing.T) {
	var gotQuery atomic.Value
	upstream := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotQuery.Store(r.URL.RawQuery)
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"code":0,"msg":"","data":{"albums":{"list":[],"end":true},"news":{"list":[],"end":true}}}`))
	}))
	defer upstream.Close()

	g := newGateway(upstream.URL, time.Minute)
	srv := httptest.NewServer(g.routes())
	defer srv.Close()

	resp, err := http.Get(srv.URL + "/api/search?keyword=test")
	if err != nil {
		t.Fatal(err)
	}
	_ = resp.Body.Close()

	if q, _ := gotQuery.Load().(string); !strings.Contains(q, "keyword=test") {
		t.Errorf("upstream query = %q, want it to contain keyword=test", q)
	}
}

func TestHealthAndStats(t *testing.T) {
	g := newGateway("http://example.invalid", time.Minute)
	srv := httptest.NewServer(g.routes())
	defer srv.Close()

	resp, err := http.Get(srv.URL + "/healthz")
	if err != nil {
		t.Fatal(err)
	}
	_ = resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Errorf("/healthz status = %d, want 200", resp.StatusCode)
	}

	resp, err = http.Get(srv.URL + "/stats")
	if err != nil {
		t.Fatal(err)
	}
	_ = resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Errorf("/stats status = %d, want 200", resp.StatusCode)
	}
}
