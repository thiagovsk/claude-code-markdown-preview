# claude-code-markdown-preview

A Claude Code plugin that opens a Markdown file as a rendered page in your browser.

```
/md:preview docs/plan.md
```

## The problem

Claude Code writes a lot of Markdown: plans, reports, READMEs, ADRs, PR drafts. When Claude Code ran inside Cursor or another VS Code based editor, you opened the file and clicked "Open Preview". The editor rendered it.

In a terminal-first setup there is no preview. Claude Code has no built-in way to show a Markdown file as a rendered page, so you read raw Markdown in the terminal: tables are rows of pipes, headings are `#` signs, and long documents are hard to scan. Editor plugins such as `markdown-preview.nvim` fix this only inside the editor, and only while the editor is open.

This plugin gives Claude Code the same "open preview" action. You ask for it with a slash command, or you ask Claude in plain words ("open the plan in the browser"), and the file opens as a rendered page.

## What it does

- Renders GitHub-flavored Markdown: tables, task lists, fenced code, autolinks.
- Uses GitHub's Markdown style, with light and dark mode that follow your system setting.
- Resolves relative images and links from the file's own folder.
- Writes the page to `$TMPDIR/md-preview/` and opens it with `open` (macOS) or `xdg-open` (Linux). Your Markdown file is never changed.

## How it works

`skills/preview/scripts/render.sh` builds one HTML file. The Markdown text is embedded in the page as base64, and the browser renders it with [marked](https://github.com/markedjs/marked) and [github-markdown-css](https://github.com/sindresorhus/github-markdown-css). Both are loaded from the jsDelivr CDN, pinned to one version and checked with Subresource Integrity hashes.

So:

- Nothing to install: no Pandoc, no Node, no Python.
- The page needs internet access to load those two files. Your Markdown content stays on your machine; it is not uploaded anywhere.
- Raw HTML inside the Markdown is rendered as HTML, like on GitHub. Only preview files you trust.

## Install

Requires Claude Code v2.1.157 or later.

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
```

You can also ask in plain words, for example "preview the plan you just wrote". Claude runs the same script. With no argument, it uses the Markdown file from the conversation, or asks which file you mean.

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
