# 安装 harness-daily

harness-daily 是 **Grok Build / Claude Code / Codex 通用插件**：同一个 GitHub 仓库、同一套 skill/命令、同一份二进制。

安装方式和 [agent-triforce](https://github.com/ArtemioPadilla/agent-triforce) 一样：把 GitHub 仓库加成 plugin marketplace，在你正在用的那套 harness 里安装插件，再跑 setup。

插件只提供 skill / 斜杠命令。采集器是 **本机二进制**。`/harness-daily:setup` 负责下载二进制、写配置、登记每天 08:00 的系统任务。装完之后出报 **不依赖** 聊天窗口开着。

[English](install.md)

## 环境

- Windows / macOS / Linux
- 至少登录以下之一：[Grok Build](https://x.ai/cli)、[Claude Code](https://docs.anthropic.com/en/docs/claude-code)、[Codex CLI](https://github.com/openai/codex)

## Claude Code

在 Claude Code 会话里：

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily@harness-daily
/harness-daily:setup
```

`插件名@市场名` 是 Claude 的安装写法（市场 `harness-daily` 里的插件 `harness-daily`）。setup 会询问日报输出目录（`日报-YYYY-MM-DD.md` 的位置），然后：

1. 从 GitHub Releases 下载 `harness-daily`
2. `harness-daily init --host auto`（若希望用 Claude 写正文，可用 `--host claude`）
3. `harness-daily schedule install --time 08:00`
4. 可选：立刻 `harness-daily report` 生成昨天的日报

## Grok Build

在 Grok 会话或终端：

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily --trust
/harness-daily:setup
```

```sh
grok plugin marketplace add TardisBooo/harness-daily
grok plugin install harness-daily --trust
```

`--trust` 之后 skill / 命令才会加载。Grok 在加过 marketplace 之后用 **插件名** 安装（`harness-daily`），不是 Claude 那种 `name@marketplace`。

## Codex CLI

Codex **没有** Claude 那条 `/plugin marketplace add owner/repo`。若你的 Codex 支持 git/本地源：

```sh
codex plugin install TardisBooo/harness-daily
```

然后 `/harness-daily:setup`，或按下节只装二进制再 `harness-daily init --host codex`。

## 只要二进制

不需要安装 GitHub CLI（`gh`）：

```sh
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.sh | bash

# Windows PowerShell
irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex
```

然后：

```sh
harness-daily init --host auto --out "<日志目录>"
harness-daily doctor
harness-daily report                 # 昨天
harness-daily schedule install --time 08:00
```

`--host auto` 探测顺序：grok → claude → codex。

## 装好后应能看到

| 检查项 | 位置 |
|---|---|
| 插件已列出 | Claude / Grok 的 plugin list 有 `harness-daily` |
| 斜杠命令 | `/harness-daily:setup`、`/harness-daily:harness-daily` |
| 二进制 | `harness-daily --version` |
| 配置 | Windows `%APPDATA%\harness-daily\config.toml` · macOS/Linux `~/.config/harness-daily/config.toml` |
| 定时 | Windows 任务 `harness-daily` · macOS `io.github.harness-daily` · Linux crontab |
| 输出 | `<out>/日报-YYYY-MM-DD.md` |

## 卸载

```sh
harness-daily schedule remove
# Claude: /plugin uninstall harness-daily@harness-daily
# Grok:   grok plugin uninstall harness-daily
```

二进制目录（`%LOCALAPPDATA%\harness-daily` 或 `~/.local/bin/harness-daily`）和配置目录可自行删除。已生成的日报 Markdown 不会被删。

## 排障

- **marketplace add 之后找不到插件** — 刷新 marketplace / 开新会话。确认仓库是 `TardisBooo/harness-daily`。
- **PATH 里没有 harness-daily** — Windows 跑完 install.ps1 后开一个新终端；或直接用 `%LOCALAPPDATA%\harness-daily\harness-daily.exe`。
- **找不到 writer** — 先登录 grok / claude / codex，再 `init --host auto`。
- **错过 08:00** — 任务带 `--auto --backfill 7`，下次运行会补最近 7 天缺的日报。
