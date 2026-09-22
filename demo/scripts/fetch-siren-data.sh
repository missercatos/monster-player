#!/usr/bin/env bash
# 用 Rust 参考工具（tools/siren-ref）抓取塞壬唱片真实数据，生成 demo fixtures。
#
# 用法:
#   bash demo/scripts/fetch-siren-data.sh
#
# 产物目录: demo/fixtures/（已 gitignore）
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/demo/fixtures"
SIREN="$ROOT/target/debug/siren"

mkdir -p "$OUT"

echo "==> 构建 siren-ref"
cargo build -q -p siren-ref --manifest-path "$ROOT/Cargo.toml"

echo "==> 抓取全量专辑 / 歌曲 / 新闻"
"$SIREN" albums -o "$OUT/albums.json"
"$SIREN" songs  -o "$OUT/songs.json"
"$SIREN" news   -o "$OUT/news.json"

CID="$(jq -r '.[0].cid' "$OUT/albums.json")"
echo "==> 抓取示例专辑详情 (cid=$CID)"
"$SIREN" album "$CID" -o "$OUT/album-$CID.json"

SONG_CID="$(jq -r '.songs[0].cid' "$OUT/album-$CID.json")"
echo "==> 抓取示例歌曲详情 (cid=$SONG_CID)"
"$SIREN" song "$SONG_CID" -o "$OUT/song-$SONG_CID.json"

# 示例歌曲可能没有歌词，向前探测最多 5 首，仍未找到就换下一张专辑再试
echo "==> 抓取示例歌词"
LYRIC_FOUND=""
for cid in $(jq -r '[.songs[].cid][:5][]' "$OUT/album-$CID.json"); do
  if "$SIREN" lyrics "$cid" -o "$OUT/lyric-$cid.lrc" 2>/dev/null; then
    LYRIC_FOUND="$cid"
    break
  fi
done
if [[ -z "$LYRIC_FOUND" ]]; then
  NEXT_CID="$(jq -r '.[1].cid' "$OUT/albums.json")"
  echo "==> 该专辑暂无歌词，尝试下一张专辑 (cid=$NEXT_CID)"
  "$SIREN" album "$NEXT_CID" -o "$OUT/album-$NEXT_CID.json"
  for cid in $(jq -r '[.songs[].cid][:5][]' "$OUT/album-$NEXT_CID.json"); do
    if "$SIREN" lyrics "$cid" -o "$OUT/lyric-$cid.lrc" 2>/dev/null; then
      LYRIC_FOUND="$cid"
      break
    fi
  done
fi
if [[ -n "$LYRIC_FOUND" ]]; then
  echo "==> 歌词已保存 (song cid=$LYRIC_FOUND)"
else
  echo "==> 两次探测均无歌词，跳过"
fi

echo "==> 完成，fixtures 位于 $OUT"
ls -1 "$OUT"
