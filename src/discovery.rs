use crate::config::Config;
use crate::model::HarnessRoot;
use crate::util::resolve_path;
use std::path::{Path, PathBuf};

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn builtin_candidates(id: &str) -> Vec<PathBuf> {
    let home = home_dir();
    let mut v = vec![home.join(format!(".{id}"))];
    if id == "codex" && cfg!(windows) {
        v.push(PathBuf::from(r"D:\SOFTWARE_DATA\Codex\Home"));
    }
    v
}

pub fn detect_root(id: &str, cfg: &Config) -> Option<PathBuf> {
    if let Some(p) = cfg.harness_override_dir(id) {
        if p.exists() {
            return Some(resolve_path(&p));
        }
    }
    for c in builtin_candidates(id) {
        if c.exists() {
            return Some(resolve_path(&c));
        }
    }
    None
}

fn fingerprint_ok(id: &str, root: &Path) -> bool {
    match id {
        "codex" => root.join("history.jsonl").exists() || root.join("sessions").exists(),
        "claude" => root.join("projects").exists(),
        "grok" => root.join("sessions").exists(),
        "pi" | "omp" => root.join("agent").join("sessions").exists() || root.join("agent").exists(),
        _ => root.exists(),
    }
}

pub fn scan_roots(cfg: &Config) -> Vec<HarnessRoot> {
    let mut out = Vec::new();
    for id in ["codex", "claude", "grok", "pi", "omp"] {
        let label = crate::model::tool_label(id).to_string();
        if !cfg.harness_enabled(id) {
            continue;
        }
        match detect_root(id, cfg) {
            Some(path) if fingerprint_ok(id, &path) => out.push(HarnessRoot {
                id: id.into(),
                label,
                path,
                detected: true,
            }),
            Some(path) => out.push(HarnessRoot {
                id: id.into(),
                label,
                path,
                detected: false,
            }),
            None => out.push(HarnessRoot {
                id: id.into(),
                label,
                path: builtin_candidates(id)
                    .into_iter()
                    .next()
                    .unwrap_or_default(),
                detected: false,
            }),
        }
    }
    out
}
