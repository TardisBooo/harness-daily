use anyhow::Result;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use regex::Regex;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static NANO_RE: OnceLock<Regex> = OnceLock::new();
static CMD_RE: OnceLock<Regex> = OnceLock::new();
static IMAGE_RE: OnceLock<Regex> = OnceLock::new();
static SMOKE_RE: OnceLock<Regex> = OnceLock::new();
static ACK_RE: OnceLock<Regex> = OnceLock::new();
static LEADING_RE: OnceLock<Regex> = OnceLock::new();
static HANDOFF_RE: OnceLock<Regex> = OnceLock::new();

const SKIP_PREFIXES: &[&str] = &[
    "<",
    "Caveat:",
    "[Request interrupted",
    "<user_instructions>",
    "Base directory for this skill",
    "===",
    "This session is being continued",
];

const NOISE: &[&str] = &[
    "继续",
    "continue",
    "ok",
    "okay",
    "好",
    "是",
    "对",
    "yes",
    "no",
    "next",
    "go",
    "说中文",
    "已经关闭wps了",
    "开始修复",
];

pub fn parse_tz(name: &str) -> Result<Tz> {
    name.parse::<Tz>()
        .map_err(|e| anyhow::anyhow!("未知时区 {name}: {e}"))
}

pub fn parse_iso(ts: &str) -> Option<DateTime<Utc>> {
    let nano = NANO_RE.get_or_init(|| Regex::new(r"\.(\d{7,})").unwrap());
    let mut s = ts.trim().to_string();
    if let Some(c) = nano.captures(&s) {
        let frac = &c[1];
        let six = &frac[..6.min(frac.len())];
        s = nano.replace(&s, format!(".{six}")).into_owned();
    }
    if s.ends_with('Z') {
        s = s.trim_end_matches('Z').to_string() + "+00:00";
    }
    DateTime::parse_from_rfc3339(&s)
        .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f%:z"))
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|n| Utc.from_utc_datetime(&n))
        })
}

