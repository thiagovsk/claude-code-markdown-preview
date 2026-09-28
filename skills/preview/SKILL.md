---
name: preview
description: Open a Markdown file as a rendered page in the default browser. Use when the user asks to preview, open, or view a Markdown file.
argument-hint: "[file.md]"
allowed-tools: Bash(${CLAUDE_SKILL_DIR}/scripts/render.sh *)
---

Render the Markdown file in `$ARGUMENTS` and open it in the browser by running:

`${CLAUDE_SKILL_DIR}/scripts/render.sh "<path to the file>"`

If `$ARGUMENTS` is empty, use the Markdown file the user is talking about or the one you wrote most recently in this session. If there is no clear file, ask which one.

The script prints the path of the HTML page it opened. Reply with one short line that names the file. Do not read, summarize, or change the Markdown file.
