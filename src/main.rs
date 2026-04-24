mod adapter;
mod cli;
mod config;
mod daemon;
mod extractor;
mod injector;
mod storage;
mod webui;

use anyhow::Result;
use clap::Parser;
use colored::*;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::extractor::{extract_from_file, scan_directory};
use crate::injector::tui;
use crate::storage::sqlite_store::SqliteStore;
use crate::webui::server;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置和存储
    let config = Config::load().unwrap_or_default();
    let store = SqliteStore::new()?;

    match cli.command {
        Some(Commands::Daemon) => {
            tui::print_welcome();
            daemon::run_daemon(&config, &store).await?;
        }

        Some(Commands::Extract { path }) => {
            let target = match path {
                Some(p) => p,
                None => {
                    // 尝试找到默认的 claude 日志目录
                    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                    home.join(".claude")
                }
            };

            println!("{} Scanning: {}", "→".dimmed(), target.display().to_string().cyan());

            let memories = if target.is_file() {
                extract_from_file(&target, cli.agent.as_deref())?
            } else {
                scan_directory(&target, false)?
            };

            if memories.is_empty() {
                println!("{}", "No memories extracted.".dimmed());
            } else {
                store.add_batch(memories.clone())?;
                println!(
                    "{} Extracted and saved {} memory units\n",
                    "✓".green(),
                    memories.len().to_string().bold()
                );
                tui::print_memory_list(&memories);
            }
        }

        Some(Commands::List { limit, agent, context_type }) => {
            let memories = if let Some(agent_filter) = agent {
                store.by_agent(&agent_filter)?
            } else if let Some(ct_filter) = context_type {
                store.by_context_type(&ct_filter)?
            } else {
                store.recent(limit)?
            };
            tui::print_memory_list(&memories);
        }

        Some(Commands::Search { query, limit, semantic }) => {
            if semantic {
                let results = store.semantic_search(&query, limit)?;
                if results.is_empty() {
                    println!("{} No results for '{}'", "✗".red(), query.yellow());
                } else {
                    println!("{} Found {} semantic result(s) for '{}'\n", "✓".green(), results.len(), query.yellow());
                    let memories: Vec<_> = results.into_iter().map(|(m, _)| m).collect();
                    tui::print_memory_list(&memories);
                }
            } else {
                let memories = store.search(&query)?;
                if memories.is_empty() {
                    println!("{} No results for '{}'", "✗".red(), query.yellow());
                } else {
                    println!("{} Found {} result(s) for '{}'\n", "✓".green(), memories.len(), query.yellow());
                    tui::print_memory_list(&memories);
                }
            }
        }

        Some(Commands::Inject { limit }) => {
            let memories = store.recent(limit)?;
            if memories.is_empty() {
                println!("{}", "⚠ No memories to inject. Run 'agentmem extract' first.".yellow());
            } else {
                tui::print_memory_box(&memories, cli.agent.as_deref());
            }
        }

        Some(Commands::Delete { id }) => {
            if store.delete(&id)? {
                println!("{} Memory {} deleted", "✓".green(), id.yellow());
            } else {
                println!("{} Memory {} not found", "✗".red(), id.yellow());
            }
        }

        Some(Commands::Init { shell }) => {
            let data_dir = store.data_dir();
            println!("{} AgentMem initialized!", "✓".green());
            println!("  Data directory: {}", data_dir.display().to_string().cyan());
            println!("  Config file:    {}", Config::config_path()?.display().to_string().cyan());

            if let Some(shell_type) = shell {
                print_shell_setup(&shell_type)?;
            } else {
                println!("\n{} To enable automatic context injection, add a shell wrapper:", "💡".yellow());
                println!("  alias claude='agentmem inject && claude'");
                println!("\n{} Or run 'agentmem init --shell zsh' for setup instructions.", "→".dimmed());
            }
        }

        Some(Commands::Webui { port }) => {
            server::run_webui(store, port).await?;
        }

        Some(Commands::Status) => {
            let count = store.count()?;
            let adapters = adapter::get_all_adapters();

            println!("{}", "AgentMem Status".bold().white());
            println!("{}", "═".repeat(50).dimmed());
            println!("  Version:        {}", env!("CARGO_PKG_VERSION").cyan());
            println!("  Data directory: {}", store.data_dir().display().to_string().cyan());
            println!("  Memories:       {}", count.to_string().bold().white());
            println!("  Adapters:       {}", adapters.len().to_string().bold().white());
            for adapter in adapters {
                println!("    • {} ({})", adapter.name().green(), "active".green());
            }
            println!("{}", "═".repeat(50).dimmed());
        }

        None => {
            tui::print_welcome();
            println!("{}", "Usage: agentmem <COMMAND>".bold());
            println!();
            println!("{} Run 'agentmem --help' for more information.", "→".dimmed());
        }
    }

    Ok(())
}

fn print_shell_setup(shell_type: &str) -> Result<()> {
    let wrapper = match shell_type.to_lowercase().as_str() {
        "zsh" | "bash" => {
            r#"
# Add to your ~/.zshrc or ~/.bashrc:

# AgentMem: Auto-inject memory before Claude Code sessions
claude_with_mem() {
    agentmem inject --agent claude
    command claude "$@"
}
alias claude='claude_with_mem'

# Or use a simpler wrapper:
# alias claude='agentmem inject && claude'
"#
        }
        "fish" => {
            r#"
# Add to your ~/.config/fish/config.fish:

function claude_with_mem
    agentmem inject --agent claude
    command claude $argv
end

alias claude='claude_with_mem'
"#
        }
        _ => {
            println!("{} Shell '{}' not specifically supported yet.", "⚠".yellow(), shell_type);
            println!("  Add this to your shell config:");
            println!("  alias claude='agentmem inject && claude'");
            return Ok(());
        }
    };

    println!("\n{} Add the following to your shell config:", "💡".yellow());
    println!("{}", wrapper.cyan());
    Ok(())
}