pub fn day_bounds(date: NaiveDate, tz: Tz) -> (DateTime<Utc>, DateTime<Utc>) {
    let start_local = date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .single()
        .expect("local midnight");
    let end_local = (date + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .single()
        .expect("next midnight");
    (
        start_local.with_timezone(&Utc),
        end_local.with_timezone(&Utc),
    )
}

pub fn in_day(ts: DateTime<Utc>, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
    ts >= start && ts < end
}

pub fn clean_prompt(text: &str) -> String {
    let cmd = CMD_RE.get_or_init(|| {
        Regex::new(
            r"(?s)<command-message>(.*?)</command-message>.*?<command-args>(.*?)</command-args>",
        )
        .unwrap()
    });
    let image = IMAGE_RE.get_or_init(|| Regex::new(r"\[Image #\d+\]\s*").unwrap());
    let leading = LEADING_RE.get_or_init(|| Regex::new(r"(?i)^(继续|continue)[。.\s]*").unwrap());
    let handoff = HANDOFF_RE.get_or_init(|| {
        Regex::new(r"(?i)\[MOBIUS_HANDOFF[^\]]*\]\s*(MOBIUS HANDOFF:\s*)?").unwrap()
    });
    let mut t = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some(c) = cmd.captures(&t) {
        t = format!("/{} {}", c[1].trim(), c[2].trim())
            .trim()
            .to_string();
    }
    t = image.replace_all(&t, "").into_owned();
    t = handoff.replace_all(&t, "").into_owned();
    t = leading.replace(&t, "").into_owned();
    t.trim_matches(|c: char| " ，、；;:-".contains(c))
        .trim()
        .to_string()
}

pub fn worth_keeping(text: &str) -> bool {
    text.chars().count() >= 2 && !SKIP_PREFIXES.iter().any(|p| text.starts_with(p))
}

pub fn is_filler(text: &str) -> bool {
    let t = text
        .trim()
        .trim_end_matches(|c: char| "。，,.!！?？ ".contains(c))
        .to_lowercase();
    if NOISE.contains(&t.as_str()) || t.chars().count() < 4 {
        return true;
    }
    let smoke = SMOKE_RE
        .get_or_init(|| Regex::new(r"(?i)^(reply with|respond with|say |回复[:：]?)").unwrap());
    if smoke.is_match(text.trim()) {
        return true;
    }
    let ack = ACK_RE.get_or_init(|| {
        Regex::new(r"^(同意|好的|目前看起来|可以开始|没什么问题|已经关闭|开始修复|说中文)").unwrap()
    });
    ack.is_match(text.trim()) && text.trim().chars().count() < 40
}

pub fn truncate(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= n {
        text.to_string()
    } else {
        chars[..n.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}

pub fn short_project(path: &str) -> String {
    let p = path.trim_end_matches(['\\', '/']);
    let name = p.replace('/', "\\");
    let last = name.rsplit('\\').next().unwrap_or(p).to_string();
    let lower = name.to_lowercase();
    if lower.ends_with("users\\msi-nb") || lower.ends_with("users/msi-nb") {
        return "本机工具配置".into();
    }
    last
}

pub fn weekday_zh(date: NaiveDate) -> &'static str {
    match date.format("%u").to_string().as_str() {
        "1" => "周一",
        "2" => "周二",
        "3" => "周三",
        "4" => "周四",
        "5" => "周五",
        "6" => "周六",
        _ => "周日",
    }
}

pub fn iter_jsonl(path: &Path) -> impl Iterator<Item = Value> {
    let file = File::open(path).ok();
    let lines = file.map(|f| BufReader::new(f).lines());
    lines.into_iter().flatten().flatten().filter_map(|line| {
        let line = line.trim().to_string();
        if line.is_empty() {
            None
        } else {
            serde_json::from_str(&line).ok()
        }
    })
}

pub fn glob_pat(root: &Path, rel: &str) -> String {
    let mut s = root.to_string_lossy().replace('\\', "/");
    if s.ends_with('/') {
        s.pop();
    }
    format!("{s}/{rel}")
}

pub fn resolve_path(p: &Path) -> PathBuf {
    dunce::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

pub fn mtime_utc(path: &Path) -> Option<DateTime<Utc>> {
    path.metadata()
        .ok()?
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
}

pub fn decode_claude_dir(name: &str) -> String {
    let re = Regex::new(r"^([A-Za-z])--(.*)$").unwrap();
    if let Some(c) = re.captures(name) {
        format!("{}:\\{}", &c[1], c[2].replace('-', "\\"))
    } else {
        name.to_string()
    }
}

pub fn decode_pi_dir(name: &str) -> String {
    let s = name.trim_matches('-');
    let re = Regex::new(r"^([A-Za-z])-{1,2}").unwrap();
    let s = re.replace(s, |c: &regex::Captures| format!("{}:\\", &c[1]));
    s.replace('-', "\\")
}

pub fn json_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(|x| x.as_str())
}

#[allow(dead_code)]
pub fn json_get<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = v;
    for part in path.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn nano_iso_parses() {
        let ts = parse_iso("2026-09-02T17:37:29.591899500Z").unwrap();
        assert_eq!(ts.format("%Y-%m-%d").to_string(), "2026-09-02");
    }

    #[test]
    fn filler_detects_continue_and_smoke() {
        assert!(is_filler("continue"));
        assert!(is_filler("Reply with GLM53_1_OK only."));
        assert!(!is_filler("修复数据下载速率问题"));
    }

    #[test]
    fn shanghai_day_bounds() {
        let tz = parse_tz("Asia/Shanghai").unwrap();
        let d = NaiveDate::from_ymd_opt(2026, 9, 15).unwrap();
        let (s, e) = day_bounds(d, tz);
        assert_eq!(s.format("%H:%M").to_string(), "16:00"); // 00:00 CST = prev 16:00 UTC
        assert!(e > s);
    }
}
