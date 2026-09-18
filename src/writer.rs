use crate::config::Config;
use crate::hosts::{self, Host};
use crate::model::{CollectPayload, LlmReport};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn write_report(cfg: &Config, payload: &CollectPayload, work_dir: &Path) -> Result<LlmReport> {
    fs::create_dir_all(work_dir)?;
    let collect_path = work_dir.join("collect.json");
    let prompt_path = work_dir.join("writer-prompt.md");
    fs::write(&collect_path, serde_json::to_string_pretty(payload)?)?;
    fs::write(&prompt_path, build_prompt(payload))?;

    let (host, bin) = hosts::resolve_host(cfg)?;
    let prompt = fs::read_to_string(&prompt_path)?;
    let output = run_host(
        host,
        &bin,
        &cfg.writer.args_extra,
        &prompt,
        &prompt_path,
        work_dir,
    )?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::write(
        work_dir.join("writer-stdout.json"),
        output.stdout.as_slice(),
    );
    let _ = fs::write(work_dir.join("writer-stderr.txt"), output.stderr.as_slice());
    if !output.status.success() {
        anyhow::bail!(
            "{} 退出码 {:?} stderr={} stdout={}",
            host.label(),
            output.status.code(),
            truncate_log(&stderr, 800),
            truncate_log(&stdout, 800)
        );
    }
    let report = parse_llm_json(&stdout).or_else(|_| extract_embedded_json(&stdout))?;
    if report.projects.is_empty() && payload.projects.iter().any(|p| !p.prompts.is_empty()) {
        anyhow::bail!(
            "{} 返回了空 projects。stdout={}",
            host.label(),
            truncate_log(&stdout, 800)
        );
    }
    Ok(report)
}

fn run_host(
    host: Host,
    bin: &Path,
    extra: &[String],
    prompt: &str,
    prompt_path: &Path,
    work_dir: &Path,
) -> Result<std::process::Output> {
    let mut cmd = Command::new(bin);
    match host {
        Host::Grok => {
            // Scheduled writer used to hit max-turns: the prompt mentioned 日报,
            // Grok injected this plugin skill plus TinyFish MCP, then cancelled.
            // Deny all tools (MCP included). Deny wins over --always-approve.
            cmd.env("GROK_MEMORY", "0")
                .env("GROK_DISABLE_AUTOUPDATER", "1")
                .env("GROK_AGENT_DASHBOARD", "0")
                .arg("--prompt-file")
                .arg(prompt_path)
                .arg("--output-format")
                .arg("json")
                .arg("--max-turns")
                .arg("2")
                .arg("--no-subagents")
                .arg("--disable-web-search")
                .arg("--no-auto-update")
                .arg("--cwd")
                .arg(work_dir)
                .arg("--disallowed-tools")
                .arg("Agent")
                .arg("--deny")
                .arg("*");
            for e in extra {
                cmd.arg(e);
            }
            cmd.stdin(Stdio::null());
        }
        Host::Claude => {
            cmd.arg("-p")
                .arg("--output-format")
                .arg("json")
                .arg("--max-turns")
                .arg("4");
            for e in extra {
                cmd.arg(e);
            }
            cmd.arg(prompt);
            cmd.stdin(Stdio::null());
        }
        Host::Codex => {
            cmd.arg("exec").arg("--sandbox").arg("read-only").arg("-");
            for e in extra {
                cmd.arg(e);
            }
            cmd.stdin(Stdio::piped());
        }
    }
    if host == Host::Codex {
        let mut child = cmd
            .spawn()
            .with_context(|| format!("启动 {} 失败", bin.display()))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes())?;
        }
        child
            .wait_with_output()
            .with_context(|| format!("等待 {} 失败", bin.display()))
    } else {
        cmd.output()
            .with_context(|| format!("执行 {} 失败", bin.display()))
    }
}

fn slim_payload(payload: &CollectPayload) -> serde_json::Value {
    let projects: Vec<serde_json::Value> = payload
        .projects
        .iter()
        .filter(|p| !p.prompts.is_empty())
        .map(|p| {
            let prompts: Vec<String> = p
                .prompts
                .iter()
                .take(12)
                .map(|t| crate::util::truncate(t, 180))
                .collect();
            serde_json::json!({
                "name": p.name,
                "path": p.path,
                "prompts": prompts,
            })
        })
        .collect();
    serde_json::json!({
        "date": payload.date,
        "projects": projects,
    })
}

