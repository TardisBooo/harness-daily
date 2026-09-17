use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const TOOLS: [&str; 5] = ["codex", "claude", "grok", "pi", "omp"];

pub fn tool_label(id: &str) -> &'static str {
    match id {
        "codex" => "Codex",
        "claude" => "Claude Code",
        "grok" => "Grok Build",
        "pi" => "Pi",
        "omp" => "OMP",
        _ => "unknown",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub tool: String,
    pub project: String,
    pub time: DateTime<Utc>,
    pub text: Option<String>,
    pub kind: String, // session | prompt | note
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBundle {
    pub name: String,
    pub path: String,
    pub tools: Vec<String>,
    pub start: String,
    pub end: String,
    pub rounds: usize,
    pub sessions: usize,
    pub prompts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStat {
    pub tool: String,
    pub label: String,
    pub sessions: usize,
    pub prompts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLine {
    pub time: String,
    pub tool: String,
    pub project: String,
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectPayload {
    pub date: String,
    pub timezone: String,
    pub projects: Vec<ProjectBundle>,
    pub stats: Vec<ToolStat>,
    pub notes: Vec<String>,
    pub audit: Vec<AuditLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmProject {
    pub name: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmIssue {
    pub project: String,
    pub item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmReport {
    #[serde(default)]
    pub projects: Vec<LlmProject>,
    #[serde(default)]
    pub issues: Vec<LlmIssue>,
}

#[derive(Debug, Clone)]
pub struct HarnessRoot {
    pub id: String,
    #[allow(dead_code)]
    pub label: String,
    pub path: std::path::PathBuf,
    pub detected: bool,
}
