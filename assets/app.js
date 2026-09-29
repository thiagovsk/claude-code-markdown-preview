const pageUrl = location.href.split("#")[0];
const root = DATA.root;
const files = new Map(DATA.files);
const paths = [...files.keys()];
const baseEl = document.getElementById("base");
const content = document.getElementById("content");
const tree = document.getElementById("tree");
const filter = document.getElementById("filter");
let current = null;

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
  content.innerHTML = files.get(path);
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
document.getElementById("note").textContent = DATA.truncated
  ? `Showing ${paths.length} of ${DATA.total} files`
  : `${paths.length} file${paths.length === 1 ? "" : "s"}`;
filter.addEventListener("input", renderSidebar);
window.addEventListener("popstate", () => show(decodeURI(location.hash.slice(1)), { push: false }));
renderSidebar();
show(decodeURI(location.hash.slice(1)), { push: false });