fn build_prompt(payload: &CollectPayload) -> String {
    let body = serde_json::to_string_pretty(&slim_payload(payload)).unwrap_or_else(|_| "{}".into());
    format!(
        r#"根据下面采集 JSON，用中文归纳各项目当天做了什么。不要编造未出现的工作。不要读文件、不要改文件、不要调用工具。

硬性要求：
1. 按项目组织工作内容，不要写工具名、时间、轮数。
2. 同一项目下不同会话的提问必须合并成几条完整工作说明。
3. 连通性测试（Reply with XXX_OK / 只回复：XXX）不要当正式工作。
4. 只输出一个 JSON 对象（不要 Markdown、不要代码围栏），结构如下：
{{
  "projects": [ {{ "name": "项目短名", "path": "完整路径", "items": ["工作说明1", "工作说明2"] }} ],
  "issues": [ {{ "project": "项目短名", "item": "待跟进问题" }} ]
}}
issues 若无问题给空数组；每个有提问的项目都必须出现在 projects 里。

采集数据：
{body}
"#
    )
}

fn parse_llm_json(stdout: &str) -> Result<LlmReport> {
    let trimmed = stdout.trim();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        for key in ["result", "text", "message", "content"] {
            if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
                if let Ok(r) = extract_embedded_json(s) {
                    if !r.projects.is_empty() {
                        return Ok(r);
                    }
                }
            }
        }
        if v.get("projects").is_some() {
            return Ok(serde_json::from_value(v)?);
        }
    }
    if let Ok(v) = serde_json::from_str::<LlmReport>(trimmed) {
        if !v.projects.is_empty() || trimmed.contains("\"projects\"") {
            return Ok(v);
        }
    }
    extract_embedded_json(trimmed)
}

fn extract_embedded_json(s: &str) -> Result<LlmReport> {
    let s = s.trim();
    let start = s.find('{').context("模型输出中没有 JSON 对象")?;
    let end = s.rfind('}').context("模型输出 JSON 不完整")?;
    let slice = &s[start..=end];
    serde_json::from_str(slice)
        .with_context(|| format!("无法解析 JSON：{}", crate::util::truncate(slice, 400)))
}

fn truncate_log(s: &str, n: usize) -> String {
    crate::util::truncate(s, n)
}

pub fn save_collect_snapshot(output_dir: &Path, payload: &CollectPayload) -> Result<()> {
    let p = output_dir.join(".harness-daily-last-collect.json");
    let mut f = fs::File::create(&p)?;
    f.write_all(serde_json::to_string_pretty(payload)?.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_json() {
        let s = r#"{"projects":[{"name":"MyDesk","path":"E:\\x","items":["做了 A"]}],"issues":[]}"#;
        let r = parse_llm_json(s).unwrap();
        assert_eq!(r.projects[0].name, "MyDesk");
        assert_eq!(r.projects[0].items[0], "做了 A");
    }

    #[test]
    fn parse_wrapped_result() {
        let inner = r#"{"projects":[{"name":"p","items":["x"]}],"issues":[]}"#;
        let s = serde_json::json!({"type":"result","result": inner}).to_string();
        let r = parse_llm_json(&s).unwrap();
        assert_eq!(r.projects[0].name, "p");
    }

    #[test]
    fn parse_text_field() {
        let inner =
            r#"{ "projects": [ { "name": "MyDesk", "items": ["做了 A"] } ], "issues": [] }"#;
        let s = serde_json::json!({"text": inner, "stopReason": "end_turn"}).to_string();
        let r = parse_llm_json(&s).unwrap();
        assert_eq!(r.projects[0].name, "MyDesk");
    }

    #[test]
    fn writer_prompt_does_not_mention_daily_report_skill_triggers() {
        let payload = CollectPayload {
            date: "2026-09-17".into(),
            timezone: "Asia/Shanghai".into(),
            projects: vec![],
            stats: vec![],
            notes: vec![],
            audit: vec![],
        };
        let p = build_prompt(&payload);
        for needle in ["日报", "工作日志", "harness-daily"] {
            assert!(
                !p.contains(needle),
                "scheduled writer prompt must not inject the plugin skill; found {needle}"
            );
        }
    }
}
