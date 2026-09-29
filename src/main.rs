use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

const MAX_FILES: usize = 500;
const SKIPPED_DIRS: &[&str] = &[".git", "node_modules", "vendor", "tmp"];

const PAGE: &str = include_str!("../assets/page.html");
const APP_JS: &str = include_str!("../assets/app.js");
const STYLE_CSS: &str = include_str!("../assets/style.css");
const GITHUB_CSS: &str = include_str!("../assets/github-markdown.min.css");

struct Source {
    path: PathBuf,
    root: PathBuf,
    name: String,
    files: Vec<String>,
    total: usize,
}

fn main() {
    let mut open = true;
    let mut target = None;
    for arg in env::args().skip(1) {
        if arg == "--no-open" {
            open = false;
        } else if target.is_none() {
            target = Some(arg);
        } else {
            usage();
        }
    }
    let Some(target) = target else { usage() };

    let result = collect(Path::new(&target)).and_then(|source| {
        let html = build_page(&source)?;
        let out = output_path(&source);
        fs::create_dir_all(out.parent().unwrap())
            .and_then(|_| fs::write(&out, html))
            .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
        Ok(out)
    });
    match result {
        Ok(out) => {
            if open {
                open_in_browser(&out);
            }
            println!("{}", out.display());
        }
        Err(message) => {
            eprintln!("md-preview: {message}");
            process::exit(1);
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: md-preview [--no-open] <file.md | folder>");
    process::exit(2);
}

fn collect(target: &Path) -> Result<Source, String> {
    let not_found = || format!("not a file or folder: {}", target.display());
    let meta = fs::metadata(target).map_err(|_| not_found())?;
    let path = fs::canonicalize(target).map_err(|_| not_found())?;
    if meta.is_dir() {
        let mut files = Vec::new();
        walk(&path, &path, &mut files);
        if files.is_empty() {
            return Err(format!("no Markdown files found in {}", path.display()));
        }
        files.sort();
        let total = files.len();
        if total > MAX_FILES {
            eprintln!("md-preview: {total} Markdown files found, showing the first {MAX_FILES}");
            files.truncate(MAX_FILES);
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        Ok(Source {
            root: path.clone(),
            path,
            name,
            files,
            total,
        })
    } else if meta.is_file() {
        let root = path.parent().ok_or_else(not_found)?.to_path_buf();
        let name = path
            .file_name()
            .ok_or_else(not_found)?
            .to_string_lossy()
            .into_owned();
        Ok(Source {
            path: path.clone(),
            root,
            files: vec![name.clone()],
            name,
            total: 1,
        })
    } else {
        Err(not_found())
    }
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("md-preview: skipping {}: {e}", dir.display());
            return;
        }
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let path = entry.path();
        if kind.is_dir() {
            if !SKIPPED_DIRS.contains(&name.as_ref()) {
                walk(root, &path, out);
            }
        } else if kind.is_file() && is_markdown(&name) {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel: Vec<_> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect();
            out.push(rel.join("/"));
        }
    }
}

fn is_markdown(name: &str) -> bool {
    let ext = name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase());
    matches!(ext.as_deref(), Some("md" | "markdown"))
}

fn render_markdown(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    let ext = &mut options.extension;
    ext.strikethrough = true;
    ext.table = true;
    ext.autolink = true;
    ext.tasklist = true;
    ext.footnotes = true;
    ext.alerts = true;
    ext.header_id_prefix = Some(String::new());
    // Raw HTML passes through, like GitHub. The README tells users to preview only files they trust.
    options.render.r#unsafe = true;
    comrak::markdown_to_html(markdown, &options)
}

fn build_page(source: &Source) -> Result<String, String> {
    let mut entries = Vec::with_capacity(source.files.len());
    for rel in &source.files {
        let bytes =
            fs::read(source.root.join(rel)).map_err(|e| format!("cannot read {rel}: {e}"))?;
        let html = render_markdown(&String::from_utf8_lossy(&bytes));
        entries.push(serde_json::json!([rel, html]));
    }
    let data = serde_json::json!({
        "root": source.root.to_string_lossy(),
        "total": source.total,
        "truncated": source.total > source.files.len(),
        "files": entries,
    });
    Ok(fill(
        PAGE,
        &[
            ("TITLE", &html_escape(&source.name)),
            ("DATA", &script_safe_json(&data)),
            ("GITHUB_CSS", GITHUB_CSS),
            ("STYLE", STYLE_CSS),
            ("APP", APP_JS),
        ],
    ))
}

// Escaping "<" keeps "</script>" inside rendered HTML from ending the data block.
fn script_safe_json(value: &serde_json::Value) -> String {
    value.to_string().replace('<', "\\u003c")
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// Single pass, so a value that happens to contain "{{KEY}}" is never expanded.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str("{{");
            rest = after;
            continue;
        };
        let key = &after[..end];
        match values.iter().find(|(k, _)| *k == key) {
            Some((_, value)) => out.push_str(value),
            None => {
                out.push_str("{{");
                out.push_str(key);
                out.push_str("}}");
            }
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

fn output_path(source: &Source) -> PathBuf {
    let stem = match source.name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => source.name.as_str(),
    };
    let hash = fnv1a(source.path.to_string_lossy().as_bytes());
    env::temp_dir()
        .join("md-preview")
        .join(format!("{stem}-{:012x}.html", hash & 0xffff_ffff_ffff))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn open_in_browser(path: &Path) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let spawned = Command::new(opener)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if spawned.is_err() {
        eprintln!("md-preview: '{opener}' not found, open the file yourself");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_extensions_match_github() {
        let html = render_markdown("# Hello World\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n- [x] done\n- [ ] todo\n\n~~gone~~ https://example.com\n");
        assert!(html.contains("<h1 id=\"hello-world\">"));
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\" checked=\"\""));
        assert!(html.contains("<del>gone</del>"));
        assert!(html.contains("<a href=\"https://example.com\">"));
    }

    #[test]
    fn raw_html_is_kept() {
        assert!(
            render_markdown("<details><summary>x</summary>y</details>\n").contains("<details>")
        );
    }

    #[test]
    fn json_never_closes_the_script_tag() {
        let value = serde_json::json!({ "html": "<script></script>" });
        let text = script_safe_json(&value);
        assert!(!text.contains('<'));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&text).unwrap(),
            value
        );
    }

    #[test]
    fn fill_replaces_once_and_keeps_unknown_keys() {
        let out = fill("a {{X}} b {{Y}} c {{Z", &[("X", "{{Y}}"), ("Y", "y")]);
        assert_eq!(out, "a {{Y}} b y c {{Z");
    }

    #[test]
    fn markdown_extensions_are_case_insensitive() {
        assert!(is_markdown("a.MD"));
        assert!(is_markdown("a.markdown"));
        assert!(!is_markdown("a.md.txt"));
        assert!(!is_markdown("md"));
    }

    #[test]
    fn output_path_uses_stem_and_stable_hash() {
        let source = Source {
            path: PathBuf::from("/x/notes.md"),
            root: PathBuf::from("/x"),
            name: "notes.md".into(),
            files: vec!["notes.md".into()],
            total: 1,
        };
        let a = output_path(&source);
        assert_eq!(a, output_path(&source));
        let file = a.file_name().unwrap().to_string_lossy();
        assert!(file.starts_with("notes-"), "{file}");
        assert!(file.ends_with(".html"), "{file}");
        assert_eq!(file.len(), "notes-".len() + 12 + ".html".len());
    }
}
