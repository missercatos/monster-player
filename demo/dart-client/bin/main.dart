// 最小 Dart 消费方：Dart → Go 网关 → 塞壬唱片 API。
//
// 目的：用与 Flutter 客户端相同的语言（Dart）验证网关调用链，
// 确认后续 Flutter 三端接入只需替换 UI 层，网络与数据层可直接复用。
//
// 运行：dart demo/dart-client/bin/main.dart
// 环境变量：GATEWAY_URL（默认 http://127.0.0.1:8080）
import 'dart:convert';
import 'dart:io';

Future<void> main() async {
  final base = Platform.environment['GATEWAY_URL'] ?? 'http://127.0.0.1:8080';
  stdout.writeln('== Dart 客户端 ==');
  stdout.writeln('gateway: $base');

  final client = HttpClient()..connectionTimeout = const Duration(seconds: 10);
  try {
    final (albums, albumsCache) = await _getJson(client, '$base/api/albums');
    final data = (albums['data'] as List).cast<Map<String, dynamic>>();
    stdout.writeln('[1] /api/albums -> cache=$albumsCache, albums=${data.length}');

    final target = data.first;
    stdout.writeln('    first: ${target['name']} (${target['cid']})');

    final (detail, _) = await _getJson(client, '$base/api/album/${target['cid']}/detail');
    final detailData = detail['data'] as Map<String, dynamic>;
    final songs = (detailData['songs'] as List).length;
    stdout.writeln(
        '[2] /api/album/${target['cid']}/detail -> ${detailData['name']}, songs=$songs');

    final (_, secondCache) = await _getJson(client, '$base/api/albums');
    stdout.writeln('[3] /api/albums 二次调用 -> cache=$secondCache (期望 HIT)');
    if (secondCache != 'HIT') {
      stderr.writeln('[FAIL] 网关缓存未生效');
      exitCode = 1;
      return;
    }

    stdout.writeln('[OK] Dart 链路通过');
  } finally {
    client.close(force: true);
  }
}

/// 返回 (解析后的 JSON, X-Cache 头)。
Future<(Map<String, dynamic>, String)> _getJson(HttpClient client, String url) async {
  final request = await client.getUrl(Uri.parse(url));
  request.headers.set(HttpHeaders.acceptHeader, 'application/json');
  final response = await request.close();
  final body = await response.transform(utf8.decoder).join();
  if (response.statusCode != 200) {
    throw StateError('GET $url -> HTTP ${response.statusCode}: $body');
  }
  final parsed = jsonDecode(body) as Map<String, dynamic>;
  return (parsed, response.headers.value('x-cache') ?? '-');
}
