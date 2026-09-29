#!/usr/bin/env bash
set -euo pipefail

MARKED_URL="https://cdn.jsdelivr.net/npm/marked@18.0.14/lib/marked.umd.js"
MARKED_SRI="sha384-2vpGtuKqJvFlwJqYnf/wUMuzUfhUnYBt9oay0e2yaFcq0Dh6/aEbQ8YAOeKGzlYo"
CSS_URL="https://cdn.jsdelivr.net/npm/github-markdown-css@5.9.0/github-markdown.min.css"
CSS_SRI="sha384-mYBW/AGDT6JhlmN0DlBZPH4430+HhjMvn1xOSmsXnjSDn+zfyMwb3xCym6H5ICgn"
MAX_FILES=500

if [ $# -ne 1 ]; then
  echo "usage: render.sh <file.md | folder>" >&2
  exit 2
fi

b64() { base64 | tr -d '\n'; }

hash_of() {
  if command -v shasum >/dev/null 2>&1; then
    printf '%s' "$1" | shasum
  elif command -v sha1sum >/dev/null 2>&1; then
    printf '%s' "$1" | sha1sum
  else
    printf '%s' "$1" | cksum
  fi | tr -cd '[:alnum:]' | cut -c1-12
}

html_escape() { printf '%s' "$1" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'; }

list_markdown() {
  (cd "$1" && find . \( -name .git -o -name node_modules -o -name vendor -o -name tmp \) -prune -o \
    -type f \( -iname '*.md' -o -iname '*.markdown' \) -print) | sed 's|^\./||' | LC_ALL=C sort
}

if [ -d "$1" ]; then
  root="$(cd "$1" && pwd)"
  files="$(list_markdown "$root")"
  if [ -z "$files" ]; then
    echo "render.sh: no Markdown files found in $root" >&2
    exit 1
  fi
  total="$(printf '%s\n' "$files" | wc -l | tr -d ' ')"
  truncated=false
  if [ "$total" -gt "$MAX_FILES" ]; then
    echo "render.sh: $total Markdown files found, showing the first $MAX_FILES" >&2
    files="$(printf '%s\n' "$files" | sed -n "1,${MAX_FILES}p")"
    truncated=true
  fi
  name="$(basename "$root")"
elif [ -f "$1" ]; then
  root="$(cd "$(dirname "$1")" && pwd)"
  files="$(basename "$1")"
  total=1
  truncated=false
  name="$files"
else
  echo "render.sh: not a file or folder: $1" >&2
  exit 1
fi

out_dir="${TMPDIR:-/tmp}/md-preview"
mkdir -p "$out_dir"
out_file="$out_dir/${name%.*}-$(hash_of "$root/$name").html"
title="$(html_escape "$name")"

{
  cat <<HTML
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>$title</title>
<base id="base" href="">
<link rel="stylesheet" href="$CSS_URL" integrity="$CSS_SRI" crossorigin="anonymous">
<script src="$MARKED_URL" integrity="$MARKED_SRI" crossorigin="anonymous"></script>
HTML
  printf '<script>\nconst ROOT_B64 = "%s";\nconst TOTAL = %s;\nconst TRUNCATED = %s;\nconst FILES = [\n' \
    "$(printf '%s' "$root" | b64)" "$total" "$truncated"
  while IFS= read -r rel; do
    printf '["%s","%s"],\n' "$(printf '%s' "$rel" | b64)" "$(b64 < "$root/$rel")"
  done <<< "$files"
  printf '];\n</script>\n'
  cat <<'HTML'
<style>
  :root { --bg: #ffffff; --side-bg: #f6f8fa; --border: #d0d7de; --text: #1f2328; --muted: #59636e; --active: #ddf4ff; }
  @media (prefers-color-scheme: dark) {
    :root { --bg: #0d1117; --side-bg: #010409; --border: #30363d; --text: #e6edf3; --muted: #9198a1; --active: #1f3a5f; }
  }
  html, body { margin: 0; background: var(--bg); color: var(--text); }
  .layout { display: grid; grid-template-columns: 300px minmax(0, 1fr); min-height: 100vh; }
  .layout.single { grid-template-columns: minmax(0, 1fr); }
  .layout.single aside { display: none; }
  aside { background: var(--side-bg); border-right: 1px solid var(--border); padding: 12px; position: sticky; top: 0; height: 100vh; overflow: auto; box-sizing: border-box; font: 14px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
  aside input { width: 100%; box-sizing: border-box; padding: 6px 8px; margin-bottom: 8px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); }
  aside .note { color: var(--muted); font-size: 12px; margin: 0 0 8px; }
  aside ul { list-style: none; margin: 0; padding-left: 14px; }
  aside > nav > ul { padding-left: 0; }
  aside summary { cursor: pointer; color: var(--muted); padding: 2px 0; }
  aside a { display: block; padding: 2px 6px; border-radius: 6px; color: var(--text); text-decoration: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  aside a:hover { background: var(--border); }
  aside a.active { background: var(--active); font-weight: 600; }
  main .markdown-body { box-sizing: border-box; max-width: 980px; margin: 0 auto; padding: 32px 24px; }
  main .path { max-width: 980px; margin: 0 auto; padding: 16px 24px 0; color: var(--muted); font: 12px ui-monospace, SFMono-Regular, Menlo, monospace; box-sizing: border-box; }
  @media (max-width: 800px) {
    .layout { grid-template-columns: minmax(0, 1fr); }
    aside { position: static; height: auto; max-height: 40vh; border-right: 0; border-bottom: 1px solid var(--border); }
  }
</style>
</head>
<body>
<div class="layout" id="layout">
  <aside>
    <input id="filter" type="search" placeholder="Filter files" aria-label="Filter files">
    <p class="note" id="note"></p>
    <nav id="tree"></nav>
  </aside>
  <main>
    <div class="path" id="path"></div>
    <article class="markdown-body" id="content"></article>
  </main>
</div>
<script>
  const pageUrl = location.href.split("#")[0];
  const decode = (b64) => new TextDecoder().decode(Uint8Array.from(atob(b64), (c) => c.charCodeAt(0)));
  const root = decode(ROOT_B64);
  const files = new Map(FILES.map(([path, body]) => [decode(path), body]));
  const paths = [...files.keys()];
  const cache = new Map();
  const baseEl = document.getElementById("base");
  const content = document.getElementById("content");
  const tree = document.getElementById("tree");
  const filter = document.getElementById("filter");
  let current = null;

  const textOf = (path) => {
    if (!cache.has(path)) cache.set(path, decode(files.get(path)));
    return cache.get(path);
  };
  const dirOf = (path) => path.slice(0, path.lastIndexOf("/") + 1);
  const fileUrl = (abs) => "file://" + abs.split("/").map(encodeURIComponent).join("/");
  const resolvePath = (from, href) => {
    const out = [];
    for (const part of (dirOf(from) + href).split("/")) {
      if (part === "" || part === ".") continue;
      if (part === "..") out.pop();
      else out.push(part);
    }
    return out.join("/");
  };
  const defaultPath = () => paths.find((p) => /^readme\.(md|markdown)$/i.test(p)) || paths[0];

  function show(path, { push = true } = {}) {
    if (!files.has(path)) path = defaultPath();
    current = path;
    baseEl.href = fileUrl(root + "/" + dirOf(path));
    content.innerHTML = marked.parse(textOf(path), { gfm: true });
    document.getElementById("path").textContent = path;
    document.title = path.split("/").pop();
    if (push && location.hash.slice(1) !== encodeURI(path)) history.pushState(null, "", pageUrl + "#" + encodeURI(path));
    tree.querySelectorAll("a").forEach((a) => a.classList.toggle("active", a.dataset.path === path));
    window.scrollTo(0, 0);
  }

  function link(path, label) {
    const a = document.createElement("a");
    a.href = pageUrl + "#" + encodeURI(path);
    a.dataset.path = path;
    a.textContent = label;
    a.title = path;
    a.addEventListener("click", (event) => { event.preventDefault(); show(path); });
    return a;
  }

  function buildTree() {
    const rootNode = { dirs: new Map(), files: [] };
    for (const path of paths) {
      const parts = path.split("/");
      let node = rootNode;
      for (const dir of parts.slice(0, -1)) {
        if (!node.dirs.has(dir)) node.dirs.set(dir, { dirs: new Map(), files: [] });
        node = node.dirs.get(dir);
      }
      node.files.push(path);
    }
    const render = (node) => {
      const ul = document.createElement("ul");
      for (const [dir, child] of node.dirs) {
        const li = document.createElement("li");
        const details = document.createElement("details");
        details.open = true;
        const summary = document.createElement("summary");
        summary.textContent = dir + "/";
        details.append(summary, render(child));
        li.append(details);
        ul.append(li);
      }
      for (const path of node.files) {
        const li = document.createElement("li");
        li.append(link(path, path.split("/").pop()));
        ul.append(li);
      }
      return ul;
    };
    return render(rootNode);
  }

  function renderSidebar() {
    const query = filter.value.trim().toLowerCase();
    tree.replaceChildren();
    if (query) {
      const ul = document.createElement("ul");
      for (const path of paths.filter((p) => p.toLowerCase().includes(query))) {
        const li = document.createElement("li");
        li.append(link(path, path));
        ul.append(li);
      }
      tree.append(ul);
    } else {
      tree.append(buildTree());
    }
    tree.querySelectorAll("a").forEach((a) => a.classList.toggle("active", a.dataset.path === current));
  }

  content.addEventListener("click", (event) => {
    const a = event.target.closest("a");
    if (!a) return;
    const href = a.getAttribute("href") || "";
    if (href.startsWith("#")) {
      event.preventDefault();
      const target = document.getElementById(decodeURIComponent(href.slice(1)));
      if (target) target.scrollIntoView();
      return;
    }
    if (/^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("//")) return;
    const target = resolvePath(current, decodeURI(href.split(/[?#]/)[0]));
    if (files.has(target)) {
      event.preventDefault();
      show(target);
    }
  });

  if (paths.length === 1) document.getElementById("layout").classList.add("single");
  document.getElementById("note").textContent = TRUNCATED
    ? `Showing ${paths.length} of ${TOTAL} files`
    : `${paths.length} file${paths.length === 1 ? "" : "s"}`;
  filter.addEventListener("input", renderSidebar);
  window.addEventListener("popstate", () => show(decodeURI(location.hash.slice(1)), { push: false }));
  renderSidebar();
  show(decodeURI(location.hash.slice(1)), { push: false });
</script>
</body>
</html>
HTML
} > "$out_file"

if command -v open >/dev/null 2>&1; then
  open "$out_file"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$out_file" >/dev/null 2>&1 &
else
  echo "render.sh: no 'open' or 'xdg-open' found, open the file yourself" >&2
fi

echo "$out_file"
