# claude-code-markdown-preview

A Claude Code plugin that opens a Markdown file, or a whole folder of them with a file tree, as a rendered page in your browser.

```
/md:preview docs/plan.md
/md:preview docs/
```

## The problem

Claude Code writes a lot of Markdown: plans, reports, READMEs, ADRs, PR drafts. When Claude Code ran inside Cursor or another VS Code based editor, you opened the file and clicked "Open Preview". The editor rendered it.

In a terminal-first setup there is no preview. Claude Code has no built-in way to show a Markdown file as a rendered page, so you read raw Markdown in the terminal: tables are rows of pipes, headings are `#` signs, and long documents are hard to scan. Editor plugins such as `markdown-preview.nvim` fix this only inside the editor, and only while the editor is open.

This plugin gives Claude Code the same "open preview" action. You ask for it with a slash command, or you ask Claude in plain words ("open the plan in the browser"), and the file opens as a rendered page.

## What it does

- Renders GitHub-flavored Markdown: tables, task lists, fenced code, autolinks.
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

`skills/preview/scripts/render.sh` builds one HTML file. The Markdown text of each file is embedded in the page as base64, and the browser renders it with [marked](https://github.com/markedjs/marked) and [github-markdown-css](https://github.com/sindresorhus/github-markdown-css). Both are loaded from the jsDelivr CDN, pinned to one version and checked with Subresource Integrity hashes.

So:

- Nothing to install: no Pandoc, no Node, no Python.
- The page needs internet access to load those two files. Your Markdown content stays on your machine; it is not uploaded anywhere.
- Raw HTML inside the Markdown is rendered as HTML, like on GitHub. Only preview files you trust.

## Requirements

There is nothing to install besides the plugin itself.

| What | Used for | Where it comes from |
|---|---|---|
| Claude Code v2.1.157 or later | loading the plugin from `~/.claude/skills/` | you already have it |
| `bash`, `base64`, `sed` | building the HTML page | preinstalled on macOS and Linux |
| `shasum`, `sha1sum` or `cksum` (any one) | naming the output file | preinstalled on macOS and Linux |
| `open` (macOS) or `xdg-open` (Linux) | opening the page in your default browser | preinstalled on macOS; on Linux part of `xdg-utils` |
| [marked](https://github.com/markedjs/marked) 18.0.14 | turning Markdown into HTML | loaded by the browser from jsDelivr, not installed |
| [github-markdown-css](https://github.com/sindresorhus/github-markdown-css) 5.9.0 | GitHub's Markdown style | loaded by the browser from jsDelivr, not installed |

To check your machine, run:

```bash
claude --version
command -v bash base64 sed
command -v shasum || command -v sha1sum || command -v cksum
command -v open || command -v xdg-open
```

Every command should print a version or a path. If `xdg-open` is missing on Linux, install `xdg-utils` with your package manager (for example `sudo apt install xdg-utils`).

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
skills/preview/scripts/render.sh  builds the HTML page and opens it
```

## Troubleshooting

- `/md:preview` is not listed: run `claude plugin validate <plugin folder>`, then `/reload-plugins`, then check the Errors tab in `/plugin`.
- The page is blank: the browser could not load the CDN files. Check your internet connection.
- Images are missing: image paths are resolved from the Markdown file's folder. Absolute URLs work too.

## License

MIT
