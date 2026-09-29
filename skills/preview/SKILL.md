---
name: preview
description: Open a Markdown file, or every Markdown file in a folder with a file tree, as a rendered page in the default browser. Use when the user asks to preview, open, or view Markdown files.
argument-hint: "[file.md | folder]"
allowed-tools: Bash(${CLAUDE_SKILL_DIR}/scripts/render.sh *)
---

Render the file or folder in `$ARGUMENTS` and open it in the browser by running:

`${CLAUDE_SKILL_DIR}/scripts/render.sh "<path to the file or folder>"`

A folder opens every Markdown file inside it, with a file tree to move between them. If `$ARGUMENTS` is empty, use the Markdown file or folder the user is talking about, or the file you wrote most recently in this session. If there is no clear target, ask which one.

The script prints the path of the HTML page it opened, and warns on stderr when a folder has more files than it shows. On the first run it compiles the renderer, which takes about a minute; wait for it. Reply with one short line that names what you opened, and repeat that warning if there was one. Do not read, summarize, or change the Markdown files.
