#!/bin/bash
# release-2026-11-14: a minified copy of web/ for the release build (no comments, no whitespace), then
#   STELLA_WEB_DIR=$PWD/target/web-min cargo build --release
# Every other file is copied as it is. ESBUILD names the esbuild binary (default: the 3d3d-site copy).
set -eu
cd "$(dirname "$0")/.."
ESBUILD=${ESBUILD:-$HOME/3d3d-site/node_modules/.bin/esbuild}
out=target/web-min
rm -rf "$out"; mkdir -p "$out"; cp -a web/. "$out/"
n=0
while IFS= read -r -d '' f; do
  rel=${f#web/}
  case "$f" in
    *.js) "$ESBUILD" "$f" --minify --legal-comments=none --log-level=warning --outfile="$out/$rel" --allow-overwrite ;;
    *.css) "$ESBUILD" "$f" --minify --legal-comments=none --log-level=warning --outfile="$out/$rel" --allow-overwrite ;;
    *) continue ;;
  esac
  n=$((n + 1))
done < <(find web -type f \( -name '*.js' -o -name '*.css' \) -not -path '*/vendor/*' -print0)
before=$(du -sb web | cut -f1); after=$(du -sb "$out" | cut -f1)
echo "minified $n files: web/ $((before / 1000)) KB -> $out $((after / 1000)) KB"
