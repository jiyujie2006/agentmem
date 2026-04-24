use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "agentmem")]
#[command(about = "A lightweight local daemon that makes AI coding agents remember across sessions")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// 指定 Agent 类型（claude, cursor, windsurf）
    #[arg(short, long, global = true)]
    pub agent: Option<String>,

    /// 指定项目路径
    #[arg(short, long, global = true)]
    pub project: Option<PathBuf>,

    /// 开启详细日志
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 启动守护进程（监控日志文件并自动提取记忆）
    Daemon,

    /// 手动扫描日志文件并提取记忆
    Extract {
        /// 要扫描的文件或目录路径
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },

    /// 列出所有已存储的记忆
    List {
        /// 限制输出条数
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// 按 Agent 过滤
        #[arg(short, long)]
        agent: Option<String>,

        /// 按上下文类型过滤（preference, decision, fact, pattern）
        #[arg(short, long)]
        context_type: Option<String>,
    },

    /// 搜索记忆（支持语义检索）
    Search {
        /// 搜索关键词
        #[arg(value_name = "QUERY")]
        query: String,

        /// 限制结果条数
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// 启用语义搜索（按相关性排序）
        #[arg(long)]
        semantic: bool,
    },

    /// 显示与新会话最相关的记忆（注入上下文）
    Inject {
        /// 限制注入条数
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },

    /// 删除指定 ID 的记忆
    Delete {
        /// 记忆 ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// 初始化配置
    Init {
        /// Shell 类型（用于安装 wrapper）
        #[arg(short, long)]
        shell: Option<String>,
    },

    /// 启动 WebUI 仪表盘
    Webui {
        /// 监听端口
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// 显示状态信息
    Status,
}
