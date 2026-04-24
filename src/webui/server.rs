use crate::storage::sqlite_store::SqliteStore;
use anyhow::Result;
use serde::Serialize;
use std::net::SocketAddr;
use std::path::PathBuf;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize)]
struct DashboardData {
    total_memories: usize,
    agent_distribution: Vec<AgentCount>,
    type_distribution: Vec<TypeCount>,
    timeline: Vec<TimelinePoint>,
    recent_memories: Vec<MemoryView>,
    top_tags: Vec<TagCount>,
}

#[derive(Serialize)]
struct AgentCount {
    agent: String,
    count: usize,
}

#[derive(Serialize)]
struct TypeCount {
    context_type: String,
    count: usize,
}

#[derive(Serialize)]
struct TimelinePoint {
    date: String,
    count: usize,
}

#[derive(Serialize)]
struct MemoryView {
    id: String,
    agent: String,
    context_type: String,
    content: String,
    confidence: f32,
    timestamp: String,
}

#[derive(Serialize)]
struct TagCount {
    tag: String,
    count: usize,
}

pub async fn run_webui(store: SqliteStore, port: u16) -> Result<()> {
    let db_path = store.db_path().clone();
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = TcpListener::bind(addr).await?;

    println!("🌐 AgentMem WebUI running at http://{}", addr);
    println!("   Press Ctrl+C to stop\n");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let db_path = db_path.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            if let Ok(n) = socket.read(&mut buf).await {
                if n == 0 { return; }
                let request = String::from_utf8_lossy(&buf[..n]);

                let response = if request.starts_with("GET /api/data") {
                    match build_api_response(&db_path) {
                        Ok(json) => format_http_response(200, "application/json", &json),
                        Err(e) => format_http_response(500, "text/plain", &format!("Error: {}", e)),
                    }
                } else {
                    format_http_response(200, "text/html", HTML_DASHBOARD)
                };

                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
    }
}

fn format_http_response(status: u16, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, content_type, body.len(), body
    )
}

fn build_api_response(db_path: &PathBuf) -> Result<String> {
    // 每次请求独立打开连接（简单并发安全）
    let store = SqliteStore::open(db_path)?;
    let data = build_dashboard_data(&store);
    Ok(serde_json::to_string(&data)?)
}

fn build_dashboard_data(store: &SqliteStore) -> DashboardData {
    let all = store.load_all().unwrap_or_default();
    let total = all.len();

    let mut agent_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut type_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut tag_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut timeline: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for m in &all {
        *agent_counts
            .entry(m.agent_type.to_string())
            .or_insert(0) += 1;
        *type_counts
            .entry(m.context_type.to_string())
            .or_insert(0) += 1;
        for tag in &m.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
        let date = m.timestamp.format("%Y-%m-%d").to_string();
        *timeline.entry(date).or_insert(0) += 1;
    }

    let mut agent_distribution: Vec<AgentCount> = agent_counts
        .into_iter()
        .map(|(agent, count)| AgentCount { agent, count })
        .collect();
    agent_distribution.sort_by(|a, b| b.count.cmp(&a.count));

    let mut type_distribution: Vec<TypeCount> = type_counts
        .into_iter()
        .map(|(context_type, count)| TypeCount { context_type, count })
        .collect();
    type_distribution.sort_by(|a, b| b.count.cmp(&a.count));

    let mut top_tags: Vec<TagCount> = tag_counts
        .into_iter()
        .map(|(tag, count)| TagCount { tag, count })
        .collect();
    top_tags.sort_by(|a, b| b.count.cmp(&a.count));
    top_tags.truncate(10);

    let mut timeline_vec: Vec<TimelinePoint> = timeline
        .into_iter()
        .map(|(date, count)| TimelinePoint { date, count })
        .collect();
    timeline_vec.sort_by(|a, b| a.date.cmp(&b.date));

    let recent_memories: Vec<MemoryView> = all
        .into_iter()
        .take(20)
        .map(|m| MemoryView {
            id: m.id,
            agent: m.agent_type.to_string(),
            context_type: m.context_type.to_string(),
            content: m.content,
            confidence: m.confidence,
            timestamp: m.timestamp.format("%Y-%m-%d %H:%M").to_string(),
        })
        .collect();

    DashboardData {
        total_memories: total,
        agent_distribution,
        type_distribution,
        timeline: timeline_vec,
        recent_memories,
        top_tags,
    }
}

