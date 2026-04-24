# Show HN: I made AI coding agents remember across sessions

**AgentMem** — a 5MB single-binary daemon that makes Claude Code, Cursor, and OpenCode remember your preferences and decisions across sessions.

## The Problem

I was tired of repeating myself to Claude every morning. "We use single quotes." "The API base URL is X." "Never use var." Every new session, same context, wasted 10 minutes.

## What it does

- **Watches** your agent conversation logs (Claude Code, Cursor, OpenCode)
- **Extracts** key decisions and preferences automatically
- **Stores** them in a local SQLite database (zero cloud, zero external deps)
- **Injects** them back into new sessions via a colored terminal prompt
- **Visualizes** everything in a built-in WebUI dashboard

## Demo

```bash
curl -sSL https://github.com/jiyujie2006/agentmem/releases/latest/download/install.sh | sh
agentmem extract ~/.claude/
agentmem inject
```

Output:
```
┏ 🧠 AgentMem — Your AI remembers ┓
┃ 1. ⚙ Always use single quotes   ┃
┃ 2. 🔍 API base URL = https://...┃
┃ 3. 📌 Use PostgreSQL not Mongo  ┃
┗ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

## Why this over alternatives?

| | AgentMem | Claude-Mem | Mem0 |
|---|---|---|---|
| Cross-agent | ✅ | ❌ Claude only | ❌ SDK required |
| External deps | **Zero** | Node+Chroma | Qdrant/Pinecone |
| Privacy | 100% local | Local | Cloud by default |

## Tech stack

Rust + SQLite + FTS5. Single 5MB binary. No Docker, no config files, no API keys.

**GitHub**: https://github.com/jiyujie2006/agentmem

Would love feedback from anyone else hitting this "groundhog day" context problem.
