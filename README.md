<div align="center">

# harness-daily

**One command turns yesterday's AI coding sessions into an enterprise-style daily report — written by whichever agent CLI you already use.**

_Scan locally · Summarize with Grok Build · No extra API keys · Windows / macOS / Linux_

[![CI](https://github.com/TardisBooo/harness-daily/actions/workflows/ci.yml/badge.svg)](https://github.com/TardisBooo/harness-daily/actions/workflows/ci.yml)
[![Release](https://github.com/TardisBooo/harness-daily/actions/workflows/release.yml/badge.svg)](https://github.com/TardisBooo/harness-daily/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-32CD32.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Platforms](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-blue)

**English** | [简体中文](README.zh-CN.md)

</div>

---

## 📋 Quick Navigation

[Why harness-daily?](#-why-harness-daily) ·
[How it works](#-how-it-works) ·
[Requirements](#-requirements) ·
[Install](#-install) ·
[Quick start](#-quick-start) ·
[Commands](#-commands) ·
[Configuration](#-configuration) ·
[Report format](#-report-format) ·
[Extending](#-extending) ·
[Development](#-development) ·
[Contributing](#-contributing) ·
[License](#-license)

---

## 🤔 Why harness-daily?

If you work with multiple AI coding agents — Codex, Claude Code, Grok Build, Pi, OMP — your day is
scattered across dozens of sessions in half a dozen projects. Writing a daily standup report means
digging through histories by hand.

harness-daily does it for you:

- **Scans every harness locally** — reads session/prompt logs from disk; nothing leaves your machine
  except the distilled prompts you choose to send to your own Grok Build CLI.
- **Writes the prose with your current agent CLI** — Grok Build, Claude Code, or Codex, using
  the login you already have. No second API key. `init --host auto` picks the first one found.
- **Enterprise-report format** — "today's work" grouped by project with merged work items; raw
  prompts go to an audit appendix, not the summary.
- **Self-healing schedule** — the OS task runs `report --auto --backfill 7`, so a powered-off
  machine at 08:00 still gets its missing reports the next time it boots.

```text
08:00 (OS task) → harness-daily report
   ├─ discover roots  (codex / claude / grok / pi / omp, incl. desktop apps sharing those dirs)
   ├─ collect         (parse JSONL, dedupe, drop "continue"/"ok"/smoke-test noise)
   ├─ write            grok --prompt-file / claude -p / codex exec  (your login, your default model)
   └─ render          D:/Me/工作日志/日报-YYYY-MM-DD.md   ← summary · details · issues · stats · audit log
```

## 🧭 How it works

| Piece | Responsibility |
|---|---|
| `harness-daily` (Rust binary) | Discover harness data dirs, parse session logs, aggregate prompts per project, call the writer CLI, render Markdown, install the OS schedule |
| Universal plugin (`skills/`, `commands/`, `plugin.json`, `.claude-plugin/`, `plugin.yaml`) | Same repo installs into **Grok Build**, **Claude Code**, or **Codex** |
| Writer host | `grok --prompt-file` · `claude -p --output-format json` · `codex exec` (stdin) |
| OS scheduler | Windows `schtasks` / macOS `launchd` / Linux cron — fires `report --auto --backfill 7` daily |

Supported out of the box: **Codex CLI** (`~/.codex`), **Claude Code** (`~/.claude`), **Grok Build**
(`~/.grok`), **Pi** (`~/.pi`), **OMP** (`~/.omp`). Desktop apps that share those directories are
covered automatically. Moved a data dir? Point `[harnesses.<id>].data_dir` at the new location.

## ✅ Requirements

- At least one of [Grok Build](https://x.ai/cli), [Claude Code](https://docs.anthropic.com/en/docs/claude-code), or [Codex CLI](https://github.com/openai/codex), **logged in**
- Windows, macOS, or Linux
- Optional: [Rust](https://rustup.rs) if you build from source

## 📦 Install

### 1. Get the binary

**Download from Releases** (recommended):

```sh
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash   # macOS / Linux
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex          # Windows PowerShell
```

**Or with cargo:**

```sh
cargo install --git https://github.com/TardisBooo/harness-daily
```

**Or from source:**

```sh
git clone https://github.com/TardisBooo/harness-daily
cd harness-daily && cargo install --path .
```

### 2. Install the plugin into the CLI you use

Same repository, three hosts:

```sh
grok plugin install TardisBooo/harness-daily --trust     # Grok Build
claude plugin install TardisBooo/harness-daily           # Claude Code (marketplace / local path)
codex plugin install TardisBooo/harness-daily            # Codex (if your Codex build supports git/local sources)
```

This adds `/harness-daily` (or the equivalent skill) inside that CLI. Daily generation still
runs from the OS scheduler + the `harness-daily` binary, not from a live chat window.

## 🚀 Quick start

```sh
harness-daily init --host auto --out "D:/Me/工作日志"   # grok → claude → codex
harness-daily doctor
harness-daily report
harness-daily schedule install --time 08:00
```

Force a writer:

```sh
harness-daily init --host grok
harness-daily init --host claude
harness-daily init --host codex
```

Generate a specific day or catch up a week:

```sh
harness-daily report --date 2026-09-16
harness-daily report --backfill 7 --auto
```

## 🎛 Commands

```
harness-daily init --host auto|grok|claude|codex [--out DIR] [--time 08:00]
harness-daily scan                                          list detected harness data roots
harness-daily doctor                                        self-check grok login, paths, output dir
harness-daily report [--date D] [--backfill N] [--auto]
                         [--dry-collect] [--out DIR]        generate report(s)
harness-daily schedule install|remove|status|run            manage the OS scheduled task
```

`--auto` skips dates whose report file already exists; `--dry-collect` only writes the collection
JSON (no model call) for debugging.

## ⚙️ Configuration

`%APPDATA%/harness-daily/config.toml` (Windows) · `~/.config/harness-daily/config.toml` (macOS/Linux)

```toml
timezone    = "Asia/Shanghai"
output_dir  = 'D:\Me\工作日志'
report_time = "08:00"
backfill_days = 7
extra_roots = []            # extra scan roots (moved data dirs)

[writer]
host = "auto"               # grok | claude | codex | auto
bin  = ""                   # empty = discover; or an absolute path

[report]
include_audit_log = true    # set false to omit the raw-prompt appendix
include_stats     = true

[harnesses.codex]
enabled  = true
data_dir = ""               # override if you moved ~/.codex
```

## 📄 Report format

```markdown
# 工作日报 · 2026-09-16（周三）
## 一、今日工作总结      ← per project, merged work items (no tools, no times)
## 二、工作明细          ← path · time span · completed items
## 三、问题与待办        ← issues the model extracted
## 四、工作量统计        ← sessions / prompts per harness
## 附录：操作审计日志    ← every raw prompt of the day, time-sorted
```

## 🧩 Extending

- **New harness / desktop app**: copy [`adapters/example-jsonl.toml`](adapters/example-jsonl.toml)
  into your config dir and edit the paths/keys. Built-in Rust adapters live in `src/collect.rs`;
  PRs welcome (see [`CONTRIBUTING.md`](CONTRIBUTING.md)).
- **Writer CLI**: `--host grok|claude|codex|auto`. Collection always covers all five harnesses.
- **Privacy**: everything is local. Only the per-project prompt digest is passed to your own
  `grok -p` process; the audit appendix can be disabled.

## 🛠 Development

```sh
cargo test                 # unit tests (JSON parsing, timezone bounds, noise filters)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run -- scan           # try discovery against your real machine
```

Repository layout: `src/collect.rs` (per-harness collectors) · `src/writer.rs` (grok headless
invocation + strict-JSON recovery) · `src/render.rs` (Markdown assembly) · `src/schedule.rs`
(Windows/macOS/Linux schedulers) · `skills/` + `commands/` (Grok plugin).

## 🤝 Contributing

Bug reports and new-harness adapters are very welcome — see
[CONTRIBUTING.md](CONTRIBUTING.md). For security issues, see [SECURITY.md](SECURITY.md).

## 📜 License

[MIT](LICENSE)
