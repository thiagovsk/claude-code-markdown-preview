#!/usr/bin/env bash
set -euo pipefail

MARKED_URL="https://cdn.jsdelivr.net/npm/marked@18.0.14/lib/marked.umd.js"
MARKED_SRI="sha384-2vpGtuKqJvFlwJqYnf/wUMuzUfhUnYBt9oay0e2yaFcq0Dh6/aEbQ8YAOeKGzlYo"
CSS_URL="https://cdn.jsdelivr.net/npm/github-markdown-css@5.9.0/github-markdown.min.css"
CSS_SRI="sha384-mYBW/AGDT6JhlmN0DlBZPH4430+HhjMvn1xOSmsXnjSDn+zfyMwb3xCym6H5ICgn"

if [ $# -ne 1 ]; then
  echo "usage: render.sh <file.md>" >&2
  exit 2
fi

if [ ! -f "$1" ]; then
  echo "render.sh: file not found: $1" >&2
  exit 1
fi

src_dir="$(cd "$(dirname "$1")" && pwd)"
src_name="$(basename "$1")"
src_path="$src_dir/$src_name"

out_dir="${TMPDIR:-/tmp}/md-preview"
mkdir -p "$out_dir"
if command -v shasum >/dev/null 2>&1; then
  hash_cmd="shasum"
elif command -v sha1sum >/dev/null 2>&1; then
  hash_cmd="sha1sum"
else
  hash_cmd="cksum"
fi
out_hash="$(printf '%s' "$src_path" | $hash_cmd | tr -cd '[:alnum:]' | cut -c1-12)"
out_file="$out_dir/${src_name%.*}-$out_hash.html"

content_b64="$(base64 < "$src_path" | tr -d '\n')"
title="$(printf '%s' "$src_name" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g')"
base_href="file://$(printf '%s/' "$src_dir" | sed 's/ /%20/g')"

cat > "$out_file" <<HTML
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>$title</title>
<base href="$base_href">
<link rel="stylesheet" href="$CSS_URL" integrity="$CSS_SRI" crossorigin="anonymous">
<style>
  body { margin: 0; background: #ffffff; }
  @media (prefers-color-scheme: dark) { body { background: #0d1117; } }
  .markdown-body { box-sizing: border-box; max-width: 980px; margin: 0 auto; padding: 32px 16px; }
</style>
</head>
<body>
<article id="content" class="markdown-body"></article>
<script src="$MARKED_URL" integrity="$MARKED_SRI" crossorigin="anonymous"></script>
<script>
  const bytes = Uint8Array.from(atob("$content_b64"), (c) => c.charCodeAt(0));
  const markdown = new TextDecoder().decode(bytes);
  document.getElementById("content").innerHTML = marked.parse(markdown, { gfm: true });
</script>
</body>
</html>
HTML

if command -v open >/dev/null 2>&1; then
  open "$out_file"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$out_file" >/dev/null 2>&1 &
else
  echo "render.sh: no 'open' or 'xdg-open' found, open the file yourself" >&2
fi

echo "$out_file"
