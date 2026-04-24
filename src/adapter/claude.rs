use crate::adapter::schema::{AgentType, ContextType, MemoryUnit, RawMessage};
use crate::adapter::AgentAdapter;
use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::path::PathBuf;

pub struct ClaudeAdapter {
    log_dir: PathBuf,
    preference_re: Regex,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let log_dir = home.join(".claude");

        // 匹配用户明确表达偏好的关键词
        let pattern = format!(
            r"(?i)({}).*",
            [
                "always use",
                "never use",
                "prefer",
                "remember to",
                "from now on",
                "standard is",
                "base url",
                "api endpoint",
                "don't use",
                "instead of",
                "should be",
                "must be",
                "the convention is",
            ]
            .join("|")
        );

        Self {
            log_dir,
            preference_re: Regex::new(&pattern).expect("valid regex"),
        }
    }

    /// 查找所有对话日志文件
    fn find_log_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if !self.log_dir.exists() {
            return files;
        }

        // Claude Code 可能在 .claude/ 下存储对话历史
        // 常见位置：.claude/conversations/ 或 .claude/projects/*/messages.json
        let conversations_dir = self.log_dir.join("conversations");
        if conversations_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&conversations_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "json").unwrap_or(false) {
                        files.push(path);
                    }
                }
            }
        }

        // 也检查直接的 .claude/ 目录下的 json 文件
        if let Ok(entries) = std::fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    files.push(path);
                }
            }
        }

        files
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn is_available(&self) -> bool {
        self.log_dir.exists()
    }

    fn log_paths(&self) -> Vec<PathBuf> {
        self.find_log_files()
    }

    fn parse_conversation(&self, content: &str) -> Result<Vec<RawMessage>> {
        let mut messages = Vec::new();

        // 尝试解析为 JSON 数组
        if let Ok(arr) = serde_json::from_str::<Vec<Value>>(content) {
            for item in arr {
                if let (Some(role), Some(text)) = (
                    item.get("role").and_then(|v| v.as_str()),
                    item.get("content").and_then(|v| v.as_str()),
                ) {
                    messages.push(RawMessage {
                        role: role.to_string(),
                        content: text.to_string(),
                        timestamp: None,
                    });
                }
            }
            return Ok(messages);
        }

        // 尝试解析为单个 JSON 对象（包含 messages 数组）
        if let Ok(obj) = serde_json::from_str::<Value>(content) {
            if let Some(arr) = obj.get("messages").and_then(|v| v.as_array()) {
                for item in arr {
                    if let (Some(role), Some(text)) = (
                        item.get("role").and_then(|v| v.as_str()),
                        item.get("content").and_then(|v| v.as_str()),
                    ) {
                        messages.push(RawMessage {
                            role: role.to_string(),
                            content: text.to_string(),
                            timestamp: None,
                        });
                    }
                }
                return Ok(messages);
            }
        }

        // 回退：按行解析，寻找 "User:" 和 "Assistant:" 标记
        let user_re = Regex::new(r"(?i)^user[:\s-]+(.*)$").unwrap();
        let assistant_re = Regex::new(r"(?i)^assistant[:\s-]+(.*)$").unwrap();

        for line in content.lines() {
            if let Some(cap) = user_re.captures(line) {
                messages.push(RawMessage {
                    role: "user".to_string(),
                    content: cap[1].trim().to_string(),
                    timestamp: None,
                });
            } else if let Some(cap) = assistant_re.captures(line) {
                messages.push(RawMessage {
                    role: "assistant".to_string(),
                    content: cap[1].trim().to_string(),
                    timestamp: None,
                });
            }
        }

        Ok(messages)
    }

    fn extract_memories(&self, messages: &[RawMessage]) -> Vec<MemoryUnit> {
        let mut memories = Vec::new();

        for msg in messages {
            // 只从用户消息中提取偏好
            if !msg.role.eq_ignore_ascii_case("user") {
                continue;
            }

            // 检查是否包含偏好关键词
            if self.preference_re.is_match(&msg.content) {
                let content = msg.content.trim().to_string();
                if content.len() > 10 && content.len() < 2000 {
                    let mut memory = MemoryUnit::new(
                        AgentType::Claude,
                        ContextType::Preference,
                        content,
                        self.log_dir.clone(),
                        None,
                    );
                    memory = memory.with_confidence(0.85);
                    memory = memory.with_tags(vec![
                        "auto-extracted".to_string(),
                        "preference".to_string(),
                    ]);
                    memories.push(memory);
                }
            }

            // 额外规则：检测 "API base URL is ..." 这类事实
            let fact_re = Regex::new(
                r"(?i)(api\s*(?:base\s*)?url|endpoint|port|database\s*url)\s*[=:]\s*(\S+)"
            ).unwrap();
            for cap in fact_re.captures_iter(&msg.content) {
                let fact = format!("{} = {}", &cap[1], &cap[2]);
                let mut memory = MemoryUnit::new(
                    AgentType::Claude,
                    ContextType::Fact,
                    fact,
                    self.log_dir.clone(),
                    None,
                );
                memory = memory.with_confidence(0.9);
                memory = memory.with_tags(vec!["fact".to_string(), "config".to_string()]);
                memories.push(memory);
            }
        }

        memories
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}
