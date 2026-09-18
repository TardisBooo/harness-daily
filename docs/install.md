# Install harness-daily

harness-daily is a **universal plugin** for Grok Build, Claude Code, and Codex. Same GitHub repo,
same skills/commands, same binary.

This is the same marketplace flow as
[agent-triforce](https://github.com/ArtemioPadilla/agent-triforce): add the GitHub repo as a
plugin source, install the plugin in the harness you use, then run setup.

The plugin ships skills and slash commands. The collector is a **native binary**. `/harness-daily:setup`
downloads that binary, writes config, and registers a daily OS task. Reports do **not** require an
open chat window after setup.

[简体中文](install.zh-CN.md)

## Prerequisites

- Windows, macOS, or Linux
- At least one logged-in CLI: [Grok Build](https://x.ai/cli), [Claude Code](https://docs.anthropic.com/en/docs/claude-code), or [Codex CLI](https://github.com/openai/codex)

## Claude Code

In a Claude Code session:

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily@harness-daily
/harness-daily:setup
```

`plugin@marketplace` is Claude's namespaced install (`harness-daily` plugin from the
`harness-daily` marketplace). Setup will ask for an output directory (where `日报-YYYY-MM-DD.md`
is written) and then:

1. Download `harness-daily` from GitHub Releases
2. `harness-daily init --host auto` (or `--host claude` if you want Claude to write the body)
3. `harness-daily schedule install --time 08:00`
4. Optionally `harness-daily report` for yesterday

## Grok Build

In a Grok session, or from a shell:

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily --trust
/harness-daily:setup
```

```sh
grok plugin marketplace add TardisBooo/harness-daily
grok plugin install harness-daily --trust
```

`--trust` is required so Grok loads skills/commands. Grok's install flag is the plugin name after
the marketplace is added (`harness-daily`), not `name@marketplace`.

## Codex CLI

Codex does not use Claude's `/plugin marketplace add owner/repo` string. Install from git/local if
your Codex build supports it:

```sh
codex plugin install TardisBooo/harness-daily
```

Then `/harness-daily:setup`, or install the binary yourself (next section) and
`harness-daily init --host codex`.

## Binary only

No `gh` CLI required:

```sh
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash

# Windows PowerShell
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex
```

Then:

```sh
harness-daily init --host auto --out "<logs-dir>"
harness-daily doctor
harness-daily report                 # yesterday
harness-daily schedule install --time 08:00
```

`--host auto` tries grok, then claude, then codex.

## What you should see

| Check | Command / place |
|---|---|
| Plugin listed | Claude/Grok plugin list shows `harness-daily` |
| Slash commands | `/harness-daily:setup`, `/harness-daily:harness-daily` |
| Binary | `harness-daily --version` |
| Config | Windows `%APPDATA%\harness-daily\config.toml` · macOS/Linux `~/.config/harness-daily/config.toml` |
| Schedule | Windows task `harness-daily` · macOS `io.github.harness-daily` · Linux crontab line |
| Output | `<out>/日报-YYYY-MM-DD.md` |

## Uninstall

```sh
harness-daily schedule remove
# Claude: /plugin uninstall harness-daily@harness-daily
# Grok:   grok plugin uninstall harness-daily
```

Delete the binary directory (`%LOCALAPPDATA%\harness-daily` or `~/.local/bin/harness-daily`) and
the config directory if you want a clean slate. Report Markdown files are not deleted.

## Troubleshooting

- **`plugin not found` after marketplace add** — run marketplace update / start a new session. Confirm you used `TardisBooo/harness-daily` (GitHub `owner/repo`).
- **`harness-daily` not on PATH** — Windows: open a new terminal after install.ps1. Or call `%LOCALAPPDATA%\harness-daily\harness-daily.exe` directly.
- **Writer not found** — install and log into grok, claude, or codex, then `init --host auto` again.
- **Schedule missed 08:00** — the task runs `report --auto --backfill 7` and fills gaps on the next run.
