<div align="center">

# harness-daily

**一条命令，把昨天的 AI 编码会话变成企业风格的日报 —— 由你已经在用的编码 agent 亲自写。**

_本地扫描 · Grok Build 写正文 · 无需额外 API Key · Windows / macOS / Linux_

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
- **正文由 Grok Build 撰写** —— 用 `grok -p` 和你已有的登录态写日报，不需要第二个 API key。
- **企业日报格式** —— 「今日工作」按项目合并成完整工作条目；原始提问放审计附录，不混进总结。
- **自愈调度** —— 系统任务每天执行 `report --auto --backfill 7`，关机错过 08:00，开机自动补。

```text
08:00（系统任务）→ harness-daily report
   ├─ 发现数据目录   （codex / claude / grok / pi / omp，含共用这些目录的桌面端）
   ├─ 采集           （解析 JSONL、去重、剔除「继续/ok/连通性测试」）
   ├─ Grok 写正文    （grok --prompt-file … --output-format json；你的登录态与默认模型）
   └─ 渲染           D:/Me/工作日志/日报-YYYY-MM-DD.md   ← 总结 · 明细 · 问题 · 统计 · 审计
```

## 🧭 工作原理

| 组件 | 职责 |
|---|---|
| `harness-daily`（Rust 二进制） | 发现各 harness 数据目录（识别 Windows junction）、解析会话日志、按项目聚合、调用 grok、渲染 Markdown、安装系统定时任务 |
| Grok 插件（`skills/`、`commands/`） | 让你在交互式 Grok 会话里用 `/harness-daily` |
| `grok -p`（无头模式） | 读内嵌采集 JSON，返回严格 JSON 正文 |
| 系统调度器 | Windows `schtasks` / macOS `launchd` / Linux cron，每天触发 `report --auto --backfill 7` |

内置支持：**Codex CLI**（`~/.codex`）、**Claude Code**（`~/.claude`）、**Grok Build**
（`~/.grok`）、**Pi**（`~/.pi`）、**OMP**（`~/.omp`）。与这些目录共用的桌面端自动覆盖；
数据目录搬家了，把 `[harnesses.<id>].data_dir` 指过去即可。

## ✅ 环境要求

- 已安装并**登录** [Grok Build](https://x.ai/cli)（它提供写正文的模型）
- Windows / macOS / Linux
- 可选：[Rust](https://rustup.rs)（源码构建时）

## 📦 安装

### 1. 获取二进制

**从 Releases 下载**（推荐）：

```sh
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash   # macOS / Linux
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex          # Windows PowerShell
```

**或用 cargo：**

```sh
cargo install --git https://github.com/TardisBooo/harness-daily
```

**或源码构建：**

```sh
git clone https://github.com/TardisBooo/harness-daily
cd harness-daily && cargo install --path .
```

### 2. 安装 Grok 插件

```sh
grok plugin install TardisBooo/harness-daily --trust
```

装完后，交互式 Grok 会话里就有 `/harness-daily` 命令。

## 🚀 快速上手

```sh
harness-daily init --host grok --out "D:/Me/工作日志"   # 探测 harness、写配置
harness-daily doctor                                    # 检查 grok 登录、路径
harness-daily report                                    # 立刻生成昨天的日报
harness-daily schedule install --time 08:00             # 每天 08:00，自动补漏
```

指定日期或补一周：

```sh
harness-daily report --date 2026-09-16
harness-daily report --backfill 7 --auto
```

## 🎛 命令

```
harness-daily init --host grok [--out DIR] [--time 08:00]   写配置并本地安装二进制
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
host = "grok"               # 写正文的 CLI
bin  = 'C:\Users\you\.grok\bin\grok.exe'

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
- **换写正文的 CLI**：流水线按可替换 writer 设计，当前发布 `--host grok`。
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
