use crate::config::Config;
use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
#[cfg(any(windows, target_os = "macos"))]
use std::process::Command;

#[cfg(windows)]
const TASK_NAME: &str = "harness-daily";

fn exe_path() -> Result<PathBuf> {
    let installed = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("harness-daily")
        .join(if cfg!(windows) {
            "harness-daily.exe"
        } else {
            "harness-daily"
        });
    if installed.exists() {
        return Ok(installed);
    }
    env::current_exe().context("无法定位 harness-daily 可执行文件")
}

pub fn install(cfg: &Config) -> Result<()> {
    let exe = exe_path()?;
    let time = cfg.report_time.clone();
    #[cfg(windows)]
    {
        let dest_dir = exe
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&dest_dir)?;
        let wrapper = dest_dir.join("run-report.cmd");
        let log = dest_dir.join("task.log");
        let wrapper_body = format!(
            "@echo off\r\necho ===== %DATE% %TIME% =====>>\"{log}\"\r\n\"{exe}\" report --auto --backfill {n} >>\"{log}\" 2>&1\r\n",
            log = log.display(),
            exe = exe.display(),
            n = cfg.backfill_days
        );
        std::fs::write(&wrapper, wrapper_body)?;
        let tr = format!("\"{}\"", wrapper.display());
        let status = Command::new("schtasks")
            .args([
                "/Create", "/TN", TASK_NAME, "/TR", &tr, "/SC", "DAILY", "/ST", &time, "/F",
            ])
            .status()
            .context("调用 schtasks 失败")?;
        if !status.success() {
            anyhow::bail!("schtasks /Create 失败");
        }
        // Default schtasks /Create stops on battery and does not catch up a missed 08:00.
        let ps = format!(
            "$t = Get-ScheduledTask -TaskName '{TASK_NAME}'; $t.Settings.DisallowStartIfOnBatteries = $false; $t.Settings.StopIfGoingOnBatteries = $false; $t.Settings.StartWhenAvailable = $true; $t.Settings.ExecutionTimeLimit = 'PT2H'; Set-ScheduledTask -InputObject $t | Out-Null"
        );
        let patched = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps])
            .status();
        if !matches!(patched, Ok(s) if s.success()) {
            eprintln!("warning: 未能关闭计划任务的「电池上停止」；笔记本未插电时 08:00 可能被跳过");
        }
        println!("已安装 Windows 计划任务 {TASK_NAME}（每天 {}）", time);
        println!("任务日志 {}", log.display());
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let plist_dir = dirs::home_dir()
            .unwrap()
            .join("Library")
            .join("LaunchAgents");
        std::fs::create_dir_all(&plist_dir)?;
        let plist = plist_dir.join("io.github.harness-daily.plist");
        let hour: u32 = time.split(':').next().unwrap_or("8").parse().unwrap_or(8);
        let minute: u32 = time.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>io.github.harness-daily</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
    <string>report</string>
    <string>--auto</string>
    <string>--backfill</string>
    <string>{}</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict><key>Hour</key><integer>{hour}</integer><key>Minute</key><integer>{minute}</integer></dict>
  <key>RunAtLoad</key><true/>
</dict></plist>
"#,
            exe.display(),
            cfg.backfill_days
        );
        std::fs::write(&plist, body)?;
        let _ = Command::new("launchctl")
            .args(["unload", &plist.display().to_string()])
            .status();
        Command::new("launchctl")
            .args(["load", &plist.display().to_string()])
            .status()?;
        println!("已安装 launchd {}", plist.display());
        Ok(())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let (h, m) = {
            let mut it = time.split(':');
            let h = it.next().unwrap_or("8");
            let m = it.next().unwrap_or("0");
            (h, m)
        };
        println!(
            "请将以下行加入 crontab（每天 {}:{} 触发）：\n{} {} * * * \"{}\" report --auto --backfill {}",
            h,
            m,
            m,
            h,
            exe.display(),
            cfg.backfill_days
        );
        Ok(())
    }
}

pub fn remove() -> Result<()> {
    #[cfg(windows)]
    {
        let status = Command::new("schtasks")
            .args(["/Delete", "/TN", TASK_NAME, "/F"])
            .status()?;
        if !status.success() {
            anyhow::bail!("删除计划任务失败（可能本来就不存在）");
        }
        println!("已删除计划任务 {TASK_NAME}");
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let plist = dirs::home_dir()
            .unwrap()
            .join("Library/LaunchAgents/io.github.harness-daily.plist");
        let _ = Command::new("launchctl")
            .args(["unload", &plist.display().to_string()])
            .status();
        let _ = std::fs::remove_file(&plist);
        println!("已移除 launchd 任务");
        Ok(())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        println!("请从 crontab 删除含 harness-daily 的行");
        Ok(())
    }
}

pub fn status() -> Result<()> {
    #[cfg(windows)]
    {
        let status = Command::new("schtasks")
            .args(["/Query", "/TN", TASK_NAME, "/V", "/FO", "LIST"])
            .status()?;
        if !status.success() {
            println!("计划任务 {TASK_NAME} 未安装");
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        println!("请用系统工具查看 harness-daily 定时任务");
        Ok(())
    }
}

pub fn run_now() -> Result<()> {
    #[cfg(windows)]
    {
        let status = Command::new("schtasks")
            .args(["/Run", "/TN", TASK_NAME])
            .status()?;
        if !status.success() {
            anyhow::bail!("schtasks /Run 失败");
        }
        println!("已触发计划任务 {TASK_NAME}");
        Ok(())
    }
    #[cfg(not(windows))]
    {
        anyhow::bail!("请直接运行 harness-daily report")
    }
}
