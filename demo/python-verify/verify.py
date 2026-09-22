#!/usr/bin/env python3
"""跨语言一致性校验：Go 网关 vs 塞壬上游，并做最小 OpenAPI 契约检查。

- 一致性：同一接口分别直连上游与经网关请求，规范化 JSON 后必须完全一致；
- 契约：demo/contracts/openapi.yaml 中声明的 GET 路径必须在网关上真实存在并返回 JSON；
- 参数化路径不硬编码 cid，而是从 /api/albums 动态发现，保证 demo 长期可用。

退出码 0 = 全部通过；1 = 存在失败项。
"""

from __future__ import annotations

import json
import os
import sys
import urllib.parse
import urllib.request
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - 环境没有 PyYAML 时跳过契约检查
    yaml = None

UPSTREAM = os.environ.get("SIREN_BASE_URL", "https://monster-siren.hypergryph.com")
GATEWAY = os.environ.get("GATEWAY_URL", "http://127.0.0.1:8080")
SPEC_PATH = Path(__file__).resolve().parents[1] / "contracts" / "openapi.yaml"


def get(base: str, path: str) -> tuple[int, dict]:
    request = urllib.request.Request(
        base + path,
        headers={"Accept": "application/json", "User-Agent": "monster-player-demo/0.1"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.status, json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as err:
        payload = err.read().decode("utf-8", "replace")
        try:
            return err.code, json.loads(payload)
        except json.JSONDecodeError:
            return err.code, {"raw": payload}


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"))


def main() -> int:
    failures: list[str] = []

    print("== 跨语言一致性校验（Go 网关 vs 塞壬上游） ==")
    _, albums_upstream = get(UPSTREAM, "/api/albums")
    album_cid = albums_upstream["data"][0]["cid"]
    _, album_upstream = get(UPSTREAM, f"/api/album/{album_cid}/detail")
    song_cid = album_upstream["data"]["songs"][0]["cid"]

    paths = [
        "/api/albums",
        "/api/songs",
        "/api/news",
        "/api/search?keyword=" + urllib.parse.quote("月行"),
        f"/api/album/{album_cid}/detail",
        f"/api/song/{song_cid}",
    ]
    for path in paths:
        _, upstream = get(UPSTREAM, path)
        _, via_gateway = get(GATEWAY, path)
        if canonical(upstream) == canonical(via_gateway):
            print(f"[OK]   {path}")
        else:
            print(f"[FAIL] {path} — 网关与上游响应不一致")
            failures.append(path)

    if yaml is None:
        print("[SKIP] 未安装 PyYAML，跳过契约检查")
    else:
        print("== OpenAPI 契约检查 ==")
        spec = yaml.safe_load(SPEC_PATH.read_text(encoding="utf-8"))
        for path, item in spec.get("paths", {}).items():
            if "{" in path:
                continue  # 参数化路径已由一致性校验覆盖
            operation = item.get("get")
            if operation is None:
                continue

            # 契约里声明了必填查询参数时，使用契约自带的 example 构造请求
            query = []
            skippable = False
            for param in operation.get("parameters", []):
                if param.get("in") != "query" or not param.get("required"):
                    continue
                example = param.get("example") or param.get("schema", {}).get("example")
                if example is None:
                    skippable = True
                    break
                query.append((param["name"], example))
            if skippable:
                print(f"[SKIP] {path} — 契约未提供查询参数示例")
                continue

            url = path + ("?" + urllib.parse.urlencode(query) if query else "")
            status, body = get(GATEWAY, url)
            ok = status == 200 and isinstance(body, dict)
            if ok and path.startswith("/api/"):
                ok = {"code", "msg", "data"} <= set(body)
            if ok:
                print(f"[OK]   {path}")
            else:
                print(f"[FAIL] {path} — 契约不符（status={status}）")
                failures.append(path)

    print()
    if failures:
        print(f"结果：{len(failures)} 项失败 -> {failures}")
        return 1
    print("结果：全部通过")
    return 0


if __name__ == "__main__":
    sys.exit(main())
