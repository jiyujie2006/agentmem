use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// AgentMem 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 数据存储目录
    pub data_dir: PathBuf,

    /// 额外监控的日志路径
    pub extra_watch_paths: Vec<PathBuf>,

    /// 启动时自动注入记忆的最大条数
    pub inject_max_memories: usize,

    /// 记忆提取的最低置信度阈值
    pub min_confidence: f32,

    /// 是否启用文件监控（守护进程模式）
    pub enable_watcher: bool,

    /// 守护进程轮询间隔（秒）
    pub poll_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agentmem");

        Self {
            data_dir,
            extra_watch_paths: Vec::new(),
            inject_max_memories: 5,
            min_confidence: 0.7,
            enable_watcher: true,
            poll_interval_secs: 5,
        }
    }
}

impl Config {
    /// 加载配置，如果不存在则创建默认配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// 配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agentmem");
        Ok(config_dir.join("config.toml"))
    }

    /// 添加额外监控路径
    #[allow(dead_code)]
    pub fn add_watch_path(&mut self, path: PathBuf) {
        if !self.extra_watch_paths.contains(&path) {
            self.extra_watch_paths.push(path);
        }
    }
}
