mod collect;
mod config;
mod discovery;
mod hosts;
mod model;
mod render;
mod schedule;
mod util;
mod writer;

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use config::{ensure_dir, Config};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "harness-daily",
    version,
    about = "Scan AI coding harness sessions and write a daily report via Grok Build"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 写入默认配置（本机宿主为 Grok Build）
    Init {
        /// grok | claude | codex | auto（探测已安装并已登录的 CLI）
        #[arg(long, default_value = "auto")]
        host: String,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value = "08:00")]
        time: String,
    },
    /// 列出探测到的 harness 数据目录
    Scan,
    /// 自检路径、grok 二进制与输出目录
    Doctor,
    /// 生成日报
    Report {
        #[arg(long)]
        date: Option<String>,
        #[arg(long, default_value_t = 0)]
        backfill: u32,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        auto: bool,
        /// 只采集，不调用 grok
        #[arg(long)]
        dry_collect: bool,
    },
    /// 系统定时任务
    Schedule {
        #[command(subcommand)]
        action: ScheduleCmd,
    },
}

#[derive(Subcommand)]
enum ScheduleCmd {
    Install {
        #[arg(long)]
        time: Option<String>,
    },
    Remove,
    Status,
    Run,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { host, out, time } => cmd_init(&host, out, &time),
        Cmd::Scan => cmd_scan(),
        Cmd::Doctor => cmd_doctor(),
        Cmd::Report {
            date,
            backfill,
            out,
            auto,
            dry_collect,
        } => cmd_report(date, backfill, out, auto, dry_collect),
        Cmd::Schedule { action } => match action {
            ScheduleCmd::Install { time } => {
                let mut cfg = Config::load()?;
                if let Some(t) = time {
                    cfg.report_time = t;
                    cfg.save()?;
                }
                schedule::install(&cfg)
            }
            ScheduleCmd::Remove => schedule::remove(),
            ScheduleCmd::Status => schedule::status(),
            ScheduleCmd::Run => schedule::run_now(),
        },
    }
}

fn cmd_init(host: &str, out: Option<PathBuf>, time: &str) -> Result<()> {
    let output_dir = out.unwrap_or_else(config::default_output_dir);
    ensure_dir(&output_dir)?;
    let mut cfg = Config::load_or_default(output_dir.clone());
    cfg.output_dir = output_dir;
    cfg.report_time = time.to_string();
    cfg.writer.host = host.to_string();
    let (resolved, bin) = hosts::resolve_host(&cfg)?;
    cfg.writer.host = resolved.id().into();
    cfg.writer.bin = bin.display().to_string();
    if cfg.writer.args_extra.is_empty() {
        cfg.writer.args_extra = hosts::default_extra_args(resolved);
    }
    if let Ok(installed) = install_self() {
        println!("已安装二进制 {}", installed.display());
    }
    let path = cfg.save()?;
    println!("已写入 {}", path.display());
    println!("输出目录 {}", cfg.output_dir.display());
    println!("写正文宿主: {} ({})", resolved.label(), cfg.writer.bin);
    cmd_scan()?;
    Ok(())
}

fn install_self() -> Result<PathBuf> {
    let src = std::env::current_exe()?;
    let dest_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("harness-daily");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(if cfg!(windows) {
        "harness-daily.exe"
    } else {
        "harness-daily"
    });
    if src != dest {
        fs::copy(&src, &dest)
            .with_context(|| format!("复制 {} → {}", src.display(), dest.display()))?;
    }
    Ok(dest)
}

fn cmd_scan() -> Result<()> {
    let cfg =
        Config::load().unwrap_or_else(|_| Config::load_or_default(config::default_output_dir()));
    let roots = discovery::scan_roots(&cfg);
    println!("{:<8} {:<12} path", "id", "status");
    for r in roots {
        let st = if r.detected { "ok" } else { "missing" };
        println!("{:<8} {:<12} {}", r.id, st, r.path.display());
    }
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    let cfg = Config::load()?;
    println!("config     {}", Config::config_path().display());
    println!("output     {}", cfg.output_dir.display());
    println!("timezone   {}", cfg.timezone);
    println!("report_at  {}", cfg.report_time);
    match hosts::resolve_host(&cfg) {
        Ok((host, p)) => {
            println!("writer     {} ({})", host.label(), p.display());
            let out = Command::new(&p).arg("--version").output();
            match out {
                Ok(o) => {
                    let v = String::from_utf8_lossy(&o.stdout);
                    let v = if v.trim().is_empty() {
                        String::from_utf8_lossy(&o.stderr).into_owned()
                    } else {
                        v.into_owned()
                    };
                    print!("bin ver    {v}");
                }
                Err(e) => println!("bin run    失败: {e}"),
            }
        }
        Err(e) => println!("writer     未找到: {e}"),
    }
    cmd_scan()?;
    if !cfg.output_dir.exists() {
        println!("warning: 输出目录不存在，init 时会创建");
    }
    Ok(())
}

fn cmd_report(
    date: Option<String>,
    backfill: u32,
    out: Option<PathBuf>,
    auto: bool,
    dry_collect: bool,
) -> Result<()> {
    let mut cfg = Config::load()?;
    if let Some(o) = out {
        cfg.output_dir = o;
    }
    ensure_dir(&cfg.output_dir)?;
    let tz = util::parse_tz(&cfg.timezone)?;
    let today = Local::now().with_timezone(&tz).date_naive();
    let dates: Vec<NaiveDate> = if let Some(d) = date {
        vec![NaiveDate::parse_from_str(&d, "%Y-%m-%d").context("日期格式应为 YYYY-MM-DD")?]
    } else if backfill > 0 {
        (1..=backfill)
            .rev()
            .map(|i| today - chrono::Duration::days(i as i64))
            .collect()
    } else {
        vec![today - chrono::Duration::days(1)]
    };

    for d in dates {
        generate_one(&cfg, d, auto, dry_collect)?;
    }
    Ok(())
}

fn generate_one(cfg: &Config, date: NaiveDate, auto: bool, dry_collect: bool) -> Result<()> {
    let out_file = cfg
        .output_dir
        .join(format!("日报-{}.md", date.format("%Y-%m-%d")));
    if auto && out_file.exists() {
        println!("[skip] {} 已存在", out_file.display());
        return Ok(());
    }
    println!("[collect] {}", date);
    let payload = collect::collect(cfg, date)?;
    for s in &payload.stats {
        println!(
            "[collect]   {:<12} 会话 {:>3}  提问 {:>3}",
            s.label, s.sessions, s.prompts
        );
    }
    writer::save_collect_snapshot(&cfg.output_dir, &payload)?;
    if dry_collect {
        let p = cfg
            .output_dir
            .join(format!("collect-{}.json", date.format("%Y-%m-%d")));
        fs::write(&p, serde_json::to_string_pretty(&payload)?)?;
        println!("[dry] 已写入 {}", p.display());
        return Ok(());
    }
    let work = cfg.output_dir.join(".harness-daily-work");
    let llm = writer::write_report(cfg, &payload, &work).context("调用 agent CLI 写正文失败")?;
    let md = render::render(
        date,
        &payload,
        &llm,
        cfg.report.include_audit_log,
        cfg.report.include_stats,
    );
    fs::write(&out_file, md)?;
    println!("[done] {}", out_file.display());
    Ok(())
}
