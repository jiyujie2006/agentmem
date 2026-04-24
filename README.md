# AgentMem

> **Install a 5MB binary. Your AI never forgets again.**
>
> [![GitHub stars](https://img.shields.io/github/stars/yourusername/agentmem?style=social)](https://github.com/yourusername/agentmem)
> ![License](https://img.shields.io/badge/license-MIT-blue.svg)
> ![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
> ![SQLite](https://img.shields.io/badge/storage-SQLite-blue.svg)

**AgentMem** is a lightweight local daemon that extracts key decisions and preferences from your AI coding conversations — across **Claude Code**, **Cursor**, **OpenCode**, and more — and automatically injects them into new sessions. Zero external dependencies. Zero cloud. 100% local SQLite.

---

## The Problem

Every time you start a new session with Claude Code, Cursor, or OpenCode, your agent **forgets everything**:

- "We use single quotes in this project" → forgotten
- "API base URL is `https://api.example.com/v2`" → forgotten  
- "Never use `var`, always `const`" → forgotten

You waste 5–20 minutes re-establishing context. Every. Single. Session.

**AgentMem fixes this.** It watches your conversation logs, extracts what matters, and reminds your AI before you even start typing.

---

## 30-Second Demo

```bash
# 1. Install
curl -sSL https://install.agentmem.dev | sh

# 2. Extract your existing conversations
agentmem extract ~/.claude/
agentmem extract ~/.cursor/

# 3. Inject context before your next session
agentmem inject
```

Output:
```
┏ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ ┓
┃ 🧠 AgentMem — Your AI remembers                                          ┃
┣ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ ┫
┃  1. ⚙  [preference] → Always use single quotes for strings               ┃
┃  2. 🔍 [fact]       → API base URL = https://api.example.com/v2          ┃
┃  3. 📌 [decision]   → Use PostgreSQL instead of MongoDB                  ┃
┃  4. ⚙  [preference] → Never use var, always const or let                 ┃
┃  5. 🔍 [fact]       → Dev server port = 3000                             ┃
┗ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ ┛
```

**Launch the WebUI dashboard** to visualize your AI memory:

```bash
agentmem webui --port 8080
# Open http://localhost:8080 in your browser
```

---

## Why AgentMem?

| | AgentMem | Claude-Mem | Mem0 |
|---|:---:|:---:|:---:|
| **Cross-platform** | ✅ All agents | ❌ Claude only | ❌ SDK required |
| **Deployment** | Single binary | Node.js + Chroma | Python service |
| **External deps** | **Zero** | Node.js + Chroma | Qdrant/Pinecone |
| **Privacy** | 100% local | Local | Cloud by default |
| **Setup** | One command | Multi-step | Infrastructure |
| **Semantic search** | ✅ Built-in | ❌ | ✅ (requires cloud) |
| **WebUI dashboard** | ✅ Built-in | ❌ | ❌ |

---

## Installation

### macOS / Linux (one line)

```bash
curl -sSL https://install.agentmem.dev | sh
```

### From Source

```bash
git clone https://github.com/yourusername/agentmem.git
cd agentmem
cargo build --release
# Binary: target/release/agentmem
```

---

## Quick Start

```bash
# Initialize
agentmem init

# Start the daemon (watches logs & auto-extracts)
agentmem daemon

# Or manually import existing conversations
agentmem extract ~/.claude/
agentmem extract ~/.cursor/
agentmem extract ~/.opencode/

# View your cross-agent memory
agentmem list
agentmem list --agent cursor     # filter by agent
agentmem list --context-type fact # filter by type

# Search
agentmem search "PostgreSQL"
agentmem search "how to config database" --semantic  # semantic relevance

# Inject into your next session
agentmem inject

# Launch the dashboard
agentmem webui
```

---

## Shell Integration

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# Auto-inject before Claude Code sessions
alias claude='agentmem inject && claude'
alias cursor='agentmem inject && cursor'
```

Or use the wrapper function for more control:

```bash
claude_with_mem() {
    agentmem inject --agent claude
    command claude "$@"
}
alias claude='claude_with_mem'
```

---

## How It Works

### 1. Watch
AgentMem monitors your AI agent log directories (`~/.claude/`, `~/.cursor/`, `~/.opencode/`) using filesystem events.

### 2. Extract
When a conversation file changes, AgentMem parses the messages and runs rule-based extraction:

| Trigger | Example | Type |
|---------|---------|------|
| "always use ..." | "Always use single quotes" | Preference |
| "from now on ..." | "From now on use TypeScript" | Preference |
| "base URL is ..." | "API base URL is https://..." | Fact |
| "decided to ..." | "We decided to use PostgreSQL" | Decision |

### 3. Store
Extracted memories are stored in a local **SQLite database** with FTS5 full-text search. Your data never leaves your machine. v0.1 JSON data is automatically migrated on first run.

### 4. Inject
When you run `agentmem inject`, the most relevant recent memories are displayed in a beautiful terminal box — ready for you to paste into your agent's system prompt.

### 5. Visualize
The built-in WebUI dashboard shows your memory distribution across agents, types, timeline, and tags — all running locally with zero external dependencies.

---

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Extractor  │────→│   Storage   │────→│  Injector   │────→│    WebUI    │
│  (fs notify)│     │  (SQLite)   │     │  (TUI/CLI)  │     │  (HTTP/JS)  │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
       ↑                                                              ↓
   Claude Logs                                                   Browser
   Cursor Logs                                                  Dashboard
   OpenCode Logs
```

**Non-invasive by design.** We don't hack into Claude or Cursor. We simply read their conversation logs, extract what matters, and give it back to you.

---

## Commands

```bash
agentmem daemon                    # Start background file watcher
agentmem extract [PATH]            # Manually scan logs & extract memories
agentmem list                      # Show stored memories
agentmem list --agent cursor       # Filter by agent
agentmem list --context-type fact  # Filter by type
agentmem search <QUERY>            # Keyword search (FTS5)
agentmem search <QUERY> --semantic # Semantic relevance search
agentmem inject                    # Display memories for session injection
agentmem delete <ID>               # Remove a specific memory
agentmem webui [--port 8080]       # Launch dashboard in browser
agentmem status                    # Show daemon & storage status
agentmem init                      # Initialize config & directories
```

---

## Roadmap

- [x] v0.1 — Claude Code support, JSON storage, rule extraction
- [x] v0.2 — Cursor adapter, cross-agent memory sync
- [x] v0.3 — SQLite + FTS5 + semantic search
- [x] v0.4 — OpenCode support, WebUI dashboard
- [ ] v1.1 — Windsurf IDE support
- [ ] v1.2 — Shell wrapper auto-install
- [ ] v2.0 — Team memory sync (end-to-end encrypted)

---

## Contributing

We love adapter contributions! Adding support for a new AI agent is as simple as implementing the `AgentAdapter` trait:

```rust
impl AgentAdapter for MyAgentAdapter {
    fn name(&self) -> &'static str { "my-agent" }
    fn log_paths(&self) -> Vec<PathBuf> { /* ... */ }
    fn parse_conversation(&self, content: &str) -> Result<Vec<RawMessage>> { /* ... */ }
    fn extract_memories(&self, messages: &[RawMessage]) -> Vec<MemoryUnit> { /* ... */ }
}
```

See [`src/adapter/`](src/adapter/) for examples.

---

## License

MIT © AgentMem Contributors
