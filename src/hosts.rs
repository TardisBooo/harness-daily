use crate::config::Config;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    Grok,
    Claude,
    Codex,
}

impl Host {
    pub fn id(self) -> &'static str {
        match self {
            Host::Grok => "grok",
            Host::Claude => "claude",
            Host::Codex => "codex",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Host::Grok => "Grok Build",
            Host::Claude => "Claude Code",
            Host::Codex => "Codex",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" | "" => anyhow::bail!("auto 不是具体 host"),
            "grok" | "grok-build" => Ok(Host::Grok),
            "claude" | "claude-code" => Ok(Host::Claude),
            "codex" => Ok(Host::Codex),
            other => {
                anyhow::bail!("不支持的 writer host: {other}（可选 grok / claude / codex / auto）")
            }
        }
    }

    pub fn detect_order() -> [Host; 3] {
        [Host::Grok, Host::Claude, Host::Codex]
    }
}

pub fn resolve_host(cfg: &Config) -> Result<(Host, PathBuf)> {
    let raw = cfg.writer.host.trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("auto") {
        for host in Host::detect_order() {
            if let Some(bin) = find_host_bin(host, "") {
                return Ok((host, bin));
            }
        }
        anyhow::bail!("未找到 grok / claude / codex，请安装其中之一并登录后再 init");
    }
    let host = Host::parse(raw)?;
    let bin = find_host_bin(host, &cfg.writer.bin)
        .with_context(|| format!("找不到 {} 可执行文件", host.label()))?;
    Ok((host, bin))
}

pub fn find_host_bin(host: Host, configured: &str) -> Option<PathBuf> {
    let configured = configured.trim();
    if !configured.is_empty() {
        let p = PathBuf::from(configured);
        if p.exists() && usable_bin(host, &p) {
            return Some(p);
        }
    }
    match host {
        Host::Grok => find_grok(),
        Host::Claude => find_named(
            "claude",
            &[
                extra_npm_bin("@anthropic-ai/claude-code/bin/claude.exe"),
                extra_npm_bin("@anthropic-ai/claude-code/bin/claude"),
            ],
        ),
        Host::Codex => find_named(
            "codex",
            &[
                extra_npm_bin("@openai/codex/bin/codex.js"),
                dirs::home_dir().map(|h| h.join(".codex").join("bin").join("codex.exe")),
            ],
        ),
    }
}

fn extra_npm_bin(rel: &str) -> Option<PathBuf> {
    let mut cands = Vec::new();
    if let Some(d) = dirs::config_dir() {
        cands.push(d.join("npm").join("node_modules").join(rel));
    }
    if let Some(d) = dirs::data_dir() {
        cands.push(d.join("npm").join("node_modules").join(rel));
    }
    cands.into_iter().find(|p| p.exists())
}

fn find_grok() -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let candidates = [
        home.join(".grok").join("bin").join("grok.exe"),
        home.join(".grok").join("bin").join("grok"),
        PathBuf::from(r"D:\SOFTWARE\OpenAgents\bin\grok.exe"),
    ];
    if let Some(p) = candidates.into_iter().find(|p| p.exists()) {
        return Some(p);
    }
    which_prefer("grok", true)
}

fn find_named(name: &str, extras: &[Option<PathBuf>]) -> Option<PathBuf> {
    for extra in extras.iter().flatten() {
        if extra.exists() && usable_bin_path(extra, false) {
            return Some(extra.clone());
        }
    }
    which_prefer(name, false)
}

fn which_prefer(name: &str, skip_cmd: bool) -> Option<PathBuf> {
    let names = [
        format!("{name}.exe"),
        name.to_string(),
        format!("{name}.cmd"),
    ];
    for n in names {
        if let Ok(p) = which::which(&n) {
            if usable_bin_path(&p, skip_cmd) {
                return Some(p);
            }
        }
    }
    None
}

fn usable_bin(host: Host, p: &Path) -> bool {
    usable_bin_path(p, matches!(host, Host::Grok))
}

fn usable_bin_path(p: &Path, skip_cmd: bool) -> bool {
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "ps1" {
        return false;
    }
    if skip_cmd && (ext == "cmd" || ext == "bat") {
        return false;
    }
    p.exists()
}

pub fn default_extra_args(host: Host) -> Vec<String> {
    match host {
        Host::Grok => vec!["--always-approve".into()],
        Host::Claude => vec!["--dangerously-skip-permissions".into()],
        Host::Codex => vec![],
    }
}
