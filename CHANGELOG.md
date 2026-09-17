# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Universal writer hosts: `--host auto|grok|claude|codex` (`grok --prompt-file`,
  `claude -p`, `codex exec` on stdin).
- Plugin manifests for Claude Code (`.claude-plugin/`) and Codex (`plugin.yaml`)
  in the same repository as the Grok plugin.

## [0.1.0] — 2026-09-17

### Added

- Collectors for Codex CLI (`~/.codex`), Claude Code (`~/.claude`), Grok Build (`~/.grok`),
  Pi (`~/.pi`), OMP (`~/.omp`) with junction-aware root discovery on Windows.
- Report pipeline: per-project prompt aggregation → noise/smoke-test filtering →
  body written by headless `grok -p` (strict JSON with multi-shape recovery) → Markdown render.
- Enterprise-style report: summary (projects only), details, issues, per-harness stats,
  and an optional raw-prompt audit appendix.
- `init` / `scan` / `doctor` / `report` / `schedule` commands.
- OS scheduling: Windows `schtasks`, macOS `launchd`, Linux cron guidance;
  `--auto --backfill N` self-healing for missed days.
- Grok Build plugin (`skills/harness-daily`, `/harness-daily` slash command).
- CI (fmt + clippy + test on Windows/macOS/Linux) and release workflow with
  per-platform binaries + install scripts.

[0.1.0]: https://github.com/TardisBooo/harness-daily/releases/tag/v0.1.0
