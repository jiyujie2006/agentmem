pub mod summarizer;
pub mod watcher;

use crate::adapter::get_all_adapters;
use crate::adapter::schema::MemoryUnit;
use crate::extractor::summarizer::RuleBasedSummarizer;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// 从指定文件提取记忆
pub fn extract_from_file(path: &PathBuf, adapter_name: Option<&str>) -> Result<Vec<MemoryUnit>> {
    let content = fs::read_to_string(path)?;

    let adapters = get_all_adapters();
    let summarizer = RuleBasedSummarizer::new();

    // 如果指定了适配器名称，优先使用
    let mut memories = Vec::new();

    // 根据文件路径或参数推断适配器
    let inferred_name = adapter_name.map(|s| s.to_string())
        .or_else(|| infer_adapter_from_path(path));

    if let Some(name) = inferred_name {
        for adapter in adapters {
            if adapter.name() == name {
                let messages = adapter.parse_conversation(&content)?;
                let extracted = adapter.extract_memories(&messages);
                memories.extend(extracted);
                break;
            }
        }
    } else {
        // 尝试所有适配器，但只使用第一个成功提取的
        for adapter in adapters {
            let messages = adapter.parse_conversation(&content)?;
            if !messages.is_empty() {
                let extracted = adapter.extract_memories(&messages);
                if !extracted.is_empty() {
                    memories.extend(extracted);
                    break; // 只使用第一个成功提取的适配器
                }
            }
        }
    }

    // 如果适配器没有提取到，使用通用规则提取
    if memories.is_empty() {
        let extracted = summarizer.extract_from_text(&content, Some("fallback"));
        for e in extracted {
            memories.push(e.into_memory_unit(
                crate::adapter::schema::AgentType::Unknown,
                path.clone(),
                None,
            ));
        }
    }

    Ok(memories)
}

/// 根据文件路径推断适配器类型
fn infer_adapter_from_path(path: &PathBuf) -> Option<String> {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains(".cursor") || path_str.contains("/cursor/") {
        Some("cursor".to_string())
    } else if path_str.contains(".claude") || path_str.contains("/claude/") {
        Some("claude".to_string())
    } else if path_str.contains(".opencode") || path_str.contains("/opencode/") {
        Some("opencode".to_string())
    } else {
        None
    }
}

/// 扫描目录并提取所有记忆
pub fn scan_directory(dir: &PathBuf, recursive: bool) -> Result<Vec<MemoryUnit>> {
    let mut memories = Vec::new();

    if !dir.exists() {
        return Ok(memories);
    }

    let entries = if recursive {
        walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>()
    } else {
        std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect()
    };

    for path in entries {
        if let Ok(mut m) = extract_from_file(&path, None) {
            memories.append(&mut m);
        }
    }

    Ok(memories)
}
