# claude-code-markdown-preview

A Claude Code plugin that opens a Markdown file as a rendered page in your browser. Point it at a folder instead and you get every Markdown file in it on one page, with a searchable file tree to move between them (up to 500 files per folder).

<img width="1651" height="750" alt="Screenshot 2026-09-28 at 10 44 37" src="https://github.com/user-attachments/assets/5491729e-f91c-410e-8369-984d3b5d5a81" />


Easy to use:

```
/md:preview docs/plan.md
/md:preview docs/
```

## The problem

Claude Code writes a lot of Markdown: plans, reports, READMEs, ADRs, PR drafts. When Claude Code ran inside Cursor or another VS Code based editor, you opened the file and clicked "Open Preview". The editor rendered it.

In a terminal-first setup there is no preview. Claude Code has no built-in way to show a Markdown file as a rendered page, so you read raw Markdown in the terminal: tables are rows of pipes, headings are `#` signs, and long documents are hard to scan. Editor plugins such as `markdown-preview.nvim` fix this only inside the editor, and only while the editor is open.

This plugin gives Claude Code the same "open preview" action. You ask for it with a slash command, or you ask Claude in plain words ("open the plan in the browser"), and the file opens as a rendered page.

## What it does

- Renders GitHub-flavored Markdown: tables, task lists, fenced code, autolinks, strikethrough, footnotes, alerts and heading anchors.
- Uses GitHub's Markdown style, with light and dark mode that follow your system setting.
- Resolves relative images and links from the file's own folder.
- Opens a folder as one page with a file tree on the side:
  - every `.md` and `.markdown` file in the folder and its subfolders, with `.git`, `node_modules`, `vendor` and `tmp` skipped
  - a filter box to find a file by name
  - links between Markdown files (`[plan](../plan.md)`) open the other file in the same page
  - the open file is in the URL, so the browser back button and a page reload keep your place
  - it starts on the folder's `README.md` when there is one
  - at most 500 files per page; above that it shows the first 500 and says so
- Writes the page to `$TMPDIR/md-preview/` and opens it with `open` (macOS) or `xdg-open` (Linux). Your Markdown files are never changed.

The page is a snapshot. After you edit a file, run the command again to see the change.

## How it works

`skills/preview/scripts/render.sh` runs a small Rust program, `md-preview`, and builds it on the first run. The program walks the folder, renders each Markdown file to HTML with [comrak](https://github.com/kivikakk/comrak) (the same GFM extensions GitHub uses) and writes one self-contained HTML page. The page carries the rendered HTML of every file, [github-markdown-css](https://github.com/sindresorhus/github-markdown-css) and a small script for the file tree and navigation. Nothing is loaded from the network.

So:

- The page works offline. Your Markdown content stays on your machine; it is not uploaded anywhere.
- The first run compiles the program, which takes about a minute. After that it starts instantly and rebuilds itself only when the source changes.
- Raw HTML inside the Markdown is rendered as HTML, like on GitHub. Only preview files you trust.

## Requirements

Besides the plugin itself you need a Rust toolchain, used once to build the program.

| What | Used for | Where it comes from |
|---|---|---|
| Claude Code v2.1.157 or later | loading the plugin from `~/.claude/skills/` | you already have it |
| `cargo` (Rust 1.85 or later) | building `md-preview` on the first run | [rustup.rs](https://rustup.rs) |
| `bash` | the launcher script | preinstalled on macOS and Linux |
| `open` (macOS) or `xdg-open` (Linux) | opening the page in your default browser | preinstalled on macOS; on Linux part of `xdg-utils` |

To check your machine, run:

```bash
claude --version
cargo --version
command -v open || command -v xdg-open
```

Every command should print a version or a path. If `cargo` is missing, install Rust with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`. If `xdg-open` is missing on Linux, install `xdg-utils` with your package manager (for example `sudo apt install xdg-utils`).

You can also build ahead of time, for example right after cloning:

```bash
cargo build --release --manifest-path ~/.claude/skills/md/Cargo.toml
```

## Install

### Load it in every session

Claude Code loads any folder in `~/.claude/skills/` that has a `.claude-plugin/plugin.json` as a plugin, in every session:

```bash
git clone https://github.com/thiagovsk/claude-code-markdown-preview ~/.claude/skills/md
```

If you already keep a clone somewhere else, link it instead:

```bash
ln -s /path/to/claude-code-markdown-preview ~/.claude/skills/md
```

New sessions load the plugin on their own.

### Sessions that are already open

A session that was open before you installed the plugin does not see it yet. Run this in that session:

```
/reload-plugins
```

It reloads plugins without restarting the session. After that `/md:preview` is in the `/` menu, and Claude in that session knows about the skill, so asking "open this Markdown file in the browser" works too. You don't need to explain the plugin to Claude.

### Try it for one session

```bash
claude --plugin-dir /path/to/claude-code-markdown-preview
```

## Usage

```
/md:preview README.md
/md:preview docs/adr/0001-some-decision.md
/md:preview docs/
/md:preview .
```

You can also ask in plain words, for example "preview the plan you just wrote" or "open the docs folder in the browser". Claude runs the same script. With no argument, it uses the Markdown file or folder from the conversation, or asks which one you mean.

The first run asks for permission to run the script unless your settings already allow it. The skill pre-approves only its own script for the turn it runs in.

## Layout

```
.claude-plugin/plugin.json        plugin manifest (name: md)
skills/preview/SKILL.md           the /md:preview skill
skills/preview/scripts/render.sh  builds md-preview when needed and runs it
Cargo.toml, src/main.rs           md-preview: walks the folder, renders Markdown, writes and opens the page
assets/                           page template, stylesheet, client script and vendored github-markdown-css
tests/cli.rs                      end-to-end tests of the binary
```

To work on the program:

```bash
cargo test
cargo build --release
```

## Troubleshooting

- `/md:preview` is not listed: run `claude plugin validate <plugin folder>`, then `/reload-plugins`, then check the Errors tab in `/plugin`.
- `render.sh` says cargo was not found: install Rust from [rustup.rs](https://rustup.rs) and run the command again. The launcher also looks in `~/.cargo/bin`.
- The build fails: run `cargo build --release --manifest-path ~/.claude/skills/md/Cargo.toml` to see the full compiler output.
- Images are missing: image paths are resolved from the Markdown file's folder. Absolute URLs work too.

## License

MIT
