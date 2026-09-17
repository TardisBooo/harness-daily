# Contributing to harness-daily

Thanks for your interest in improving harness-daily!

## Ways to help

- **New harness adapter** — the highest-value contribution. See below.
- **Bug reports** — open an issue with: OS, harness versions, `harness-daily doctor` output,
  and (redacted) logs. Never paste API keys or secrets.
- **Writer integrations** — new `--host` backends (claude/pi headless) must follow the same
  contract: strict JSON in, non-zero exit on failure.

## Adding a harness adapter

1. Study real data layout of the target CLI (sessions dir, timestamp format, user-prompt field).
2. Implement a collector in `src/collect.rs` following `collect_codex`/`collect_claude`:
   - Accept `(root, day_start, day_end)` and return `Vec<Activity>`.
   - Parse defensively: skip malformed lines, never panic on unexpected shapes.
   - Normalize timestamps to UTC; dedupe re-sent prompts within a 2-minute window.
3. Add a detection entry in `src/discovery.rs` (default paths + fingerprint).
4. Add fixtures under `tests/fixtures/<id>/` — **synthetic data only**, never real session logs.
5. Update the support table in both READMEs.

Run the full gate before opening a PR:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same three steps on Windows, macOS, and Linux.

## Design constraints

- **Local-first**: collectors read files only; the only outbound call is the user's own
  writer CLI (`grok -p`).
- **No partial reports**: if the writer fails, no report file is written.
- **Std-library-leaning**: keep the dependency tree small; justify any new crate.

## Commit style

Conventional commits (`feat:`, `fix:`, `docs:`, `chore:`) are appreciated but not enforced.

## License

By contributing, you agree your contributions are licensed under the MIT License.
