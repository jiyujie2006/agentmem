# 🚀 Release Day Checklist

> Based on AFFiNE / Tabularis / ScrapeGraphAI launch patterns

## Pre-Launch (T-24h)

- [ ] Update `Cargo.toml` version to `1.0.0` ✅
- [ ] Build release binary and verify size (< 10MB) ✅
- [ ] Verify `cargo check` produces zero warnings ✅
- [ ] Test install script: `cat install.sh | bash` (local mode)
- [ ] Test all CLI commands on clean machine
- [ ] Ensure README follows "1-line pitch + 10s demo + 1-line install" structure ✅
- [ ] Prepare Hacker News "Show HN" post ✅
- [ ] Prepare Reddit posts (r/programming, r/ClaudeCode, r/cursor) ✅
- [ ] Prepare Twitter/X thread (7 tweets) ✅
- [ ] Write GitHub Release Notes ✅
- [ ] Tag release: `git tag v1.0.0 && git push origin v1.0.0`
- [ ] Verify GitHub Actions builds all platform binaries

## T-2h

- [ ] Confirm GitHub Actions Release pipeline succeeded
- [ ] Verify all platform binaries are attached to Draft Release
- [ ] Test download links for Linux/macOS binaries
- [ ] Final README spell-check

## T-0h (Launch Moment)

- [ ] Click "Publish" on GitHub Release
- [ ] Verify README renders correctly on GitHub
- [ ] Verify install script points to correct release URL

## T+15min — Coordinated Blast

Post in this exact order (within 15 minutes):

1. [ ] **Hacker News**: Submit "Show HN" post
   - URL: https://news.ycombinator.com/submit
   - Title: "Show HN: I made AI coding agents remember across sessions"
   - Link: https://github.com/yourusername/agentmem

2. [ ] **Reddit r/programming**: Submit post
   - Title: "I built a 5MB tool that makes AI coding agents remember across sessions"

3. [ ] **Reddit r/ClaudeCode**: Submit post
   - Title: "Show HN: I made Claude Code remember across sessions"

4. [ ] **Reddit r/cursor**: Submit post
   - Title: "Cross-session memory for Cursor"

5. [ ] **Twitter/X**: Post thread (7 tweets)
   - Hook tweet with Asciinema GIF or terminal screenshot
   - Thread includes GitHub link in final tweet

## T+30min — Monitor & Engage

- [ ] Monitor HN comments. Reply within 10 minutes to questions.
- [ ] Monitor Reddit comments.
- [ ] Monitor Twitter mentions.
- [ ] Reply to early GitHub Issues with "Thanks for reporting, looking into this"

## T+2h — Assess Traction

- [ ] Check HN upvotes. Target: 10+ upvotes + 5+ comments = good hook
- [ ] Check GitHub Star count. Target: 50+ in first 2 hours
- [ ] If traction is low (< 5 HN upvotes), draft a follow-up technical blog post

## T+6h — Reddit Follow-up

- [ ] Reply to all Reddit comments
- [ ] Cross-post to r/rust if built with Rust (r/rust loves single-binary CLI tools)

## T+24h — Day 1 Stats

- [ ] Record GitHub Star count
- [ ] Record HN post rank
- [ ] Record Reddit upvotes
- [ ] If Star count > 200: high probability of GitHub Trending
- [ ] If Star count > 300: almost guaranteed GitHub Trending #1

## Day 2-7 — Sustain Momentum

- [ ] Day 2: Release v1.0.1 with a small bugfix (signals activity to GitHub algorithm)
- [ ] Day 3: Write technical blog "How we built cross-agent memory with SQLite"
- [ ] Day 5: Release v1.1.0 with one visible new feature
- [ ] Day 7: Submit to awesome-selfhosted, awesome-rust, awesome-cli-apps

## Week 2-4 — Content Marketing

- [ ] Week 2: "Comparing persistent memory strategies for Claude Code vs Cursor"
- [ ] Week 3: "From 0 to 1000 stars: what we learned building AgentMem"
- [ ] Week 4: "Why every AI coding agent needs persistent memory by 2027"

---

## Success Metrics

| Milestone | Target | Timeline |
|-----------|--------|----------|
| First 100 stars | ✅ | Day 1 |
| GitHub Trending | ✅ | Day 2-3 |
| First community PR | ✅ | Week 2 |
| 1000 stars | ✅ | Week 4-6 |
| Daily baseline growth | 50+ stars/day | After Week 4 |

---

## Emergency Contacts

- HN Mods: hn@ycombinator.com (only if post is flagged incorrectly)
- GitHub Support: https://support.github.com
