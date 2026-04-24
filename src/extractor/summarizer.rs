use crate::adapter::schema::{ContextType, MemoryUnit};
use regex::Regex;

/// 基于规则的记忆提取器
/// v0.1 使用简单关键词匹配，无需本地模型
pub struct RuleBasedSummarizer {
    preference_keywords: Vec<Regex>,
    decision_keywords: Vec<Regex>,
    fact_patterns: Vec<Regex>,
}

impl RuleBasedSummarizer {
    pub fn new() -> Self {
        Self {
            preference_keywords: vec![
                Regex::new(r"(?i)always\s+use\s+(.+)").unwrap(),
                Regex::new(r"(?i)never\s+use\s+(.+)").unwrap(),
                Regex::new(r"(?i)prefer\s+(.+)").unwrap(),
                Regex::new(r"(?i)from\s+now\s+on[,:]?\s*(.+)").unwrap(),
                Regex::new(r"(?i)remember\s+(?:that\s+)?(.+)").unwrap(),
                Regex::new(r"(?i)the\s+standard\s+is\s+(.+)").unwrap(),
                Regex::new(r"(?i)use\s+(.+)\s+instead\s+of\s+(.+)").unwrap(),
            ],
            decision_keywords: vec![
                Regex::new(r"(?i)let'?s\s+use\s+(.+)").unwrap(),
                Regex::new(r"(?i)we\s+should\s+(.+)").unwrap(),
                Regex::new(r"(?i)decided\s+(?:to\s+)?(.+)").unwrap(),
                Regex::new(r"(?i)architecture[,:]?\s*(.+)").unwrap(),
            ],
            fact_patterns: vec![
                Regex::new(r"(?i)(api\s*(?:base\s*)?url)\s*[=:]\s*(\S+)").unwrap(),
                Regex::new(r"(?i)(database\s*(?:url|name))\s*[=:]\s*(\S+)").unwrap(),
                Regex::new(r"(?i)(port)\s*[=:]\s*(\d+)").unwrap(),
                Regex::new(r"(?i)(version)\s*[=:]\s*(\S+)").unwrap(),
            ],
        }
    }

    /// 从一段文本中提取记忆
    pub fn extract_from_text(&self, text: &str, source_hint: Option<&str>) -> Vec<ExtractedMemory> {
        let mut results = Vec::new();
        let text = text.trim();

        if text.len() < 10 || text.len() > 3000 {
            return results;
        }

        // 尝试匹配偏好
        for re in &self.preference_keywords {
            if let Some(cap) = re.captures(text) {
                let content = if cap.len() > 2 {
                    format!("Use {} instead of {}", cap[1].trim(), cap[2].trim())
                } else {
                    cap[0].trim().to_string()
                };

                results.push(ExtractedMemory {
                    context_type: ContextType::Preference,
                    content,
                    confidence: 0.82,
                    reason: source_hint.map(|s| s.to_string()),
                });
                break; // 一行只提取一个偏好
            }
        }

        // 尝试匹配决策
        for re in &self.decision_keywords {
            if let Some(cap) = re.captures(text) {
                results.push(ExtractedMemory {
                    context_type: ContextType::Decision,
                    content: cap[0].trim().to_string(),
                    confidence: 0.75,
                    reason: source_hint.map(|s| s.to_string()),
                });
                break;
            }
        }

        // 尝试匹配事实
        for re in &self.fact_patterns {
            if let Some(cap) = re.captures(text) {
                results.push(ExtractedMemory {
                    context_type: ContextType::Fact,
                    content: format!("{} = {}", cap[1].trim(), cap[2].trim()),
                    confidence: 0.92,
                    reason: source_hint.map(|s| s.to_string()),
                });
            }
        }

        results
    }

    /// 批量处理多条消息
    #[allow(dead_code)]
    pub fn extract_batch(&self, texts: &[String]) -> Vec<ExtractedMemory> {
        let mut all = Vec::new();
        for text in texts {
            let mut extracted = self.extract_from_text(text, None);
            all.append(&mut extracted);
        }
        all
    }
}

impl Default for RuleBasedSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 提取出的记忆（尚未关联 Agent 和路径）
#[derive(Debug, Clone)]
pub struct ExtractedMemory {
    pub context_type: ContextType,
    pub content: String,
    pub confidence: f32,
    pub reason: Option<String>,
}

impl ExtractedMemory {
    pub fn into_memory_unit(
        self,
        agent_type: crate::adapter::schema::AgentType,
        source_file: std::path::PathBuf,
        project_path: Option<std::path::PathBuf>,
    ) -> MemoryUnit {
        let mut memory = MemoryUnit::new(
            agent_type,
            self.context_type,
            self.content,
            source_file,
            project_path,
        );
        memory = memory.with_confidence(self.confidence);
        if let Some(reason) = self.reason {
            memory = memory.with_tags(vec!["auto-extracted".to_string(), reason]);
        } else {
            memory = memory.with_tags(vec!["auto-extracted".to_string()]);
        }
        memory
    }
}
