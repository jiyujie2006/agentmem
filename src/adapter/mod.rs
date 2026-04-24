pub mod claude;
pub mod cursor;
pub mod opencode;
pub mod schema;

use crate::adapter::schema::{MemoryUnit, RawMessage};
use anyhow::Result;
use std::path::PathBuf;

/// 所有 Agent 适配器必须实现的接口
pub trait AgentAdapter: Send + Sync {
    /// 适配器名称
    fn name(&self) -> &'static str;

    /// 检测该 Agent 是否在系统中可用
    fn is_available(&self) -> bool;

    /// 返回该 Agent 的日志文件/目录路径列表
    fn log_paths(&self) -> Vec<PathBuf>;

    /// 解析原始对话内容为一组消息
    fn parse_conversation(&self, content: &str) -> Result<Vec<RawMessage>>;

    /// 从消息列表中提取记忆单元
    fn extract_memories(&self, messages: &[RawMessage]) -> Vec<MemoryUnit>;
}

/// 获取所有已注册的适配器
pub fn get_all_adapters() -> Vec<Box<dyn AgentAdapter>> {
    let mut adapters: Vec<Box<dyn AgentAdapter>> = Vec::new();

    let claude = claude::ClaudeAdapter::new();
    if claude.is_available() {
        adapters.push(Box::new(claude));
    }

    let cursor = cursor::CursorAdapter::new();
    if cursor.is_available() {
        adapters.push(Box::new(cursor));
    }

    let opencode = opencode::OpenCodeAdapter::new();
    if opencode.is_available() {
        adapters.push(Box::new(opencode));
    }

    adapters
}

/// 根据名称查找适配器
#[allow(dead_code)]
pub fn find_adapter(name: &str) -> Option<Box<dyn AgentAdapter>> {
    match name.to_lowercase().as_str() {
        "claude" | "claude-code" => {
            let a = claude::ClaudeAdapter::new();
            if a.is_available() {
                Some(Box::new(a))
            } else {
                None
            }
        }
        "cursor" => {
            let a = cursor::CursorAdapter::new();
            if a.is_available() {
                Some(Box::new(a))
            } else {
                None
            }
        }
        "opencode" => {
            let a = opencode::OpenCodeAdapter::new();
            if a.is_available() {
                Some(Box::new(a))
            } else {
                None
            }
        }
        _ => None,
    }
}
