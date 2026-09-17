<div align="center">

# harness-daily

**一条命令，把昨天的 AI 编码会话变成企业风格的日报 —— 由你已经在用的那套 agent CLI 亲自写。**

_本地扫描 · Grok / Claude / Codex 写正文 · Marketplace 安装 · 无需额外 API Key_

[![CI](https://github.com/TardisBooo/harness-daily/actions/workflows/ci.yml/badge.svg)](https://github.com/TardisBooo/harness-daily/actions/workflows/ci.yml)
[![Release](https://github.com/TardisBooo/harness-daily/actions/workflows/release.yml/badge.svg)](https://github.com/TardisBooo/harness-daily/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-32CD32.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Platforms](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-blue)

[English](README.md) | **简体中文**

</div>

---

## 📋 快速导航

[为什么需要 harness-daily？](#-为什么需要-harness-daily) ·
[工作原理](#-工作原理) ·
[环境要求](#-环境要求) ·
[安装](#-安装) ·
[安装教程](docs/install.zh-CN.md) ·
[快速上手](#-快速上手) ·
[命令](#-命令) ·
[配置](#-配置) ·
[日报格式](#-日报格式) ·
[扩展](#-扩展) ·
[开发](#-开发) ·
[参与贡献](#-参与贡献) ·
[许可](#-许可)

---

## 🤔 为什么需要 harness-daily？

同时用多个 AI 编码工具（Codex、Claude Code、Grok Build、Pi、OMP）的人，一天的工作散落在
几十个会话、若干个项目里。写日报 = 手工翻历史。

harness-daily 替你做：

- **本地扫描所有 harness** —— 直接读会话/提问日志；除了交给「你自己的 Grok CLI」的项目摘要外，
  任何数据都不出本机。
- **正文由当前 agent CLI 撰写** —— Grok Build、Claude Code 或 Codex，用你已有的登录态。不需要第二个 API key。`init --host auto` 按 grok → claude → codex 选用第一套能找到的。
- **企业日报格式** —— 「今日工作」按项目合并成完整工作条目；原始提问放审计附录，不混进总结。
- **自愈调度** —— 系统任务每天执行 `report --auto --backfill 7`，关机错过 08:00，开机自动补。

```text
08:00（系统任务）→ harness-daily report
   ├─ 发现数据目录   （codex / claude / grok / pi / omp，含共用这些目录的桌面端）
   ├─ 采集           （解析 JSONL、去重、剔除「继续/ok/连通性测试」）
   ├─ 写正文         grok --prompt-file / claude -p / codex exec（你的登录态与默认模型）
   └─ 渲染           D:/Me/工作日志/日报-YYYY-MM-DD.md   ← 总结 · 明细 · 问题 · 统计 · 审计
```

## 🧭 工作原理

| 组件 | 职责 |
|---|---|
| `harness-daily`（Rust 二进制） | 发现数据目录、解析会话、按项目聚合、调用写正文 CLI、渲染 Markdown、安装系统定时任务 |
| 通用插件（`skills/`、`commands/`、`plugin.json`、`.claude-plugin/`、`plugin.yaml`） | 同一仓库可装进 **Grok Build / Claude Code / Codex** |
| 写正文宿主 | `grok --prompt-file` · `claude -p --output-format json` · `codex exec`（stdin） |
| 系统调度器 | Windows `schtasks` / macOS `launchd` / Linux cron，每天触发 `report --auto --backfill 7` |

内置支持：**Codex CLI**（`~/.codex`）、**Claude Code**（`~/.claude`）、**Grok Build**
（`~/.grok`）、**Pi**（`~/.pi`）、**OMP**（`~/.omp`）。与这些目录共用的桌面端自动覆盖；
数据目录搬家了，把 `[harnesses.<id>].data_dir` 指过去即可。

## ✅ 环境要求

- 至少安装并**登录** [Grok Build](https://x.ai/cli)、[Claude Code](https://docs.anthropic.com/en/docs/claude-code) 或 [Codex CLI](https://github.com/openai/codex) 之一
- Windows / macOS / Linux
- 可选：[Rust](https://rustup.rs)（源码构建时）

## 📦 安装

推荐走 **plugin marketplace**（和 [agent-triforce](https://github.com/ArtemioPadilla/agent-triforce) 同一套）：把 GitHub 仓库加成市场源，安装插件，再跑 setup。setup 会下载本机二进制、写配置、登记每天 08:00 的系统任务。逐步说明见 [docs/install.zh-CN.md](docs/install.zh-CN.md) · [English](docs/install.md)。

### Claude Code

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily@harness-daily
/harness-daily:setup
```

### Grok Build

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily --trust
/harness-daily:setup
```

终端等价命令：

```sh
grok plugin marketplace add TardisBooo/harness-daily
grok plugin install harness-daily --trust
```

### Codex CLI

```sh
codex plugin install TardisBooo/harness-daily
```

然后在会话里 `/harness-daily:setup`，或在终端：

```sh
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex   # Windows
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash
harness-daily init --host auto
harness-daily schedule install --time 08:00
```

插件只提供斜杠命令。每天出报由 **二进制 + 系统定时任务** 完成，不依赖聊天窗口开着。

### 只要二进制（不装插件）

```sh
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex
cargo install --git https://github.com/TardisBooo/harness-daily
```

## 🚀 快速上手

Marketplace 装完后跑 `/harness-daily:setup` 即可。终端：

```sh
harness-daily init --host auto --out "D:/Me/工作日志"   # grok → claude → codex
harness-daily doctor
harness-daily report
harness-daily schedule install --time 08:00
```

指定写正文的 CLI：

```sh
harness-daily init --host grok
harness-daily init --host claude
harness-daily init --host codex
```

指定日期或补一周：

```sh
harness-daily report --date 2026-09-16
harness-daily report --backfill 7 --auto
```

## 🎛 命令

```
harness-daily init --host auto|grok|claude|codex [--out DIR] [--time 08:00]
harness-daily scan                                          列出探测到的数据目录
harness-daily doctor                                        自检（grok 登录、路径、输出目录）
harness-daily report [--date D] [--backfill N] [--auto]
                         [--dry-collect] [--out DIR]        生成日报
harness-daily schedule install|remove|status|run            管理系统定时任务
```

`--auto` 跳过已存在的日报文件；`--dry-collect` 只输出采集 JSON，不调用模型（调试用）。

## ⚙️ 配置

`%APPDATA%/harness-daily/config.toml`（Windows）· `~/.config/harness-daily/config.toml`（macOS/Linux）

```toml
timezone    = "Asia/Shanghai"
output_dir  = 'D:\Me\工作日志'
report_time = "08:00"
backfill_days = 7
extra_roots = []            # 额外扫描根（搬过家的数据目录）

[writer]
host = "auto"               # grok | claude | codex | auto
bin  = ""                   # 空=自动探测；或填绝对路径

[report]
include_audit_log = true    # false 可关闭原始提问附录
include_stats     = true

[harnesses.codex]
enabled  = true
data_dir = ""               # ~/.codex 搬家后在此覆盖
```

## 📄 日报格式

```markdown
# 工作日报 · 2026-09-16（周三）
## 一、今日工作总结      ← 按项目组织，合并后的工作条目（不写工具、时间）
## 二、工作明细          ← 路径 · 时段 · 完成内容
## 三、问题与待办        ← 模型提取的待跟进问题
## 四、工作量统计        ← 各 harness 会话数 / 提问数
## 附录：操作审计日志    ← 当日全部原始提问，按时间排序
```

## 🧩 扩展

- **新增 harness / 桌面端**：复制 [`adapters/example-jsonl.toml`](adapters/example-jsonl.toml)
  到配置目录改路径/字段；内置 Rust 适配器在 `src/collect.rs`，欢迎 PR（见
  [`CONTRIBUTING.md`](CONTRIBUTING.md)）。
- **写正文 CLI**：`--host grok|claude|codex|auto`。采集始终覆盖全部五家 harness。
- **隐私**：全部本地。只有「项目级提问摘要」会传给你自己的 `grok -p` 进程；审计附录可关闭。

## 🛠 开发

```sh
cargo test                 # 单元测试（JSON 解析、时区边界、噪声过滤）
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run -- scan           # 用你真实的机器试探测
```

目录结构：`src/collect.rs`（各 harness 采集器）· `src/writer.rs`（grok 无头调用与严格 JSON
恢复）· `src/render.rs`（Markdown 拼装）· `src/schedule.rs`（三平台调度器）· `skills/` +
`commands/`（Grok 插件）。

## 🤝 参与贡献

欢迎提交 bug 报告与新的 harness 适配器 —— 见 [CONTRIBUTING.md](CONTRIBUTING.md)。
安全问题见 [SECURITY.md](SECURITY.md)。

## 📜 许可

[MIT](LICENSE)
