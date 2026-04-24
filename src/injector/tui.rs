use crate::adapter::schema::{ContextType, MemoryUnit};
use colored::*;

/// 在终端显示记忆提示框
pub fn print_memory_box(memories: &[MemoryUnit], agent_name: Option<&str>) {
    if memories.is_empty() {
        return;
    }

    let title = match agent_name {
        Some(name) => format!("🧠 AgentMem — {} remembers", name),
        None => "🧠 AgentMem — Your AI remembers".to_string(),
    };

    let width = 72;
    let border = "━".repeat(width);

    println!();
    println!("{} {} {}", "┏".bright_blue(), border.bright_blue(), "┓".bright_blue());
    println!(
        "{} {:<width$} {}",
        "┃".bright_blue(),
        title.bold().white(),
        "┃".bright_blue(),
        width = width - 1
    );
    println!("{} {} {}", "┣".bright_blue(), border.bright_blue(), "┫".bright_blue());

    for (i, memory) in memories.iter().take(5).enumerate() {
        let icon = context_icon(&memory.context_type);
        let label = format!("[{}]", memory.context_type).dimmed();
        let content = truncate(&memory.content, width - 12);

        println!(
            "{} {:>2}. {} {:<6} {} {}",
            "┃".bright_blue(),
            (i + 1).to_string().dimmed(),
            icon,
            label,
            "→".dimmed(),
            content.white(),
        );
    }

    if memories.len() > 5 {
        let more = format!("... and {} more memories", memories.len() - 5);
        println!(
            "{} {:>width$} {}",
            "┃".bright_blue(),
            more.dimmed(),
            "┃".bright_blue(),
            width = width + 4
        );
    }

    println!("{} {} {}", "┗".bright_blue(), border.bright_blue(), "┛".bright_blue());
    println!();
}

/// 简单的列表输出（用于 CLI list 命令）
pub fn print_memory_list(memories: &[MemoryUnit]) {
    if memories.is_empty() {
        println!("{}", "No memories found.".dimmed());
        return;
    }

    println!(
        "\n{:<36} {:<12} {:<14} {}",
        "ID".dimmed(),
        "Agent".dimmed(),
        "Type".dimmed(),
        "Content".bold()
    );
    println!("{}", "─".repeat(100).dimmed());

    for m in memories {
        let id_short = &m.id[..8.min(m.id.len())];
        let content = truncate(&m.content, 50);
        let type_colored = match m.context_type {
            ContextType::Preference => "preference".yellow(),
            ContextType::Decision => "decision".cyan(),
            ContextType::Fact => "fact".green(),
            ContextType::Pattern => "pattern".magenta(),
            ContextType::Unknown => "unknown".normal(),
        };

        println!(
            "{:<36} {:<12} {:<14} {}",
            id_short.dimmed(),
            m.agent_type.to_string().dimmed(),
            type_colored,
            content
        );
    }

    println!("{}", "─".repeat(100).dimmed());
    println!("{} {} memory units stored\n", "→".dimmed(), memories.len().to_string().bold());
}

/// 打印启动欢迎信息
pub fn print_welcome() {
    println!();
    println!("{}", "  ╔═══════════════════════════════════════════════════════════════╗".bright_blue());
    println!("  {} {:<61} {}", "║".bright_blue(), format!("AgentMem v{}", env!("CARGO_PKG_VERSION")).bold().white(), "║".bright_blue());
    println!("  {} {:<61} {}", "║".bright_blue(), "Your AI coding agents, with persistent memory".white(), "║".bright_blue());
    println!("{}", "  ╚═══════════════════════════════════════════════════════════════╝".bright_blue());
    println!();
    println!("  {} {}", "📁".yellow(), "Data directory: ~/.local/share/agentmem/ (Linux)".dimmed());
    println!("  {} {}", "💡".yellow(), "Run 'agentmem --help' for available commands".dimmed());
    println!();
}

fn context_icon(ct: &ContextType) -> ColoredString {
    match ct {
        ContextType::Preference => "⚙ ".yellow(),
        ContextType::Decision => "📌".cyan(),
        ContextType::Fact => "🔍".green(),
        ContextType::Pattern => "🔄".magenta(),
        ContextType::Unknown => "❓".normal(),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}