const HTML_DASHBOARD: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>AgentMem Dashboard</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f172a;color:#e2e8f0;line-height:1.6}
.container{max-width:1200px;margin:0 auto;padding:2rem}
header{text-align:center;margin-bottom:2rem}
header h1{font-size:2.5rem;background:linear-gradient(90deg,#60a5fa,#a78bfa);-webkit-background-clip:text;-webkit-text-fill-color:transparent}
header p{color:#94a3b8;margin-top:.5rem}
.stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:1rem;margin-bottom:2rem}
.stat-card{background:#1e293b;border-radius:12px;padding:1.5rem;text-align:center;border:1px solid #334155}
.stat-card h3{font-size:2rem;color:#60a5fa}
.stat-card p{color:#94a3b8;font-size:.875rem;margin-top:.25rem}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(400px,1fr));gap:1.5rem;margin-bottom:2rem}
.card{background:#1e293b;border-radius:12px;padding:1.5rem;border:1px solid #334155}
.card h2{font-size:1.25rem;margin-bottom:1rem;color:#f1f5f9}
.chart-container{position:relative;height:250px}
.memory-list{max-height:400px;overflow-y:auto}
.memory-item{background:#0f172a;border-radius:8px;padding:1rem;margin-bottom:.75rem;border-left:3px solid #60a5fa}
.memory-item .meta{display:flex;gap:1rem;font-size:.75rem;color:#94a3b8;margin-bottom:.25rem}
.memory-item .content{font-size:.875rem;color:#e2e8f0}
.tag{display:inline-block;background:#334155;color:#cbd5e1;padding:.25rem .5rem;border-radius:4px;font-size:.75rem;margin:.25rem .25rem 0 0}
.tag-highlight{background:#60a5fa;color:#0f172a}
.timeline-container{height:200px}
</style>
</head>
<body>
<div class="container">
<header>
<h1>🧠 AgentMem Dashboard</h1>
<p>Your AI memory, visualized</p>
</header>

<div class="stats" id="stats"></div>

<div class="grid">
<div class="card">
<h2>📊 Agent Distribution</h2>
<div class="chart-container"><canvas id="agentChart"></canvas></div>
</div>
<div class="card">
<h2>📝 Memory Types</h2>
<div class="chart-container"><canvas id="typeChart"></canvas></div>
</div>
</div>

<div class="grid">
<div class="card">
<h2>📈 Memory Timeline</h2>
<div class="chart-container timeline-container"><canvas id="timelineChart"></canvas></div>
</div>
<div class="card">
<h2>🏷️ Top Tags</h2>
<div id="tags"></div>
</div>
</div>

<div class="card">
<h2>🕐 Recent Memories</h2>
<div class="memory-list" id="memories"></div>
</div>
</div>

<script>
async function load() {
  const res = await fetch('/api/data');
  const data = await res.json();

  document.getElementById('stats').innerHTML = `
    <div class="stat-card"><h3>${data.total_memories}</h3><p>Total Memories</p></div>
    <div class="stat-card"><h3>${data.agent_distribution.length}</h3><p>Active Agents</p></div>
    <div class="stat-card"><h3>${data.type_distribution.length}</h3><p>Memory Types</p></div>
    <div class="stat-card"><h3>${data.top_tags.length}</h3><p>Unique Tags</p></div>
  `;

  new Chart(document.getElementById('agentChart'), {
    type: 'doughnut',
    data: {
      labels: data.agent_distribution.map(d => d.agent),
      datasets: [{
        data: data.agent_distribution.map(d => d.count),
        backgroundColor: ['#60a5fa','#a78bfa','#34d399','#fbbf24','#f87171']
      }]
    },
    options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { labels: { color: '#e2e8f0' } } } }
  });

  new Chart(document.getElementById('typeChart'), {
    type: 'bar',
    data: {
      labels: data.type_distribution.map(d => d.context_type),
      datasets: [{
        label: 'Count',
        data: data.type_distribution.map(d => d.count),
        backgroundColor: '#60a5fa'
      }]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { display: false } },
      scales: { x: { ticks: { color: '#94a3b8' } }, y: { ticks: { color: '#94a3b8' }, beginAtZero: true } }
    }
  });

  new Chart(document.getElementById('timelineChart'), {
    type: 'line',
    data: {
      labels: data.timeline.map(d => d.date),
      datasets: [{
        label: 'Memories per day',
        data: data.timeline.map(d => d.count),
        borderColor: '#a78bfa',
        backgroundColor: 'rgba(167,139,250,0.1)',
        fill: true,
        tension: 0.3
      }]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { labels: { color: '#e2e8f0' } } },
      scales: { x: { ticks: { color: '#94a3b8' } }, y: { ticks: { color: '#94a3b8' }, beginAtZero: true } }
    }
  });

  document.getElementById('tags').innerHTML = data.top_tags.map(t =>
    `<span class="tag ${t.count > 2 ? 'tag-highlight' : ''}">${t.tag} (${t.count})</span>`
  ).join('');

  document.getElementById('memories').innerHTML = data.recent_memories.map(m => `
    <div class="memory-item" style="border-left-color:${m.agent === 'claude' ? '#60a5fa' : m.agent === 'cursor' ? '#a78bfa' : '#34d399'}">
      <div class="meta">
        <span>🤖 ${m.agent}</span>
        <span>📌 ${m.context_type}</span>
        <span>⭐ ${(m.confidence * 100).toFixed(0)}%</span>
        <span>🕐 ${m.timestamp}</span>
      </div>
      <div class="content">${m.content.substring(0, 200)}${m.content.length > 200 ? '...' : ''}</div>
    </div>
  `).join('');
}

load();
</script>
</body>
</html>"#;
