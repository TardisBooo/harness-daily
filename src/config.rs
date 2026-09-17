use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_tz")]
    pub timezone: String,
    pub output_dir: PathBuf,
    #[serde(default = "default_time")]
    pub report_time: String,
    #[serde(default = "default_backfill")]
    pub backfill_days: u32,
    #[serde(default)]
    pub extra_roots: Vec<PathBuf>,
    #[serde(default)]
    pub writer: WriterConfig,
    #[serde(default)]
    pub report: ReportConfig,
    #[serde(default)]
    pub harnesses: HarnessToggles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_bin")]
    pub bin: String,
    #[serde(default = "default_args")]
    pub args_extra: Vec<String>,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            bin: default_bin(),
            args_extra: default_args(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    #[serde(default = "default_true")]
    pub include_audit_log: bool,
    #[serde(default = "default_true")]
    pub include_stats: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_audit_log: true,
            include_stats: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarnessToggles {
    pub codex: Option<HarnessOverride>,
    pub claude: Option<HarnessOverride>,
    pub grok: Option<HarnessOverride>,
    pub pi: Option<HarnessOverride>,
    pub omp: Option<HarnessOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarnessOverride {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub data_dir: Option<PathBuf>,
}

fn default_tz() -> String {
    "Asia/Shanghai".into()
}
fn default_time() -> String {
    "08:00".into()
}
fn default_backfill() -> u32 {
    7
}
fn default_host() -> String {
    "grok".into()
}
fn default_bin() -> String {
    "grok".into()
}
fn default_args() -> Vec<String> {
    vec!["--always-approve".into()]
}
fn default_true() -> bool {
    true
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("harness-daily")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            anyhow::bail!("未找到配置文件 {}，请先运行 harness-daily init", path.display());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("读取 {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("解析 {}", path.display()))
    }

    pub fn load_or_default(output_dir: PathBuf) -> Self {
        Self::load().unwrap_or_else(|_| Self {
            timezone: default_tz(),
            output_dir,
            report_time: default_time(),
            backfill_days: default_backfill(),
            extra_roots: vec![],
            writer: WriterConfig::default(),
            report: ReportConfig::default(),
            harnesses: HarnessToggles::default(),
        })
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(&path, text)?;
        Ok(path)
    }

    pub fn harness_enabled(&self, id: &str) -> bool {
        let ov = match id {
            "codex" => self.harnesses.codex.as_ref(),
            "claude" => self.harnesses.claude.as_ref(),
            "grok" => self.harnesses.grok.as_ref(),
            "pi" => self.harnesses.pi.as_ref(),
            "omp" => self.harnesses.omp.as_ref(),
            _ => None,
        };
        ov.map(|o| o.enabled).unwrap_or(true)
    }

    pub fn harness_override_dir(&self, id: &str) -> Option<PathBuf> {
        let ov = match id {
            "codex" => self.harnesses.codex.as_ref(),
            "claude" => self.harnesses.claude.as_ref(),
            "grok" => self.harnesses.grok.as_ref(),
            "pi" => self.harnesses.pi.as_ref(),
            "omp" => self.harnesses.omp.as_ref(),
            _ => None,
        };
        ov.and_then(|o| o.data_dir.clone()).filter(|p| !p.as_os_str().is_empty())
    }
}

pub fn default_output_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"D:\Me\工作日志")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("work-logs")
    }
}

pub fn ensure_dir(p: &Path) -> Result<()> {
    fs::create_dir_all(p).with_context(|| format!("创建目录 {}", p.display()))
}
