use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Claude,
    Cursor,
    Windsurf,
    Codex,
    Gemini,
    OpenCode,
    Unknown,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "claude"),
            AgentType::Cursor => write!(f, "cursor"),
            AgentType::Windsurf => write!(f, "windsurf"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::Gemini => write!(f, "gemini"),
            AgentType::OpenCode => write!(f, "opencode"),
            AgentType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    Preference,  // 用户偏好（编码风格、命名规范等）
    Decision,    // 架构决策
    Pattern,     // 代码模式
    Fact,        // 项目事实（API URL、配置等）
    Unknown,
}

impl std::fmt::Display for ContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextType::Preference => write!(f, "preference"),
            ContextType::Decision => write!(f, "decision"),
            ContextType::Pattern => write!(f, "pattern"),
            ContextType::Fact => write!(f, "fact"),
            ContextType::Unknown => write!(f, "unknown"),
        }
    }
}

/// 从原始对话中提取的消息
#[derive(Debug, Clone)]
pub struct RawMessage {
    pub role: String,       // "user" | "assistant"
    pub content: String,
    #[allow(dead_code)]
    pub timestamp: Option<DateTime<Utc>>,
}

/// 统一的记忆单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUnit {
    pub id: String,
    pub agent_type: AgentType,
    pub timestamp: DateTime<Utc>,
    pub context_type: ContextType,
    pub content: String,
    pub confidence: f32,
    pub source_file: PathBuf,
    pub project_path: Option<PathBuf>,
    pub tags: Vec<String>,
}

impl MemoryUnit {
    pub fn new(
        agent_type: AgentType,
        context_type: ContextType,
        content: String,
        source_file: PathBuf,
        project_path: Option<PathBuf>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_type,
            timestamp: Utc::now(),
            context_type,
            confidence: 0.8,
            content,
            source_file,
            project_path,
            tags: Vec::new(),
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}
