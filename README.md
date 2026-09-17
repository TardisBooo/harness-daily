# harness-daily

扫描本机 **Codex / Claude Code / Grok Build / Pi / OMP** 会话，按项目归纳后交给 **Grok Build** 写成企业日报，写入本地 Markdown。

- 采集在本地完成，不上传完整会话。
- 写正文使用你已经登录的 Grok（`grok -p`），不必再配一套 LLM key。
- 每天由系统计划任务触发，不依赖某个 IDE 窗口开着。

## 安装（Grok Build 插件）

需要已安装并登录 [Grok Build](https://github.com/xai-org) CLI。

```bash
# 从本地仓库（开发）
grok plugin install ./harness-daily --trust

# 或放到自动信任目录
# ~/.grok/plugins/harness-daily
```

编译并初始化采集器：

```bash
cargo install --path .
harness-daily init --host grok --out "D:/Me/工作日志"
harness-daily doctor
harness-daily report --date 2026-09-16
harness-daily schedule install --time 08:00
```

Windows 上二进制还会复制到 `%LOCALAPPDATA%\harness-daily\harness-daily.exe`，计划任务走这个路径。

## 日报结构

1. 今日工作总结（只按项目写工作内容）
2. 工作明细
3. 问题与待办
4. 工作量统计
5. 附录：操作审计日志

## 命令

```
harness-daily init --host grok
harness-daily scan
harness-daily doctor
harness-daily report [--date YYYY-MM-DD] [--backfill 7] [--auto] [--dry-collect]
harness-daily schedule install|remove|status|run
```

## 配置

`%APPDATA%/harness-daily/config.toml`（macOS/Linux：`~/.config/harness-daily/config.toml`）

迁走过的数据目录用 `[harnesses.<id>].data_dir` 或 `extra_roots`。

## 许可

MIT
