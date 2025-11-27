mod universe;
mod supervisor;
mod logging;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::logging::{subscribe, LogEntry};
use crate::supervisor::user_supervisor::UserSupervisor;
use crate::universe::{UniverseCommand, UniverseEvent};

#[derive(Clone)]
struct AppState {
    supervisor: Arc<Mutex<UserSupervisor>>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

// Simple single-page UI served at "/"
const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>Universe Supervisor</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    :root {
      font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      background: #0f172a;
      color: #e5e7eb;
    }
    body {
      margin: 0;
      padding: 0;
    }
    .app {
      max-width: 1100px;
      margin: 0 auto;
      padding: 16px;
    }
    h1 {
      font-size: 1.8rem;
      margin-bottom: 0.5rem;
    }
    .subtitle {
      color: #9ca3af;
      margin-bottom: 1.5rem;
    }
    .row {
      display: flex;
      flex-wrap: wrap;
      gap: 16px;
    }
    .card {
      background: #020617;
      border-radius: 12px;
      padding: 16px;
      box-shadow: 0 10px 40px rgba(0,0,0,0.6);
      border: 1px solid #1f2937;
      flex: 1 1 0;
      min-width: 260px;
    }
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.75rem;
    }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 2px 8px;
      border-radius: 999px;
      font-size: 0.8rem;
      border: 1px solid #374151;
    }
    .badge-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
    }
    .badge-dot.green { background: #22c55e; }
    .badge-dot.red { background: #ef4444; }

    label {
      display: block;
      font-size: 0.85rem;
      margin-bottom: 4px;
      color: #9ca3af;
    }
    input[type='text'],
    input[type='number'] {
      width: 100%;
      padding: 6px 8px;
      border-radius: 8px;
      border: 1px solid #374151;
      background: #020617;
      color: #e5e7eb;
      outline: none;
    }
    input[type='text']:focus,
    input[type='number']:focus {
      border-color: #3b82f6;
    }

    button {
      border-radius: 999px;
      border: 1px solid #374151;
      padding: 6px 12px;
      font-size: 0.85rem;
      cursor: pointer;
      background: #111827;
      color: #e5e7eb;
      transition: background 0.15s ease, transform 0.05s ease, border-color 0.15s ease;
    }
    button.primary {
      background: #3b82f6;
      border-color: #2563eb;
    }
    button.danger {
      background: #b91c1c;
      border-color: #7f1d1d;
    }
    button.small {
      padding: 4px 10px;
      font-size: 0.8rem;
    }
    button:hover {
      background: #1f2937;
      transform: translateY(-1px);
    }
    button.primary:hover {
      background: #2563eb;
    }
    button.danger:hover {
      background: #991b1b;
    }

    .universe-list {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .universe-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 10px;
      border-radius: 10px;
      background: #020617;
      border: 1px solid #1f2937;
    }
    .universe-name {
      font-weight: 500;
    }
    .universe-actions {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
      justify-content: flex-end;
    }

    .logs-container {
      max-height: 420px;
      overflow: auto;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
      font-size: 0.85rem;
      padding: 8px;
      background: #020617;
      border-radius: 10px;
      border: 1px solid #1f2937;
    }
    .log-line {
      white-space: pre-wrap;
      margin: 0;
      padding: 2px 0;
    }
    .log-info { color: #9ca3af; }
    .log-universe { color: #38bdf8; }
    .log-relationship { color: #a855f7; }
    .log-useraction { color: #22c55e; }

    .logs-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
    }
    .logs-controls {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 0.8rem;
      color: #9ca3af;
    }
    .logs-controls input {
      width: 70px;
    }

    @media (max-width: 768px) {
      .row {
        flex-direction: column;
      }
    }
  </style>
</head>
<body>
  <div class="app">
    <h1>Universe Supervisor</h1>
    <div class="subtitle">
      Manage universes, send events, and watch the log stream — all backed by your Rust async simulation.
    </div>

    <div class="row">
      <div class="card">
        <div class="card-header">
          <div>Server</div>
          <div class="badge" id="health-badge">
            <span class="badge-dot red"></span>
            <span id="health-text">Unknown</span>
          </div>
        </div>
        <button class="primary" onclick="refreshAll()">Refresh now</button>
      </div>

      <div class="card">
        <div class="card-header">
          <div>Create Universe</div>
        </div>
        <div>
          <label for="new-universe-name">Name</label>
          <input id="new-universe-name" type="text" placeholder="e.g. Alpha" />
        </div>
        <div style="margin-top: 8px;">
          <button class="primary" onclick="createUniverse()">Create</button>
        </div>
      </div>
    </div>

    <div class="row" style="margin-top: 16px;">
      <div class="card">
        <div class="card-header">
          <div>Universes</div>
          <button class="small" onclick="loadUniverses()">Refresh</button>
        </div>
        <div id="universes-container" class="universe-list">
          <div style="color:#6b7280;">No universes yet. Create one above.</div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <div>Universe Events</div>
        </div>
        <div>
          <label for="event-universe-name">Universe name</label>
          <input id="event-universe-name" type="text" placeholder="Match an existing universe name" />
        </div>
        <div style="margin-top: 8px;">
          <label for="event-strength">Strength / amount</label>
          <input id="event-strength" type="number" value="10" />
        </div>
        <div style="margin-top: 8px; display:flex; flex-wrap:wrap; gap:6px;">
          <button class="small" onclick="sendEvent('shatter')">Shatter</button>
          <button class="small" onclick="sendEvent('heal')">Heal</button>
          <button class="small danger" onclick="sendEvent('crash')">Crash</button>
        </div>
      </div>
    </div>

    <div class="card" style="margin-top: 16px;">
      <div class="logs-header">
        <div>Logs</div>
        <div class="logs-controls">
          <span>Last</span>
          <input id="logs-limit" type="number" value="200" min="1" />
          <span>lines</span>
          <button class="small" onclick="loadLogs()">Refresh</button>
        </div>
      </div>
      <div id="logs-container" class="logs-container">
        <div style="color:#6b7280;">Logs will appear here.</div>
      </div>
    </div>
  </div>

  <script>
    const API_BASE = '';

    async function fetchJson(path, options = {}) {
      const hasBody = options.body !== undefined;
    
      const headers = {};
      if (hasBody) {
        headers['Content-Type'] = 'application/json';
      }
    
      const res = await fetch(API_BASE + path, {
        ...options,
        headers,
      });
    
      if (!res.ok) {
        const text = await res.text().catch(() => '');
        throw new Error('HTTP ' + res.status + ' ' + text);
      }
    
      // If there's no JSON body, just return null
      const contentType = res.headers.get('content-type') || "";
      if (!contentType.includes('application/json')) {
        return null;
      }
    
      return res.json();
    }


    async function pingHealth() {
      const badge = document.getElementById('health-badge');
      const text = document.getElementById('health-text');
      try {
        const res = await fetch(API_BASE + '/health');
        if (res.ok) {
          text.textContent = 'Healthy';
          badge.querySelector('.badge-dot').classList.remove('red');
          badge.querySelector('.badge-dot').classList.add('green');
        } else {
          text.textContent = 'Error';
          badge.querySelector('.badge-dot').classList.remove('green');
          badge.querySelector('.badge-dot').classList.add('red');
        }
      } catch (e) {
        text.textContent = 'Offline';
        badge.querySelector('.badge-dot').classList.remove('green');
        badge.querySelector('.badge-dot').classList.add('red');
      }
    }

    async function loadUniverses() {
      const container = document.getElementById('universes-container');
      container.innerHTML = '<div style="color:#6b7280;">Loading...</div>';
      try {
        const data = await fetchJson('/universes');
        console.log("UNIVERSE LIST RESPONSE:", data);
        const universes = data.universes || [];
        if (universes.length === 0) {
          container.innerHTML = '<div style="color:#6b7280;">No universes yet. Create one above.</div>';
          return;
        }

        container.innerHTML = '';
        universes.forEach(name => {
          console.log("RENDERING UNIVERSE:", name);
          const row = document.createElement('div');
          row.className = 'universe-item';

          const left = document.createElement('div');
          left.className = 'universe-name';
          left.textContent = name;

          const right = document.createElement('div');
          right.className = 'universe-actions';

          const makeBtn = (label, cls, handler) => {
            const btn = document.createElement('button');
            if (cls) {
              cls.split(/\s+/).forEach(c => btn.classList.add(c));
            }
            btn.textContent = label;
            btn.onclick = handler;
            return btn;
          };

          right.appendChild(makeBtn('Resume', 'small', () => universeCommand(name, 'resume')));
          right.appendChild(makeBtn('Pause', 'small', () => universeCommand(name, 'pause')));
          right.appendChild(makeBtn('Collapse', 'small', () => universeCommand(name, 'collapse')));
          right.appendChild(makeBtn('Shatter', 'small', () => quickEvent(name, 'shatter')));
          right.appendChild(makeBtn('Heal', 'small', () => quickEvent(name, 'heal')));
          right.appendChild(makeBtn('Crash', 'small danger', () => quickEvent(name, 'crash')));

          row.appendChild(left);
          row.appendChild(right);
          container.appendChild(row);
        });
      } catch (e) {
        container.innerHTML = '<div style="color:#f97316;">Failed to load universes: ' + e.message + '</div>';
      }
    }

    async function createUniverse() {
      const input = document.getElementById('new-universe-name');
      const name = input.value.trim();
      if (!name) {
        alert('Please enter a universe name');
        return;
      }
      try {
        await fetchJson('/universes', {
          method: 'POST',
          body: JSON.stringify({ name }),
        });
        input.value = '';
        await loadUniverses();
      } catch (e) {
        alert('Failed to create universe: ' + e.message);
      }
    }

    async function universeCommand(name, action) {
      let path;
      switch (action) {
        case 'resume':
          path = `/universes/${encodeURIComponent(name)}/resume`;
          break;
        case 'pause':
          path = `/universes/${encodeURIComponent(name)}/pause`;
          break;
        case 'collapse':
          path = `/universes/${encodeURIComponent(name)}/collapse`;
          break;
        default:
          return;
      }
      try {
        await fetchJson(path, { method: 'POST' });
      } catch (e) {
        alert('Failed to send command: ' + e.message);
      }
    }

    async function quickEvent(name, kind) {
      const strengthInput = document.getElementById('event-strength');
      const strength = parseInt(strengthInput.value, 10) || 10;
      await sendEvent(kind, name, strength, false);
    }

    async function sendEvent(kind, explicitName, explicitStrength, fromForm = true) {
      let name = explicitName;
      let strength = explicitStrength;
      if (fromForm) {
        const nameInput = document.getElementById('event-universe-name');
        const strengthInput = document.getElementById('event-strength');
        name = (nameInput.value || '').trim();
        strength = parseInt(strengthInput.value, 10) || 10;
      }
      if (!name) {
        alert('Please enter a universe name');
        return;
      }

      let path;
      let payload = {};
      if (kind === 'shatter') {
        path = `/universes/${encodeURIComponent(name)}/events/shatter`;
        payload = { strength };
      } else if (kind === 'heal') {
        path = `/universes/${encodeURIComponent(name)}/events/heal`;
        payload = { strength };
      } else if (kind === 'crash') {
        path = `/universes/${encodeURIComponent(name)}/events/crash`;
      } else {
        return;
      }

      try {
        await fetchJson(path, {
          method: 'POST',
          body: Object.keys(payload).length ? JSON.stringify(payload) : undefined,
        });
      } catch (e) {
        alert('Failed to send event: ' + e.message);
      }
    }

    async function loadLogs() {
      const container = document.getElementById('logs-container');
      const limitInput = document.getElementById('logs-limit');
      const limit = parseInt(limitInput.value, 10) || 200;

      try {
        const logs = await fetchJson('/logs?limit=' + encodeURIComponent(limit));
        container.innerHTML = '';
        if (!Array.isArray(logs) || logs.length === 0) {
          container.innerHTML = '<div style="color:#6b7280;">No logs yet.</div>';
          return;
        }

        logs.forEach(entry => {
          const p = document.createElement('div');
          p.className = 'log-line';
          const level = (entry.level || '').toString().toLowerCase().replace(/\s+/g, '');
            if (level === 'info') {
              p.classList.add('log-info');
            } else if (level === 'universe') {
              p.classList.add('log-universe');
            } else if (level === 'relationship') {
              p.classList.add('log-relationship');
            } else if (level === 'useraction') {
              p.classList.add('log-useraction');
            }
          p.textContent = entry.message || '';
          container.appendChild(p);
        });

        container.scrollTop = container.scrollHeight;
      } catch (e) {
        container.innerHTML = '<div style="color:#f97316;">Failed to load logs: ' + e.message + '</div>';
      }
    }

    function refreshAll() {
      pingHealth();
      loadUniverses();
      loadLogs();
    }

    // Initial load
    refreshAll();
    // Auto-refresh
    setInterval(pingHealth, 5000);
    setInterval(loadUniverses, 7000);
    setInterval(loadLogs, 3000);
  </script>
</body>
</html>
"#;

#[tokio::main]
async fn main() {
    // Shared UserSupervisor
    let supervisor = Arc::new(Mutex::new(UserSupervisor::new()));

    // In-memory log buffer (for /logs endpoint)
    let logs: Arc<Mutex<Vec<LogEntry>>> = Arc::new(Mutex::new(Vec::new()));

    // Background task: process universe events
    {
        let supervisor_for_loop = supervisor.clone();
        tokio::spawn(async move {
            loop {
                {
                    let mut guard = supervisor_for_loop.lock().await;
                    guard.process_universe_events().await;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }

    // Background task: collect logs
    {
        let mut log_rx = subscribe();
        let logs_for_loop = logs.clone();
        tokio::spawn(async move {
            while let Ok(entry) = log_rx.recv().await {
                let mut buf = logs_for_loop.lock().await;
                buf.push(entry);
                const MAX_LOGS: usize = 1000;
                if buf.len() > MAX_LOGS {
                    let excess = buf.len() - MAX_LOGS;
                    buf.drain(0..excess);
                }
            }
        });
    }

    let state = AppState { supervisor, logs };

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/universes", get(list_universes).post(create_universe))
        .route("/universes/{name}/resume", post(resume_universe))
        .route("/universes/{name}/pause", post(pause_universe))
        .route("/universes/{name}/collapse", post(collapse_universe))
        .route("/universes/{name}/events/shatter", post(shatter_universe))
        .route("/universes/{name}/events/heal", post(heal_universe))
        .route("/universes/{name}/events/crash", post(crash_universe))
        .route("/logs", get(get_logs))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    println!("Server running on http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Simple health endpoint
async fn health() -> &'static str {
    "OK"
}

#[derive(Serialize)]
struct UniverseListResponse {
    universes: Vec<String>,
}

async fn list_universes(
    State(state): State<AppState>,
) -> Json<UniverseListResponse> {
    let supervisor = state.supervisor.lock().await;
    let names = supervisor
        .get_list_universes()
        .iter()
        .map(|name| (*name).clone())
        .collect();

    Json(UniverseListResponse { universes: names })
}

#[derive(Deserialize)]
struct CreateUniverseRequest {
    name: String,
}

async fn create_universe(
    State(state): State<AppState>,
    Json(payload): Json<CreateUniverseRequest>,
) -> impl axum::response::IntoResponse {
    let name = payload.name.trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "name must not be empty").into_response();
    }

    let mut supervisor = state.supervisor.lock().await;
    supervisor.new_universe(name.to_string()).await;

    StatusCode::CREATED.into_response()
}

/// Helper to send a command to a universe by name
async fn send_universe_command(
    state: &AppState,
    universe_name: String,
    command: UniverseCommand,
) -> StatusCode {
    let supervisor = state.supervisor.lock().await;
    supervisor
        .supervisor
        .send_universe_command(universe_name, command)
        .await;
    StatusCode::OK
}

async fn resume_universe(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> StatusCode {
    send_universe_command(&state, name, UniverseCommand::Start).await
}

async fn pause_universe(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> StatusCode {
    send_universe_command(&state, name, UniverseCommand::Stop).await
}

async fn collapse_universe(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> StatusCode {
    send_universe_command(&state, name, UniverseCommand::Shutdown).await
}

#[derive(Deserialize)]
struct StrengthPayload {
    strength: i32,
}

async fn shatter_universe(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<StrengthPayload>,
) -> StatusCode {
    let strength = payload.strength;
    send_universe_command(
        &state,
        name,
        UniverseCommand::InjectEvent(UniverseEvent::Shatter(strength)),
    )
        .await
}

async fn heal_universe(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<StrengthPayload>,
) -> StatusCode {
    let amount = payload.strength;
    send_universe_command(
        &state,
        name,
        UniverseCommand::InjectEvent(UniverseEvent::Heal(amount)),
    )
        .await
}

async fn crash_universe(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> StatusCode {
    send_universe_command(
        &state,
        name,
        UniverseCommand::InjectEvent(UniverseEvent::Crash),
    )
        .await
}

#[derive(Deserialize)]
struct LogsQuery {
    limit: Option<usize>,
}

async fn get_logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Json<Vec<LogEntry>> {
    let limit = query.limit.unwrap_or(100);

    let logs = state.logs.lock().await;
    let len = logs.len();
    let start = len.saturating_sub(limit);
    let slice = logs[start..].to_vec();

    Json(slice)
}
