use crate::adapter::schema::{AgentType, ContextType, MemoryUnit, RawMessage};
use crate::adapter::AgentAdapter;
use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::path::PathBuf;

pub struct OpenCodeAdapter {
    log_dir: PathBuf,
    preference_re: Regex,
}

impl OpenCodeAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        // OpenCode 可能存储在 ~/.opencode/ 或 ~/.config/opencode/
        let log_dir = if home.join(".opencode").exists() {
            home.join(".opencode")
        } else {
            home.join(".config").join("opencode")
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
                "we use",
                "we don't",
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

        let candidates = [
            self.log_dir.join("chat_history"),
            self.log_dir.join("conversations"),
            self.log_dir.join("history"),
            self.log_dir.join("logs"),
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

impl AgentAdapter for OpenCodeAdapter {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn is_available(&self) -> bool {
        self.log_dir.exists()
    }

    fn log_paths(&self) -> Vec<PathBuf> {
        self.find_log_files()
    }

    fn parse_conversation(&self, content: &str) -> Result<Vec<RawMessage>> {
        let mut messages = Vec::new();

        // JSON 数组格式
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

        // 嵌套 messages 数组
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
                        AgentType::OpenCode,
                        ContextType::Preference,
                        content.clone(),
                        self.log_dir.clone(),
                        None,
                    );
                    memory = memory.with_confidence(0.82);
                    memory = memory.with_tags(vec![
                        "auto-extracted".to_string(),
                        "preference".to_string(),
                        "opencode".to_string(),
                    ]);
                    memories.push(memory);
                }
            }

            // 事实提取
            let fact_re = Regex::new(
                r"(?i)(api\s*(?:base\s*)?url|endpoint|port|database\s*url|token|key|host)\s*[=:]\s*(\S+)"
            ).unwrap();
            for cap in fact_re.captures_iter(&msg.content) {
                let fact = format!("{} = {}", &cap[1], &cap[2]);
                let mut memory = MemoryUnit::new(
                    AgentType::OpenCode,
                    ContextType::Fact,
                    fact,
                    self.log_dir.clone(),
                    None,
                );
                memory = memory.with_confidence(0.9);
                memory = memory.with_tags(vec!["fact".to_string(), "config".to_string(), "opencode".to_string()]);
                memories.push(memory);
            }
        }

        memories
    }
}

impl Default for OpenCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}
