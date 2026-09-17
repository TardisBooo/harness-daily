use crate::config::Config;
use crate::discovery::scan_roots;
use crate::model::{Activity, AuditLine, CollectPayload, ProjectBundle, ToolStat, TOOLS};
use crate::util::{
    clean_prompt, day_bounds, decode_claude_dir, decode_pi_dir, glob_pat, in_day, is_filler,
    iter_jsonl, json_str, mtime_utc, parse_iso, parse_tz, short_project, truncate, worth_keeping,
};
use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use glob::glob;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn collect(cfg: &Config, date: NaiveDate) -> Result<CollectPayload> {
    let tz = parse_tz(&cfg.timezone)?;
    let (start, end) = day_bounds(date, tz);
    let roots = scan_roots(cfg);
    let mut acts = Vec::new();
    for r in &roots {
        if !r.detected {
            continue;
        }
        match r.id.as_str() {
            "codex" => acts.extend(collect_codex(&r.path, start, end)),
            "claude" => acts.extend(collect_claude(&r.path, start, end)),
            "grok" => acts.extend(collect_grok(&r.path, start, end)),
            "pi" | "omp" => acts.extend(collect_pi_like(&r.path, &r.id, start, end)),
            _ => {}
        }
    }
    Ok(bundle(date, tz, acts))
}

