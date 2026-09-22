#!/usr/bin/env bash
# monster-player RS · 最小多语言闭环 demo 一键运行
#
# 调用链:
#   Java / Dart / Rust(CLI) / Python  →  Go 网关  →  塞壬唱片 API
#
# 用法:
#   bash demo/run.sh            # 本机运行 Go 网关
#   bash demo/run.sh --docker   # 用 Docker 运行 Go 网关
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO="$ROOT/demo"
OUT="$DEMO/out"
PORT="${PORT:-8080}"
GATEWAY_URL="http://127.0.0.1:${PORT}"
GSON_VERSION="2.11.0"
GSON_JAR="$DEMO/java-client/lib/gson-${GSON_VERSION}.jar"
DOCKER_MODE=0
[[ "${1:-}" == "--docker" ]] && DOCKER_MODE=1

mkdir -p "$OUT"

step() { printf '\n\033[1;36m== %s ==\033[0m\n' "$*"; }
ok()   { printf '\033[1;32m[OK]\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31m[FAIL]\033[0m %s\n' "$*"; exit 1; }

cleanup() {
  if [[ -n "${GATEWAY_PID:-}" ]]; then kill "$GATEWAY_PID" 2>/dev/null || true; fi
  if [[ "$DOCKER_MODE" -eq 1 ]]; then docker stop monster-player-demo-gateway >/dev/null 2>&1 || true; fi
}
trap cleanup EXIT

step "0/6 检查工具链"
for tool in cargo go java javac dart python3 curl jq; do
  command -v "$tool" >/dev/null 2>&1 || fail "缺少工具: $tool"
done
if [[ "$DOCKER_MODE" -eq 1 ]]; then
  command -v docker >/dev/null 2>&1 || fail "缺少工具: docker"
fi
ok "工具链就绪"

step "1/6 构建 Rust 参考工具（API + 调用机制）"
cargo build -q -p siren-ref --manifest-path "$ROOT/Cargo.toml"
ok "target/debug/siren"

step "2/6 构建 Go 网关"
(cd "$DEMO/go-gateway" && go test ./... >/dev/null && go build -o "$OUT/gateway" .)
ok "$OUT/gateway（含单元测试）"

step "3/6 准备 Java 客户端"
mkdir -p "$(dirname "$GSON_JAR")" "$OUT/java-classes"
if [[ ! -f "$GSON_JAR" ]]; then
  echo "    下载 Gson ${GSON_VERSION} ..."
  curl -fsSL -o "$GSON_JAR" "https://repo1.maven.org/maven2/com/google/code/gson/gson/${GSON_VERSION}/gson-${GSON_VERSION}.jar"
fi
javac -cp "$GSON_JAR" -d "$OUT/java-classes" "$DEMO/java-client/src/demo/App.java"
ok "demo.App 编译完成"

step "4/6 启动网关（端口 ${PORT}）"
if [[ "$DOCKER_MODE" -eq 1 ]]; then
  docker build -q -t monster-player-demo-gateway "$DEMO/go-gateway" >/dev/null
  docker rm -f monster-player-demo-gateway >/dev/null 2>&1 || true
  docker run -d --rm --name monster-player-demo-gateway -p "${PORT}:8080" monster-player-demo-gateway >/dev/null
  ok "Docker 容器已启动"
else
  "$OUT/gateway" -port "$PORT" >"$OUT/gateway.log" 2>&1 &
  GATEWAY_PID=$!
  ok "本机进程已启动（pid ${GATEWAY_PID}）"
fi

for _ in $(seq 1 50); do
  curl -fsS "$GATEWAY_URL/healthz" >/dev/null 2>&1 && break
  sleep 0.2
done
curl -fsS "$GATEWAY_URL/healthz" | jq -c . || fail "网关未就绪"
ok "网关健康检查通过"

step "5/6 多语言调用链"
echo "--- Java 客户端（冷缓存首访，应看到 MISS -> HIT） ---"
java -cp "$GSON_JAR:$OUT/java-classes" demo.App "$OUT"

echo "--- Dart 客户端（与 Flutter 同语言） ---"
dart "$DEMO/dart-client/bin/main.dart"

echo "--- Rust 参考 CLI：直连上游 vs 经 Go 网关 ---"
"$ROOT/target/debug/siren" albums -o "$OUT/albums-direct.json"
"$ROOT/target/debug/siren" --base-url "$GATEWAY_URL" albums -o "$OUT/albums-via-go.json"
if diff -q <(jq -S . "$OUT/albums-direct.json") <(jq -S . "$OUT/albums-via-go.json") >/dev/null; then
  ok "Rust 直连与经网关响应一致"
else
  fail "Rust 直连与经网关响应不一致"
fi

echo "--- Python 一致性校验 + 契约检查 ---"
GATEWAY_URL="$GATEWAY_URL" python3 "$DEMO/python-verify/verify.py"

step "6/6 完成"
echo "产物目录: $OUT"
ls -1 "$OUT" | sed 's/^/  - /'
ok "最小多语言闭环验证通过"
