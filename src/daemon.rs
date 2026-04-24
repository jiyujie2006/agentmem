use crate::config::Config;
use crate::extractor::watcher::{LogWatcher, WatchEvent};
use crate::extractor::{extract_from_file, scan_directory};
use crate::storage::sqlite_store::SqliteStore;
use anyhow::Result;
use colored::Colorize;
use tokio::time::{interval, Duration};

/// 守护进程主循环
pub async fn run_daemon(config: &Config, store: &SqliteStore) -> Result<()> {
    println!("{}", "Starting AgentMem daemon...".green());

    let mut watcher = LogWatcher::new()?;
    watcher.register_adapters()?;

    // 添加用户自定义路径
    for path in &config.extra_watch_paths {
        watcher.add_path(path.clone())?;
    }

    let watched = watcher.watched_paths();
    if watched.is_empty() {
        println!("{}", "⚠ No agent log directories found.".yellow());
        println!("{}", "  Claude Code: ~/.claude/".dimmed());
        println!("{}", "  Use 'agentmem extract <path>' for manual import.".dimmed());
    } else {
        println!("{} Watching {} path(s):", "✓".green(), watched.len());
        for path in watched {
            println!("  • {}", path.display().to_string().dimmed());
        }
    }

    println!("\n{} Press Ctrl+C to stop\n", "→".dimmed());

    // 启动时先执行一次全量扫描
    let adapters = crate::adapter::get_all_adapters();
    for adapter in adapters {
        for log_path in adapter.log_paths() {
            if log_path.exists() {
                if log_path.is_file() {
                    if let Ok(memories) = extract_from_file(&log_path, Some(adapter.name())) {
                        if !memories.is_empty() {
                            let count = memories.len();
                            store.add_batch(memories)?;
                            println!("{} Extracted {} memories from {}",
                                "✓".green(),
                                count.to_string().bold(),
                                log_path.display().to_string().dimmed()
                            );
                        }
                    }
                } else if log_path.is_dir() {
                    if let Ok(memories) = scan_directory(&log_path, false) {
                        if !memories.is_empty() {
                            let count = memories.len();
                            store.add_batch(memories)?;
                            println!("{} Extracted {} memories from {}",
                                "✓".green(),
                                count.to_string().bold(),
                                log_path.display().to_string().dimmed()
                            );
                        }
                    }
                }
            }
        }
    }

    // 主循环：等待文件系统事件
    let mut poll_tick = interval(Duration::from_secs(config.poll_interval_secs));

    loop {
        tokio::select! {
            Some(event) = watcher.next_event() => {
                handle_watch_event(event, store).await?;
            }
            _ = poll_tick.tick() => {
                // 周期性打印活跃状态（可选）
                let count = store.count()?;
                println!("{} Daemon active — {} memories stored",
                    "•".dimmed(),
                    count.to_string().dimmed()
                );
            }
        }
    }
}

async fn handle_watch_event(event: WatchEvent, store: &SqliteStore) -> Result<()> {
    // 只处理 JSON 和日志文件
    let ext = event
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext != "json" && ext != "log" && ext != "md" {
        return Ok(());
    }

    // 简单防抖：等待文件写入完成
    tokio::time::sleep(Duration::from_millis(500)).await;

    match extract_from_file(&event.path, None) {
        Ok(memories) => {
            if !memories.is_empty() {
                let count = memories.len();
                store.add_batch(memories)?;
                println!(
                    "{} {} new memory(s) from {}",
                    "✓".green(),
                    count.to_string().bold(),
                    event.path.display().to_string().dimmed()
                );
            }
        }
        Err(e) => {
            if ext == "json" {
                eprintln!("{} Failed to parse {}: {}", "✗".red(), event.path.display(), e);
            }
        }
    }

    Ok(())
}