fn collect_codex(root: &Path, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Activity> {
    let mut acts = Vec::new();
    let mut sid_cwd: HashMap<String, String> = HashMap::new();
    let pat = glob_pat(&root.join("sessions"), "*/*/*/rollout-*.jsonl");
    if let Ok(paths) = glob(&pat) {
        for p in paths.flatten() {
            let Ok(f) = File::open(&p) else { continue };
            let mut first = String::new();
            let mut r = BufReader::new(f);
            if r.read_line(&mut first).is_err() {
                continue;
            }
            let Ok(obj) = serde_json::from_str::<Value>(first.trim()) else {
                continue;
            };
            if json_str(&obj, "type") != Some("session_meta") {
                continue;
            }
            let payload = obj.get("payload").cloned().unwrap_or(Value::Null);
            let sid = json_str(&payload, "id")
                .or_else(|| json_str(&payload, "session_id"))
                .unwrap_or("")
                .to_string();
            let cwd = json_str(&payload, "cwd")
                .unwrap_or("(未知项目)")
                .to_string();
            if !sid.is_empty() {
                sid_cwd.insert(sid.clone(), cwd.clone());
            }
            if let Some(ts) = json_str(&payload, "timestamp").and_then(parse_iso) {
                if in_day(ts, start, end) {
                    acts.push(Activity {
                        tool: "codex".into(),
                        project: cwd,
                        time: ts,
                        text: None,
                        kind: "session".into(),
                    });
                }
            }
        }
    }
    let history = root.join("history.jsonl");
    let mut recent: HashMap<(String, String), DateTime<Utc>> = HashMap::new();
    if history.exists() {
        for obj in iter_jsonl(&history) {
            let Some(raw) = obj
                .get("ts")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
            else {
                continue;
            };
            let ts = DateTime::<Utc>::from_timestamp(raw, 0).unwrap_or(Utc::now());
            if !in_day(ts, start, end) {
                continue;
            }
            let text = clean_prompt(json_str(&obj, "text").unwrap_or(""));
            if !worth_keeping(&text) {
                continue;
            }
            let sid = json_str(&obj, "session_id").unwrap_or("").to_string();
            let key = (sid.clone(), text.clone());
            if let Some(last) = recent.get(&key) {
                if ts - *last < Duration::minutes(2) {
                    continue;
                }
            }
            recent.insert(key, ts);
            let project = sid_cwd
                .get(&sid)
                .cloned()
                .unwrap_or_else(|| "(未定位项目)".into());
            acts.push(Activity {
                tool: "codex".into(),
                project,
                time: ts,
                text: Some(text),
                kind: "prompt".into(),
            });
        }
    }
    acts
}

fn collect_claude(root: &Path, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Activity> {
    let mut acts = Vec::new();
    let pat = glob_pat(&root.join("projects"), "*/*.jsonl");
    let Ok(paths) = glob(&pat) else {
        return acts;
    };
    for p in paths.flatten() {
        if let Some(mt) = mtime_utc(&p) {
            if mt < start {
                continue;
            }
        }
        let objs: Vec<Value> = iter_jsonl(&p).collect();
        let cwd = objs
            .iter()
            .find_map(|o| json_str(o, "cwd").map(|s| s.to_string()));
        let parent = p
            .parent()
            .and_then(|x| x.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let project = cwd.unwrap_or_else(|| decode_claude_dir(&parent));
        let mut seen: HashMap<String, DateTime<Utc>> = HashMap::new();
        let mut had_session = false;
        for obj in objs {
            let ts = json_str(&obj, "timestamp").and_then(parse_iso);
            let in_d = ts.map(|t| in_day(t, start, end)).unwrap_or(false);
            if in_d && !had_session {
                had_session = true;
                acts.push(Activity {
                    tool: "claude".into(),
                    project: project.clone(),
                    time: ts.unwrap(),
                    text: None,
                    kind: "session".into(),
                });
            }
            let t = json_str(&obj, "type").unwrap_or("");
            let mut text = None;
            if t == "queue-operation" && json_str(&obj, "operation") == Some("enqueue") && in_d {
                text = Some(clean_prompt(json_str(&obj, "content").unwrap_or("")));
            } else if t == "user" && in_d {
                if let Some(content) = obj
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    text = Some(clean_prompt(content));
                }
            }
            if let (Some(tx), Some(ts)) = (text, ts) {
                if worth_keeping(&tx) {
                    if let Some(last) = seen.get(&tx) {
                        if ts - *last < Duration::minutes(2) {
                            continue;
                        }
                    }
                    seen.insert(tx.clone(), ts);
                    acts.push(Activity {
                        tool: "claude".into(),
                        project: project.clone(),
                        time: ts,
                        text: Some(tx),
                        kind: "prompt".into(),
                    });
                }
            }
        }
    }
    acts
}

fn collect_grok(root: &Path, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Activity> {
    let mut acts = Vec::new();
    let pat = glob_pat(&root.join("sessions"), "*/prompt_history.jsonl");
    let Ok(paths) = glob(&pat) else {
        return acts;
    };
    for p in paths.flatten() {
        let encoded = p
            .parent()
            .and_then(|x| x.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let project = urlencoding::decode(&encoded)
            .map(|s| s.into_owned())
            .unwrap_or(encoded);
        let mut seen_sids = HashSet::new();
        for obj in iter_jsonl(&p) {
            let Some(ts) = json_str(&obj, "timestamp").and_then(parse_iso) else {
                continue;
            };
            if !in_day(ts, start, end) {
                continue;
            }
            if obj
                .get("is_bash")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }
            let sid = json_str(&obj, "session_id").unwrap_or("?").to_string();
            if seen_sids.insert(sid) {
                acts.push(Activity {
                    tool: "grok".into(),
                    project: project.clone(),
                    time: ts,
                    text: None,
                    kind: "session".into(),
                });
            }
            let text = clean_prompt(json_str(&obj, "prompt").unwrap_or(""));
            if worth_keeping(&text) {
                acts.push(Activity {
                    tool: "grok".into(),
                    project: project.clone(),
                    time: ts,
                    text: Some(text),
                    kind: "prompt".into(),
                });
            }
        }
    }
    acts
}

fn collect_pi_like(
    root: &Path,
    tool: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<Activity> {
    let mut acts = Vec::new();
    let sessions = root.join("agent").join("sessions");
    if !sessions.exists() {
        return acts;
    }
    let pat = glob_pat(&sessions, "*/*.jsonl");
    let Ok(paths) = glob(&pat) else {
        return acts;
    };
    for sf in paths.flatten() {
        if let Some(mt) = mtime_utc(&sf) {
            if mt < start {
                continue;
            }
        }
        let parent = sf
            .parent()
            .and_then(|x| x.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut cwd: Option<String> = None;
        let mut title: Option<String> = None;
        let mut had_session = false;
        let mut seen: HashMap<String, DateTime<Utc>> = HashMap::new();
        for obj in iter_jsonl(&sf) {
            let t = json_str(&obj, "type").unwrap_or("");
            let mut ts = json_str(&obj, "timestamp").and_then(parse_iso);
            if t == "session" {
                if let Some(c) = json_str(&obj, "cwd") {
                    cwd = Some(c.to_string());
                }
                if let Some(ti) = json_str(&obj, "title") {
                    title = Some(ti.to_string());
                }
                if ts.is_none() {
                    if let Some(name) = sf.file_name().and_then(|s| s.to_str()) {
                        if name.len() >= 19 {
                            let cand = format!(
                                "{}T{}:{}:{}+00:00",
                                &name[0..10],
                                &name[11..13],
                                &name[14..16],
                                &name[17..19]
                            );
                            ts = parse_iso(&cand);
                        }
                    }
                }
            } else if t == "title" {
                if let Some(ti) = json_str(&obj, "title") {
                    title = Some(ti.to_string());
                }
            }
            let project = cwd.clone().unwrap_or_else(|| decode_pi_dir(&parent));
            if let Some(ts) = ts {
                if in_day(ts, start, end) && !had_session {
                    had_session = true;
                    acts.push(Activity {
                        tool: tool.into(),
                        project: project.clone(),
                        time: ts,
                        text: title.clone(),
                        kind: "session".into(),
                    });
                }
            }
            if t != "message" {
                continue;
            }
            let Some(ts) = ts else { continue };
            if !in_day(ts, start, end) {
                continue;
            }
            let msg = obj.get("message").cloned().unwrap_or(Value::Null);
            if json_str(&msg, "role") != Some("user") {
                continue;
            }
            let content = msg.get("content");
            let mut texts = Vec::new();
            if let Some(s) = content.and_then(|c| c.as_str()) {
                texts.push(s.to_string());
            } else if let Some(arr) = content.and_then(|c| c.as_array()) {
                for c in arr {
                    if json_str(c, "type") == Some("text") {
                        if let Some(tx) = json_str(c, "text") {
                            texts.push(tx.to_string());
                        }
                    }
                }
            }
            let text = clean_prompt(&texts.join(" "));
            if worth_keeping(&text) {
                if let Some(last) = seen.get(&text) {
                    if ts - *last < Duration::minutes(2) {
                        continue;
                    }
                }
                seen.insert(text.clone(), ts);
                acts.push(Activity {
                    tool: tool.into(),
                    project,
                    time: ts,
                    text: Some(text),
                    kind: "prompt".into(),
                });
            }
        }
    }
    acts
}

fn bundle(date: NaiveDate, tz: Tz, acts: Vec<Activity>) -> CollectPayload {
    let mut by_proj: HashMap<String, Vec<Activity>> = HashMap::new();
    let mut notes = Vec::new();
    for a in &acts {
        if a.kind == "note" {
            if let Some(t) = &a.text {
                notes.push(t.clone());
            }
            continue;
        }
        by_proj
            .entry(a.project.clone())
            .or_default()
            .push(a.clone());
    }
    let mut projects: Vec<ProjectBundle> = by_proj
        .into_iter()
        .map(|(path, items)| {
            let mut tools: Vec<String> = items.iter().map(|a| a.tool.clone()).collect();
            tools.sort();
            tools.dedup();
            tools.sort_by_key(|t| TOOLS.iter().position(|x| x == t).unwrap_or(99));
            let times: Vec<_> = items.iter().map(|a| a.time).collect();
            let start = times.iter().min().copied().unwrap_or(Utc::now());
            let end = times.iter().max().copied().unwrap_or(start);
            let mut prompts: Vec<String> = items
                .iter()
                .filter(|a| a.kind == "prompt")
                .filter_map(|a| a.text.clone())
                .filter(|t| !is_filler(t))
                .collect();
            let mut seen = HashSet::new();
            prompts.retain(|t| seen.insert(t.clone()));
            ProjectBundle {
                name: short_project(&path),
                path: path.clone(),
                tools,
                start: start.with_timezone(&tz).format("%H:%M").to_string(),
                end: end.with_timezone(&tz).format("%H:%M").to_string(),
                rounds: items.iter().filter(|a| a.kind == "prompt").count(),
                sessions: items.iter().filter(|a| a.kind == "session").count(),
                prompts,
            }
        })
        .collect();
    projects.sort_by(|a, b| b.rounds.cmp(&a.rounds).then(a.start.cmp(&b.start)));

    let stats = TOOLS
        .iter()
        .map(|id| {
            let ta: Vec<_> = acts.iter().filter(|a| a.tool == *id).collect();
            ToolStat {
                tool: (*id).into(),
                label: crate::model::tool_label(id).into(),
                sessions: ta.iter().filter(|a| a.kind == "session").count(),
                prompts: ta.iter().filter(|a| a.kind == "prompt").count(),
            }
        })
        .collect();

    let mut audit: Vec<AuditLine> = acts
        .iter()
        .filter(|a| a.kind != "note")
        .map(|a| AuditLine {
            time: a.time.with_timezone(&tz).format("%H:%M").to_string(),
            tool: crate::model::tool_label(&a.tool).into(),
            project: a.project.clone(),
            kind: a.kind.clone(),
            text: a
                .text
                .as_deref()
                .map(|t| truncate(t, 160))
                .unwrap_or_default(),
        })
        .collect();
    audit.sort_by(|a, b| a.time.cmp(&b.time).then(a.tool.cmp(&b.tool)));

    CollectPayload {
        date: date.format("%Y-%m-%d").to_string(),
        timezone: tz.name().to_string(),
        projects,
        stats,
        notes,
        audit,
    }
}
