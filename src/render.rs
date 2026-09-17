use crate::model::{CollectPayload, LlmProject, LlmReport};
use crate::util::{truncate, weekday_zh};
use chrono::{Local, NaiveDate};

fn fallback_projects(payload: &CollectPayload) -> Vec<LlmProject> {
    payload
        .projects
        .iter()
        .filter(|p| !p.prompts.is_empty())
        .map(|p| LlmProject {
            name: p.name.clone(),
            path: Some(p.path.clone()),
            items: p
                .prompts
                .iter()
                .take(3)
                .map(|t| truncate(t, 120))
                .collect(),
        })
        .collect()
}

pub fn render(
    date: NaiveDate,
    payload: &CollectPayload,
    llm: &LlmReport,
    include_audit: bool,
    include_stats: bool,
) -> String {
    let wd = weekday_zh(date);
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut lines = Vec::new();
    lines.push(format!("# 工作日报 · {}（{}）", date.format("%Y-%m-%d"), wd));
    lines.push(String::new());
    lines.push(format!(
        "> 统计口径：{} 00:00 – 24:00（{}），来源：本机 AI coding harness 会话。",
        date.format("%Y-%m-%d"),
        payload.timezone
    ));
    lines.push(format!("> 生成时间：{now}（harness-daily + Grok Build）"));
    lines.push(String::new());

    lines.push("## 一、今日工作总结".into());
    lines.push(String::new());
    let llm_projects = if llm.projects.is_empty() {
        fallback_projects(payload)
    } else {
        llm.projects.clone()
    };
    if llm_projects.is_empty() {
        lines.push("当日无实质工作记录。".into());
        lines.push(String::new());
    } else {
        for p in &llm_projects {
            let name = if p.name.is_empty() {
                p.path.clone().unwrap_or_else(|| "未命名项目".into())
            } else {
                p.name.clone()
            };
            lines.push(format!("**{name}**"));
            if p.items.is_empty() {
                lines.push("- 持续推进该项目既有任务".into());
            } else {
                for item in &p.items {
                    lines.push(format!("- {item}"));
                }
            }
            lines.push(String::new());
        }
    }

    lines.push("## 二、工作明细".into());
    for p in &llm_projects {
        let name = if p.name.is_empty() {
            "未命名项目".into()
        } else {
            p.name.clone()
        };
        let path = p.path.clone().unwrap_or_default();
        let local = payload.projects.iter().find(|x| {
            x.path == path || x.name == name || p.path.as_ref() == Some(&x.path)
        });
        lines.push(String::new());
        lines.push(format!("### {name}"));
        lines.push(String::new());
        if !path.is_empty() {
            lines.push(format!("- 项目路径：`{path}`"));
        } else if let Some(l) = local {
            lines.push(format!("- 项目路径：`{}`", l.path));
        }
        if let Some(l) = local {
            lines.push(format!("- 工作时段：{}–{}", l.start, l.end));
        }
        lines.push("- 完成内容：".into());
        if p.items.is_empty() {
            lines.push("  1. 持续推进该项目既有任务".into());
        } else {
            for (i, item) in p.items.iter().enumerate() {
                lines.push(format!("  {}. {item}", i + 1));
            }
        }
    }
    lines.push(String::new());

    lines.push("## 三、问题与待办".into());
    lines.push(String::new());
    if llm.issues.is_empty() {
        lines.push("当日提问中未识别到明确的待修复问题。".into());
    } else {
        for i in &llm.issues {
            lines.push(format!("- **{}**：{}", i.project, i.item));
        }
    }
    lines.push(String::new());

    if include_stats {
        lines.push("## 四、工作量统计".into());
        lines.push(String::new());
        lines.push("| 工具 | 会话数 | 提问轮数 |".into());
        lines.push("|---|---|---|".into());
        for s in &payload.stats {
            lines.push(format!("| {} | {} | {} |", s.label, s.sessions, s.prompts));
        }
        for n in &payload.notes {
            lines.push(String::new());
            lines.push(format!("注：{n}"));
        }
        lines.push(String::new());
    }

    if include_audit {
        lines.push("## 附录：操作审计日志".into());
        lines.push(String::new());
        lines.push("以下为当日全部提问与会话事件的原始记录，按时间排序，供追溯核查。".into());
        lines.push(String::new());
        for a in &payload.audit {
            if a.kind == "session" {
                let title = if a.text.is_empty() {
                    String::new()
                } else {
                    format!("「{}」", a.text)
                };
                lines.push(format!(
                    "- {} [{}] {} 开始会话{title}",
                    a.time, a.tool, a.project
                ));
            } else {
                lines.push(format!(
                    "- {} [{}] {}：{}",
                    a.time, a.tool, a.project, a.text
                ));
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}
