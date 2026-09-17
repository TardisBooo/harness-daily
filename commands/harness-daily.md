---
description: 扫描本机 AI coding harness 会话并生成日报
argument-hint: "[--date YYYY-MM-DD]"
---

运行本机采集器（写正文使用当前已配置的 agent CLI：Grok / Claude / Codex）：

```bash
harness-daily report $ARGUMENTS
```

若二进制不在 PATH：

- Windows：`%LOCALAPPDATA%\harness-daily\harness-daily.exe`
- macOS / Linux：`~/.local/bin/harness-daily`

尚未初始化时先执行：

```bash
harness-daily init --host auto
```
