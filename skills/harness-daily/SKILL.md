---
name: harness-daily
description: >
  汇总本机 Codex / Claude Code / Grok Build / Pi / OMP 的会话，生成企业日报 Markdown。
  当用户提到日报、工作日志、harness-daily、每天写日报、汇总昨天的工作时使用。
argument-hint: "[--date YYYY-MM-DD] [--backfill N]"
---

# harness-daily

通用日报插件：采集器是独立 CLI `harness-daily`；写正文使用**当前这套 agent CLI** 的登录态
（Grok `grok --prompt-file`、Claude `claude -p`、Codex `codex exec`）。不要另配 API key。

## 生成昨天的日报

```bash
harness-daily report
```

指定日期：

```bash
harness-daily report --date YYYY-MM-DD
```

只采集不调用模型（调试）：

```bash
harness-daily report --date YYYY-MM-DD --dry-collect
```

## 安装与定时

Marketplace（推荐，与 agent-triforce 相同）：

```
/plugin marketplace add TardisBooo/harness-daily
/plugin install harness-daily@harness-daily
/harness-daily:setup
```

Grok 在 add marketplace 之后用 `/plugin install harness-daily --trust`。

已经有二进制时：

```bash
harness-daily init --host auto --out "D:/Me/工作日志"
harness-daily doctor
harness-daily schedule install --time 08:00
```

`--host auto` 按 grok → claude → codex 探测已安装的 CLI。也可显式 `--host grok|claude|codex`。

## 规则

- 「今日工作总结」只按项目写工作内容，不要写工具名和时间。
- 同一项目下各 harness / session 的提问要合并。
- 定时任务走操作系统，不依赖当前会话是否开着。
