# AgentMem v1.0.0 — Your AI Never Forgets

## What's New

AgentMem v1.0 is the first stable release of the cross-agent persistent memory layer for AI coding tools.

### Features

- **Multi-Agent Support**: Claude Code, Cursor, and OpenCode adapters included
- **Non-Invasive Extraction**: Watches conversation logs via filesystem events — no API hooks, no browser extensions
- **SQLite + FTS5 Storage**: Local-only database with full-text search index. Your data never leaves your machine.
- **Semantic Search**: Find memories by natural language relevance, not just keyword matching
- **Terminal Injection**: `agentmem inject` displays a beautiful colored prompt box with your most relevant memories
- **WebUI Dashboard**: Built-in HTTP server with Chart.js visualization — agent distribution, memory types, timeline, tags
- **Zero External Dependencies**: Single 5MB binary. No Docker, no config files, no API keys.

### Installation

```bash
curl -sSL https://github.com/jiyujie2006/agentmem/releases/latest/download/install.sh | sh
```

### Quick Start

```bash
agentmem init
agentmem extract ~/.claude/
agentmem extract ~/.cursor/
agentmem inject
agentmem webui --port 8080
```

### Architecture

- Rust + SQLite (bundled)
- `notify` crate for filesystem watching
- Rule-based preference extraction (regex + keywords)
- Semantic scoring via multi-factor relevance algorithm
- Embedded HTML dashboard (Chart.js via CDN)

### Roadmap

- v1.1: Windsurf IDE support
- v1.2: Auto shell wrapper installation
- v2.0: Team memory sync (end-to-end encrypted)

---

**Full Changelog**: https://github.com/jiyujie2006/agentmem/compare/v0.1.0...v1.0.0
