use crate::adapter::schema::{AgentType, ContextType, MemoryUnit, RawMessage};
use crate::adapter::AgentAdapter;
use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::path::PathBuf;

pub struct CursorAdapter {
    log_dir: PathBuf,
    preference_re: Regex,
}

impl CursorAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        // Cursor 通常存储在用户目录下的 .cursor/ 中
        // Linux 也可能是 ~/.config/Cursor/
        let log_dir = if home.join(".cursor").exists() {
            home.join(".cursor")
        } else {
            home.join(".config").join("Cursor")
        };

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
                "going forward",
                "let's use",
            ]
            .join("|")
        );

        Self {
            log_dir,
            preference_re: Regex::new(&pattern).expect("valid regex"),
        }
    }

    fn find_log_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if !self.log_dir.exists() {
            return files;
        }

        // Cursor 可能的日志位置
        let candidates = [
            self.log_dir.join("chat_history"),
            self.log_dir.join("conversations"),
            self.log_dir.join("history"),
            self.log_dir.clone(),
        ];

        for dir in &candidates {
            if dir.exists() && dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "json").unwrap_or(false) {
                            files.push(path);
                        }
                    }
                }
            }
        }

        files
    }
}

impl AgentAdapter for CursorAdapter {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn is_available(&self) -> bool {
        self.log_dir.exists()
    }

    fn log_paths(&self) -> Vec<PathBuf> {
        self.find_log_files()
    }

    fn parse_conversation(&self, content: &str) -> Result<Vec<RawMessage>> {
        let mut messages = Vec::new();

        // Cursor 的 JSON 格式通常类似:
        // { "messages": [{"role":"user","content":"..."}, ...] }
        // 或者是直接的数组格式
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

            // Cursor 可能使用不同的字段名
            if let Some(arr) = obj.get("chats").and_then(|v| v.as_array()) {
                for item in arr {
                    if let (Some(role), Some(text)) = (
                        item.get("sender").and_then(|v| v.as_str()),
                        item.get("text").and_then(|v| v.as_str()),
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

        Ok(messages)
    }

    fn extract_memories(&self, messages: &[RawMessage]) -> Vec<MemoryUnit> {
        let mut memories = Vec::new();

        for msg in messages {
            if !msg.role.eq_ignore_ascii_case("user") {
                continue;
            }

            // 偏好提取
            if self.preference_re.is_match(&msg.content) {
                let content = msg.content.trim().to_string();
                if content.len() > 10 && content.len() < 2000 {
                    let mut memory = MemoryUnit::new(
                        AgentType::Cursor,
                        ContextType::Preference,
                        content.clone(),
                        self.log_dir.clone(),
                        None,
                    );
                    memory = memory.with_confidence(0.82);
                    memory = memory.with_tags(vec![
                        "auto-extracted".to_string(),
                        "preference".to_string(),
                    ]);
                    memories.push(memory);
                }
            }

            // 事实提取
            let fact_re = Regex::new(
                r"(?i)(api\s*(?:base\s*)?url|endpoint|port|database\s*url|token|key)\s*[=:]\s*(\S+)"
            ).unwrap();
            for cap in fact_re.captures_iter(&msg.content) {
                let fact = format!("{} = {}", &cap[1], &cap[2]);
                let mut memory = MemoryUnit::new(
                    AgentType::Cursor,
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

impl Default for CursorAdapter {
    fn default() -> Self {
        Self::new()
    }
}
