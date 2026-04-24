# Reddit Launch Posts

## r/programming

**Title**: I built a 5MB tool that makes AI coding agents remember across sessions

**Body**:

Every time I start a new Claude Code or Cursor session, I have to re-explain our codebase conventions. "Use single quotes." "API base URL is X." "Never use var."

So I built AgentMem — a tiny local daemon that:
- Watches your agent conversation logs
- Auto-extracts key decisions/preferences
- Stores them in local SQLite
- Injects them back when you start a new session

It works across Claude Code, Cursor, and OpenCode. Zero external dependencies, zero cloud, 5MB single binary.

```bash
curl -sSL https://github.com/jiyujie2006/agentmem/releases/latest/download/install.sh | sh
agentmem extract ~/.claude/
agentmem inject
```

GitHub: https://github.com/jiyujie2006/agentmem

Curious if others feel this pain or if it's just me being lazy.

---

## r/ClaudeCode

**Title**: Show HN: I made Claude Code remember across sessions (cross-agent memory layer)

**Body**:

Tired of Claude forgetting everything when you start a new session? I built a tiny tool that persists your key decisions and preferences locally.

- Auto-extracts from `.claude/` logs
- Stores in local SQLite (not cloud)
- Injects context before new sessions
- Also works with Cursor and OpenCode

5MB binary, one-line install.

https://github.com/jiyujie2006/agentmem

---

## r/cursor

**Title**: Cross-session memory for Cursor (and Claude Code, OpenCode)

**Body**:

Built a lightweight daemon that watches your Cursor chat logs and persists key decisions across sessions. No cloud, no config, 5MB binary.

Works by reading `~/.cursor/` logs, extracting preferences with regex rules, storing in SQLite, and displaying a prompt box before new sessions.

Also has a built-in WebUI dashboard to visualize your memory distribution.

https://github.com/jiyujie2006/agentmem
