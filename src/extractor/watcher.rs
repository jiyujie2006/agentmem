use crate::adapter::get_all_adapters;
use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// 文件系统监控事件
#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub kind: String,
}

/// 监控所有已注册 Agent 的日志目录
pub struct LogWatcher {
    watcher: RecommendedWatcher,
    rx: mpsc::Receiver<WatchEvent>,
    watched_paths: Vec<PathBuf>,
}

impl LogWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    for path in event.paths {
                        let kind = format!("{:?}", event.kind);
                        let _ = tx.blocking_send(WatchEvent { path, kind });
                    }
                }
            },
            Config::default(),
        )?;

        Ok(Self {
            watcher,
            rx,
            watched_paths: Vec::new(),
        })
    }

    /// 注册所有可用的 Agent 日志路径
    pub fn register_adapters(&mut self) -> Result<()> {
        let adapters = get_all_adapters();

        for adapter in adapters {
            let paths = adapter.log_paths();
            for path in &paths {
                if path.exists() {
                    // 如果是文件，监控其父目录；如果是目录，直接监控
                    let watch_path = if path.is_file() {
                        path.parent().unwrap_or(path.as_ref()).to_path_buf()
                    } else {
                        path.clone()
                    };

                    self.watcher
                        .watch(&watch_path, RecursiveMode::NonRecursive)?;
                    self.watched_paths.push(watch_path);
                }
            }

            // 同时监控日志目录本身
            let log_dirs: Vec<PathBuf> = adapter
                .log_paths()
                .iter()
                .filter_map(|p| {
                    if p.is_file() {
                        p.parent().map(|pp| pp.to_path_buf())
                    } else {
                        Some(p.clone())
                    }
                })
                .collect();

            for dir in log_dirs {
                if dir.exists() && !self.watched_paths.contains(&dir) {
                    self.watcher.watch(&dir, RecursiveMode::NonRecursive)?;
                    self.watched_paths.push(dir);
                }
            }
        }

        Ok(())
    }

    /// 手动添加监控路径（用于测试或自定义配置）
    pub fn add_path(&mut self, path: PathBuf) -> Result<()> {
        if path.exists() {
            let watch_path = if path.is_file() {
                path.parent().unwrap_or(&path).to_path_buf()
            } else {
                path.clone()
            };

            if !self.watched_paths.contains(&watch_path) {
                self.watcher
                    .watch(&watch_path, RecursiveMode::NonRecursive)?;
                self.watched_paths.push(watch_path);
            }
        }
        Ok(())
    }

    /// 获取下一个文件系统事件（异步）
    pub async fn next_event(&mut self) -> Option<WatchEvent> {
        self.rx.recv().await
    }

    /// 获取当前已监控的路径列表
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}
