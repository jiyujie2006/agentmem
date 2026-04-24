use crate::adapter::schema::MemoryUnit;
use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::path::PathBuf;

/// 基于 JSON 文件的存储后端（v0.1 简化方案）
/// v0.3 将迁移到 SQLite
#[allow(dead_code)]
pub struct JsonStore {
    data_dir: PathBuf,
    memories_file: PathBuf,
}

#[allow(dead_code)]
impl JsonStore {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agentmem");

        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data dir: {:?}", data_dir))?;

        let memories_file = data_dir.join("memories.json");

        // 初始化空文件
        if !memories_file.exists() {
            fs::write(&memories_file, "[]")?;
        }

        Ok(Self {
            data_dir,
            memories_file,
        })
    }

    /// 加载所有记忆
    pub fn load_all(&self) -> Result<Vec<MemoryUnit>> {
        let content = fs::read_to_string(&self.memories_file)
            .with_context(|| format!("Failed to read {:?}", self.memories_file))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let memories: Vec<MemoryUnit> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse memories.json")?;

        Ok(memories)
    }

    /// 保存所有记忆
    pub fn save_all(&self, memories: &[MemoryUnit]) -> Result<()> {
        let json = serde_json::to_string_pretty(memories)
            .with_context(|| "Failed to serialize memories")?;

        fs::write(&self.memories_file, json)
            .with_context(|| format!("Failed to write {:?}", self.memories_file))?;

        Ok(())
    }

    /// 添加一条记忆（去重检查）
    #[allow(dead_code)]
    pub fn add(&self, memory: MemoryUnit) -> Result<()> {
        let mut memories = self.load_all()?;

        // 简单去重：如果 content 完全相同则不添加
        if !memories.iter().any(|m| m.content == memory.content) {
            memories.push(memory);
            self.save_all(&memories)?;
        }

        Ok(())
    }

    /// 批量添加记忆
    pub fn add_batch(&self, new_memories: Vec<MemoryUnit>) -> Result<()> {
        let mut memories = self.load_all()?;
        let existing: std::collections::HashSet<String> =
            memories.iter().map(|m| m.content.clone()).collect();

        for m in new_memories {
            if !existing.contains(&m.content) {
                memories.push(m);
            }
        }

        self.save_all(&memories)?;
        Ok(())
    }

    /// 根据关键词搜索记忆
    pub fn search(&self, query: &str) -> Result<Vec<MemoryUnit>> {
        let memories = self.load_all()?;
        let query_lower = query.to_lowercase();

        Ok(memories
            .into_iter()
            .filter(|m| {
                m.content.to_lowercase().contains(&query_lower)
                    || m.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    /// 获取最近 N 条记忆
    pub fn recent(&self, n: usize) -> Result<Vec<MemoryUnit>> {
        let mut memories = self.load_all()?;
        // 按时间倒序
        memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(memories.into_iter().take(n).collect())
    }

    /// 按 Agent 类型过滤
    #[allow(dead_code)]
    pub fn by_agent(&self, agent: &str) -> Result<Vec<MemoryUnit>> {
        let memories = self.load_all()?;
        Ok(memories
            .into_iter()
            .filter(|m| m.agent_type.to_string() == agent.to_lowercase())
            .collect())
    }

    /// 删除指定 ID 的记忆
    pub fn delete(&self, id: &str) -> Result<bool> {
        let mut memories = self.load_all()?;
        let original_len = memories.len();
        memories.retain(|m| m.id != id);
        let removed = memories.len() < original_len;
        self.save_all(&memories)?;
        Ok(removed)
    }

    /// 获取记忆总数
    pub fn count(&self) -> Result<usize> {
        Ok(self.load_all()?.len())
    }

    /// 获取数据目录路径
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}

impl Default for JsonStore {
    fn default() -> Self {
        Self::new().expect("Failed to create JsonStore")
    }
}
