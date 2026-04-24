use crate::adapter::schema::{AgentType, ContextType, MemoryUnit};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::PathBuf;

/// SQLite 存储后端（v0.3）
pub struct SqliteStore {
    conn: Connection,
    db_path: PathBuf,
}

impl SqliteStore {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agentmem");

        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data dir: {:?}", data_dir))?;

        let db_path = data_dir.join("memories.db");
        Self::open(&db_path)
    }

    /// 从已有数据库路径打开（用于 WebUI 等并发场景）
    pub fn open(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open SQLite db: {:?}", db_path))?;

        let store = Self {
            conn,
            db_path: db_path.clone(),
        };
        store.init_schema()?;
        store.migrate_from_json()?;

        Ok(store)
    }

    /// 初始化数据库表结构
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id          TEXT PRIMARY KEY,
                agent_type  TEXT NOT NULL,
                timestamp   DATETIME NOT NULL,
                context_type TEXT NOT NULL,
                content     TEXT NOT NULL,
                confidence  REAL NOT NULL,
                source_file TEXT NOT NULL,
                project_path TEXT,
                tags        TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )?;

        // 全文搜索虚拟表
        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                content,
                tags,
                content_rowid=rowid,
                content='memories'
            )",
            [],
        )?;

        // 创建触发器保持 FTS 索引同步
        self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS memories_fts_insert
             AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, content, tags)
                VALUES (new.rowid, new.content, new.tags);
             END",
            [],
        )?;

        self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS memories_fts_delete
             AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, content, tags)
                VALUES ('delete', old.rowid, old.content, old.tags);
             END",
            [],
        )?;

        self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS memories_fts_update
             AFTER UPDATE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, content, tags)
                VALUES ('delete', old.rowid, old.content, old.tags);
                INSERT INTO memories_fts(rowid, content, tags)
                VALUES (new.rowid, new.content, new.tags);
             END",
            [],
        )?;

        // 常用查询索引
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_type ON memories(agent_type)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_context_type ON memories(context_type)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON memories(timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    /// 从旧版 JSON 文件迁移数据（一次性）
    fn migrate_from_json(&self) -> Result<()> {
        let json_path = self.db_path.with_file_name("memories.json");
        if !json_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&json_path)?;
        let memories: Vec<MemoryUnit> = serde_json::from_str(&content)?;

        if !memories.is_empty() {
            let mut migrated = 0;
            for m in memories {
                if self.insert(&m).is_ok() {
                    migrated += 1;
                }
            }
            // 迁移完成后重命名旧文件
            let backup = json_path.with_extension("json.backup");
            std::fs::rename(&json_path, &backup)?;
            println!(
                "✓ Migrated {} memories from JSON to SQLite",
                migrated
            );
        }

        Ok(())
    }

    fn insert(&self, memory: &MemoryUnit) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO memories
             (id, agent_type, timestamp, context_type, content, confidence, source_file, project_path, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                &memory.id,
                &memory.agent_type.to_string(),
                &memory.timestamp.to_rfc3339(),
                &memory.context_type.to_string(),
                &memory.content,
                &memory.confidence,
                &memory.source_file.to_string_lossy().to_string(),
                &memory.project_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                &serde_json::to_string(&memory.tags)?,
            ],
        )?;
        Ok(())
    }

    /// 添加一条记忆
    #[allow(dead_code)]
    pub fn add(&self, memory: MemoryUnit) -> Result<()> {
        self.insert(&memory)
    }

    /// 批量添加
    pub fn add_batch(&self, memories: Vec<MemoryUnit>) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for m in memories {
            let _ = self.insert(&m);
        }
        tx.commit()?;
        Ok(())
    }

    /// 加载所有记忆
    pub fn load_all(&self) -> Result<Vec<MemoryUnit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_type, timestamp, context_type, content, confidence, source_file, project_path, tags
             FROM memories ORDER BY timestamp DESC"
        )?;

        let rows: Vec<MemoryUnit> = stmt
            .query_map([], |row| Ok(self.row_to_memory(row)?))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// 关键词搜索（FTS5 + LIKE 回退）
    pub fn search(&self, query: &str) -> Result<Vec<MemoryUnit>> {
        // 先尝试 FTS5
        let fts_query = query.split_whitespace().collect::<Vec<_>>().join(" OR ");
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.agent_type, m.timestamp, m.context_type, m.content, m.confidence, m.source_file, m.project_path, m.tags
             FROM memories m
             JOIN memories_fts fts ON m.rowid = fts.rowid
             WHERE memories_fts MATCH ?1
             ORDER BY rank"
        )?;

        let rows: Vec<MemoryUnit> = stmt
            .query_map([&fts_query], |row| Ok(self.row_to_memory(row)?))?
            .collect::<Result<Vec<_>, _>>()?;

        if !rows.is_empty() {
            return Ok(rows);
        }

        // FTS 回退到 LIKE
        let like = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_type, timestamp, context_type, content, confidence, source_file, project_path, tags
             FROM memories
             WHERE content LIKE ?1 OR tags LIKE ?1
             ORDER BY timestamp DESC"
        )?;

        let rows: Vec<MemoryUnit> = stmt
            .query_map([&like], |row| Ok(self.row_to_memory(row)?))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// 语义搜索（基于关键词密度评分）
    pub fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<(MemoryUnit, f32)>> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let all = self.load_all()?;
        let mut scored: Vec<(MemoryUnit, f32)> = Vec::new();

        for m in all {
            let content_lower = m.content.to_lowercase();
            let mut score = 0.0_f32;

            // 关键词匹配得分
            for word in &query_words {
                if content_lower.contains(word) {
                    score += 1.0;
                }
            }

            // 标签匹配加分
            for tag in &m.tags {
                let tag_lower = tag.to_lowercase();
                for word in &query_words {
                    if tag_lower.contains(word) {
                        score += 0.5;
                    }
                }
            }

            // Agent 类型相关性
            if query_lower.contains(&m.agent_type.to_string().to_lowercase()) {
                score += 0.3;
            }

            // 上下文类型相关性
            if query_lower.contains(&m.context_type.to_string().to_lowercase()) {
                score += 0.3;
            }

            // 置信度加权
            score *= m.confidence;

            if score > 0.0 {
                scored.push((m, score));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(limit);
        Ok(scored)
    }

    /// 最近 N 条
    pub fn recent(&self, n: usize) -> Result<Vec<MemoryUnit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_type, timestamp, context_type, content, confidence, source_file, project_path, tags
             FROM memories ORDER BY timestamp DESC LIMIT ?1"
        )?;

        let rows: Vec<MemoryUnit> = stmt
            .query_map([n as i64], |row| Ok(self.row_to_memory(row)?))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// 按 Agent 过滤
    pub fn by_agent(&self, agent: &str) -> Result<Vec<MemoryUnit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_type, timestamp, context_type, content, confidence, source_file, project_path, tags
             FROM memories WHERE agent_type = ?1 ORDER BY timestamp DESC"
        )?;

        let rows: Vec<MemoryUnit> = stmt
            .query_map([agent.to_lowercase()], |row| Ok(self.row_to_memory(row)?))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// 按上下文类型过滤
    pub fn by_context_type(&self, ct: &str) -> Result<Vec<MemoryUnit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_type, timestamp, context_type, content, confidence, source_file, project_path, tags
             FROM memories WHERE context_type = ?1 ORDER BY timestamp DESC"
        )?;

        let rows: Vec<MemoryUnit> = stmt
            .query_map([ct.to_lowercase()], |row| Ok(self.row_to_memory(row)?))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// 删除指定 ID（支持前缀匹配）
    pub fn delete(&self, id: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM memories WHERE id LIKE ?1",
            [format!("{}%", id)],
        )?;
        Ok(affected > 0)
    }

    /// 获取总数
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memories",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// 获取数据目录
    pub fn data_dir(&self) -> PathBuf {
        self.db_path.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf()
    }

    /// 获取数据库路径
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    fn row_to_memory(&self, row: &rusqlite::Row) -> rusqlite::Result<MemoryUnit> {
        let agent_type = match row.get::<_, String>(1)?.to_lowercase().as_str() {
            "claude" => AgentType::Claude,
            "cursor" => AgentType::Cursor,
            "windsurf" => AgentType::Windsurf,
            "codex" => AgentType::Codex,
            "gemini" => AgentType::Gemini,
            "opencode" => AgentType::OpenCode,
            _ => AgentType::Unknown,
        };

        let context_type = match row.get::<_, String>(3)?.to_lowercase().as_str() {
            "preference" => ContextType::Preference,
            "decision" => ContextType::Decision,
            "pattern" => ContextType::Pattern,
            "fact" => ContextType::Fact,
            _ => ContextType::Unknown,
        };

        let timestamp_str: String = row.get(2)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let tags: Vec<String> = serde_json::from_str(&row.get::<_, String>(8)?)
            .unwrap_or_default();

        Ok(MemoryUnit {
            id: row.get(0)?,
            agent_type,
            timestamp,
            context_type,
            content: row.get(4)?,
            confidence: row.get(5)?,
            source_file: PathBuf::from(row.get::<_, String>(6)?),
            project_path: row.get::<_, Option<String>>(7)?.filter(|s: &String| !s.is_empty()).map(PathBuf::from),
            tags,
        })
    }
}
