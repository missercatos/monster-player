package demo;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.reflect.TypeToken;

import java.lang.reflect.Type;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * 最小 Java 消费方：Java → Go 网关 → 塞壬唱片 API。
 *
 * <p>验证四件事：
 * <ol>
 *   <li>Java 能按契约解析网关响应（Gson 反序列化到 record）；</li>
 *   <li>Java 能以专辑 cid 继续请求专辑详情，形成完整调用链；</li>
 *   <li>重复请求命中网关缓存（X-Cache: HIT）；</li>
 *   <li>能从 /stats 读取计数，对应目标架构中的数据统计角色。</li>
 * </ol>
 *
 * <p>运行：java -cp lib/gson.jar:out demo.App
 * <p>环境变量：GATEWAY_URL（默认 http://127.0.0.1:8080）
 */
public final class App {

    // ---- 契约结构（与 docs/monster-siren-api.md 对应） ----
    record Album(String cid, String name, String coverUrl, List<String> artistes) {}
    record AlbumSong(String cid, String name, List<String> artistes) {}
    record AlbumDetail(String cid, String name, String intro, String belong,
                       String coverUrl, String coverDeUrl, List<AlbumSong> songs) {}
    record ApiResponse<T>(int code, String msg, T data) {}

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    private static final HttpClient HTTP = HttpClient.newBuilder()
            .connectTimeout(Duration.ofSeconds(10))
            .build();
    private static final String BASE =
            System.getenv().getOrDefault("GATEWAY_URL", "http://127.0.0.1:8080");

    public static void main(String[] args) throws Exception {
        Path outDir = Path.of(args.length > 0 ? args[0] : "demo/out");
        Files.createDirectories(outDir);

        System.out.println("== Java 客户端 ==");
        System.out.println("gateway: " + BASE);

        HttpResponse<String> health = get("/healthz");
        System.out.println("[1] /healthz -> HTTP " + health.statusCode() + " " + health.body().trim());

        HttpResponse<String> albumsResp = get("/api/albums");
        String firstCache = albumsResp.headers().firstValue("X-Cache").orElse("-");
        Type albumListType = new TypeToken<ApiResponse<List<Album>>>() {}.getType();
        ApiResponse<List<Album>> albums = GSON.fromJson(albumsResp.body(), albumListType);
        requireOk(albums.code(), albums.msg(), "/api/albums");
        List<Album> all = albums.data();
        System.out.printf("[2] /api/albums -> HTTP %d, cache=%s, albums=%d%n",
                albumsResp.statusCode(), firstCache, all.size());

        int sampleSize = Math.min(3, all.size());
        for (Album album : all.subList(0, sampleSize)) {
            System.out.printf("    - %s (%s) / %s%n",
                    album.name(), album.cid(), String.join(", ", album.artistes()));
        }

        Album target = all.get(0);
        HttpResponse<String> detailResp = get("/api/album/" + target.cid() + "/detail");
        Type detailType = new TypeToken<ApiResponse<AlbumDetail>>() {}.getType();
        ApiResponse<AlbumDetail> detail = GSON.fromJson(detailResp.body(), detailType);
        requireOk(detail.code(), detail.msg(), "album detail");
        System.out.printf("[3] /api/album/%s/detail -> %s, songs=%d%n",
                target.cid(), detail.data().name(), detail.data().songs().size());

        HttpResponse<String> againResp = get("/api/albums");
        String secondCache = againResp.headers().firstValue("X-Cache").orElse("-");
        System.out.printf("[4] /api/albums 二次调用 -> cache=%s (期望 HIT)%n", secondCache);
        if (!"HIT".equals(secondCache)) {
            throw new IllegalStateException("网关缓存未生效: second X-Cache=" + secondCache);
        }

        HttpResponse<String> statsResp = get("/stats");
        JsonObject stats = GSON.fromJson(statsResp.body(), JsonObject.class);
        System.out.println("[5] /stats -> " + stats);

        Map<String, Object> report = new LinkedHashMap<>();
        report.put("gateway", BASE);
        report.put("generated_at", Instant.now().toString());
        report.put("albums_total", all.size());
        report.put("album_sample", all.subList(0, sampleSize).stream().map(Album::name).toList());
        report.put("detail_album", detail.data().name());
        report.put("detail_songs", detail.data().songs().size());
        report.put("first_call_cache", firstCache);
        report.put("second_call_cache", secondCache);
        report.put("stats", stats);
        Path reportPath = outDir.resolve("java-report.json");
        Files.writeString(reportPath, GSON.toJson(report));

        System.out.println("[OK] Java 链路通过，报告写入 " + reportPath);
    }

    private static HttpResponse<String> get(String path) throws Exception {
        HttpRequest request = HttpRequest.newBuilder(URI.create(BASE + path))
                .timeout(Duration.ofSeconds(30))
                .header("Accept", "application/json")
                .GET()
                .build();
        return HTTP.send(request, HttpResponse.BodyHandlers.ofString());
    }

    private static void requireOk(int code, String msg, String what) {
        if (code != 0) {
            throw new IllegalStateException(what + " 返回业务错误: code=" + code + ", msg=" + msg);
        }
    }
}
