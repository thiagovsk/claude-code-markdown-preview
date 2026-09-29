use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("md-preview-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Sandbox(dir)
    }

    fn write(&self, rel: &str, body: &str) -> PathBuf {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, body).unwrap();
        path
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_md-preview"))
            .arg("--no-open")
            .args(args)
            .env("TMPDIR", &self.0)
            .output()
            .unwrap()
    }

    fn render(&self, target: &Path) -> (PathBuf, String, String) {
        let out = self.run(&[target.to_str().unwrap()]);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(out.status.success(), "{stderr}");
        let page = PathBuf::from(String::from_utf8(out.stdout).unwrap().trim());
        assert!(
            page.starts_with(self.0.join("md-preview")),
            "{}",
            page.display()
        );
        assert_eq!(page.extension().and_then(|e| e.to_str()), Some("html"));
        (page.clone(), fs::read_to_string(page).unwrap(), stderr)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn renders_a_single_file_offline() {
    let sb = Sandbox::new("single");
    let file = sb.write(
        "docs/Plan & Notes.md",
        "# Hello World\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n- [ ] todo\n\n<script>alert(1)</script>\n",
    );
    let (page, html, stderr) = sb.render(&file);

    assert!(page
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("Plan & Notes-"));
    assert!(html.contains("<title>Plan &amp; Notes.md</title>"));
    assert!(html.contains("id=\\\"hello-world\\\""));
    assert!(html.contains("\\u003ctable>"));
    assert!(html.contains("type=\\\"checkbox\\\""));
    assert!(html.contains("\"total\":1,\"truncated\":false"));
    assert!(html.contains(".markdown-body{"), "github css is inlined");
    assert!(!html.contains("cdn.jsdelivr.net"));
    assert!(!html.contains("integrity="));
    assert_eq!(
        html.matches("</script>").count(),
        2,
        "raw script in markdown must not close the data block"
    );
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn renders_a_folder_sorted_and_skips_noise() {
    let sb = Sandbox::new("folder");
    let root = sb.0.join("docs");
    sb.write("docs/b.md", "b");
    sb.write("docs/README.md", "readme");
    sb.write("docs/a/x.MARKDOWN", "x");
    sb.write("docs/a/notes.txt", "not markdown");
    sb.write("docs/node_modules/z.md", "skip");
    sb.write("docs/.git/y.md", "skip");
    sb.write("docs/vendor/v.md", "skip");
    sb.write("docs/tmp/t.md", "skip");
    let (page, html, stderr) = sb.render(&root);

    assert!(page
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("docs-"));
    let order: Vec<usize> = ["[\"README.md\",", "[\"a/x.MARKDOWN\",", "[\"b.md\","]
        .iter()
        .map(|needle| {
            html.find(needle)
                .unwrap_or_else(|| panic!("missing {needle}"))
        })
        .collect();
    assert!(
        order[0] < order[1] && order[1] < order[2],
        "files must be sorted bytewise"
    );
    for skipped in ["z.md", "y.md", "v.md", "t.md", "notes.txt"] {
        assert!(!html.contains(skipped), "{skipped} should be skipped");
    }
    assert!(html.contains(&format!(
        "\"root\":\"{}\"",
        root.canonicalize().unwrap().display()
    )));
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn caps_a_folder_at_500_files_and_warns() {
    let sb = Sandbox::new("cap");
    let root = sb.0.join("many");
    for i in 0..501 {
        sb.write(&format!("many/f{i:03}.md"), "x");
    }
    let (_, html, stderr) = sb.render(&root);
    assert!(
        stderr.contains("501 Markdown files found, showing the first 500"),
        "{stderr}"
    );
    assert!(html.contains("\"total\":501,\"truncated\":true"));
    assert!(html.contains("[\"f499.md\","));
    assert!(!html.contains("[\"f500.md\","));
}

#[test]
fn same_input_gives_same_page_path() {
    let sb = Sandbox::new("stable");
    let file = sb.write("a.md", "one");
    let (first, _, _) = sb.render(&file);
    sb.write("a.md", "two");
    let (second, html, _) = sb.render(&file);
    assert_eq!(first, second);
    assert!(html.contains("\\u003cp>two\\u003c/p>"));
}

#[test]
fn fails_on_a_folder_without_markdown() {
    let sb = Sandbox::new("empty");
    sb.write("empty/readme.txt", "x");
    let out = sb.run(&[sb.0.join("empty").to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no Markdown files found"));
}

#[test]
fn fails_on_a_missing_path() {
    let sb = Sandbox::new("missing");
    let out = sb.run(&[sb.0.join("nope.md").to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a file or folder"));
}

#[test]
fn prints_usage_without_exactly_one_target() {
    let sb = Sandbox::new("usage");
    assert_eq!(sb.run(&[]).status.code(), Some(2));
    assert_eq!(sb.run(&["a.md", "b.md"]).status.code(), Some(2));
}
