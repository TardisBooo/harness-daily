---
description: Install the harness-daily binary, write config, and enable the daily 08:00 report task
argument-hint: "[output-dir]"
---

# harness-daily setup

Scaffold a working harness-daily install on this machine. Goal: from plugin install to first report in a few minutes.

The plugin only adds slash commands. The collector is a native binary; this command installs it, writes config, and registers the OS schedule.

User arguments (optional output directory): `$ARGUMENTS`

## Step 1 — Detect OS and existing install

- Windows: look for `%LOCALAPPDATA%\harness-daily\harness-daily.exe`
- macOS / Linux: look for `~/.local/bin/harness-daily` or `harness-daily` on PATH
- Config: `%APPDATA%\harness-daily\config.toml` (Windows) or `~/.config/harness-daily/config.toml`

If the binary already exists, run `harness-daily doctor` and skip to Step 3 unless the user asked for a full reinstall.

## Step 2 — Install the binary

Do **not** require the GitHub CLI. Prefer the published install scripts.

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex
```

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash
```

If the script fails (no network, unsupported arch), fall back to:

```bash
cargo install --git https://github.com/TardisBooo/harness-daily
```

After install, confirm `harness-daily --version` (on Windows, new terminals may be needed for PATH; you can invoke the absolute path from Step 1).

## Step 3 — Init

Ask for the report output directory if `$ARGUMENTS` is empty. Default:

- Windows: `D:/Me/工作日志` if that folder exists, otherwise `%USERPROFILE%/work-logs`
- macOS / Linux: `~/work-logs`

Then run:

```bash
harness-daily init --host auto --out "<output-dir>" --time 08:00
```

`--host auto` picks grok → claude → codex. If the user is clearly inside Claude Code, you may use `--host claude`; inside Grok Build, `--host grok`; inside Codex, `--host codex`.

## Step 4 — Schedule

```bash
harness-daily schedule install --time 08:00
```

On Linux, if the command only prints a crontab line, show it and ask the user to add it.

## Step 5 — First report (optional)

Ask whether to generate yesterday's report now:

```bash
harness-daily report
```

If they decline, say the next automatic run is 08:00 local (config timezone, default Asia/Shanghai).

## Step 6 — Summary

Print:

```
harness-daily setup complete
  binary:  <path>
  writer:  <host> (<bin>)
  output:  <dir>
  schedule: daily 08:00 (OS task, --auto --backfill 7)

Next:
  /harness-daily:harness-daily --date YYYY-MM-DD
  harness-daily doctor
  harness-daily report --dry-collect
```

Do not overwrite an existing config without asking. Do not store API keys; the writer uses the already-logged-in agent CLI.
