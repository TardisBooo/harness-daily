---
name: harness-daily
description: >
  汇总本机 Codex / Claude Code / Grok Build / Pi / OMP 的会话，生成企业日报 Markdown。
  当用户提到日报、工作日志、harness-daily、每天写日报、汇总昨天的工作时使用。
argument-hint: "[--date YYYY-MM-DD] [--backfill N]"
---

# harness-daily

用本机已安装的 `harness-daily` CLI 采集各 harness 会话，再由当前 Grok 会话写正文。

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

## 安装与定时（用户本机）

```bash
harness-daily init --host grok --out "D:/Me/工作日志"
harness-daily doctor
harness-daily schedule install --time 08:00
```

## 规则

- 日报正文由 Grok 根据采集 JSON 归纳；不要另配 API key。
- 「今日工作总结」只按项目写工作内容，不要写工具名和时间。
- 本插件不安装到 Codex。
