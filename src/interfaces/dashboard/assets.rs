pub const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Memory-chan Live Desk</title>
  <style>
    :root {
      color-scheme: light;
      --ink: #263049;
      --muted: #65728a;
      --line: rgba(38, 48, 73, .13);
      --panel: rgba(255, 255, 255, .82);
      --pink: #ff8fb3;
      --rose: #ff6f9d;
      --sky: #72c7ff;
      --aqua: #58dfcf;
      --lemon: #ffe07a;
      --violet: #b8a6ff;
    }

    * { box-sizing: border-box; }

    body {
      min-width: 320px;
      margin: 0;
      color: var(--ink);
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background:
        linear-gradient(135deg, rgba(114, 199, 255, .20), transparent 34%),
        linear-gradient(225deg, rgba(255, 143, 179, .22), transparent 36%),
        linear-gradient(180deg, #fffaff, #f7fdff 46%, #fffdf7);
    }

    body::before {
      content: "";
      position: fixed;
      inset: 0;
      pointer-events: none;
      background-image:
        radial-gradient(circle at 18px 18px, rgba(255, 143, 179, .18) 0 2px, transparent 2px),
        radial-gradient(circle at 54px 42px, rgba(114, 199, 255, .20) 0 2px, transparent 2px);
      background-size: 72px 72px;
      mask-image: linear-gradient(180deg, #000, transparent 72%);
    }

    button, pre { font: inherit; }

    .shell { min-height: 100vh; padding: 18px; }

    .stage {
      position: relative;
      overflow: hidden;
      min-height: calc(100vh - 36px);
      border: 1px solid rgba(255, 143, 179, .32);
      border-radius: 8px;
      background: rgba(255, 255, 255, .74);
      box-shadow: 0 28px 72px rgba(55, 65, 92, .15);
      backdrop-filter: blur(18px);
    }

    .stage::before {
      content: "";
      position: absolute;
      top: 104px;
      left: 320px;
      right: -40px;
      height: 10px;
      background: linear-gradient(90deg, rgba(88, 223, 207, .28), rgba(255, 224, 122, .30), rgba(255, 143, 179, .22));
      transform: rotate(-9deg);
      transform-origin: left center;
      pointer-events: none;
    }

    .top {
      position: relative;
      z-index: 1;
      display: grid;
      grid-template-columns: 316px minmax(0, 1fr) auto;
      gap: 16px;
      align-items: center;
      padding: 16px 18px;
      border-bottom: 1px solid var(--line);
    }

    .brand { display: flex; align-items: center; gap: 12px; min-width: 0; }

    .mascot {
      position: relative;
      flex: 0 0 auto;
      width: 58px;
      height: 62px;
      border-radius: 31px 31px 18px 18px;
      background: linear-gradient(145deg, var(--sky), var(--violet) 52%, var(--pink));
      box-shadow: inset 0 0 0 2px rgba(255, 255, 255, .78), 0 14px 28px rgba(184, 166, 255, .26);
    }

    .mascot::before {
      content: "";
      position: absolute;
      left: 9px;
      right: 9px;
      bottom: 8px;
      height: 32px;
      border-radius: 18px 18px 13px 13px;
      background:
        radial-gradient(circle at 13px 14px, #263049 0 2px, transparent 2.5px),
        radial-gradient(circle at 27px 14px, #263049 0 2px, transparent 2.5px),
        radial-gradient(circle at 20px 22px, rgba(255, 111, 157, .30) 0 7px, transparent 7px),
        #fff1e9;
    }

    .mascot::after {
      content: "";
      position: absolute;
      left: -4px;
      top: 8px;
      width: 66px;
      height: 15px;
      border-radius: 999px;
      background: linear-gradient(90deg, var(--lemon), #fff7ba, var(--lemon));
      transform: rotate(-8deg);
      box-shadow: 0 5px 13px rgba(255, 224, 122, .35);
    }

    .brand strong { display: block; font-size: 21px; line-height: 1.05; }
    .brand span { display: block; margin-top: 4px; color: var(--muted); font-size: 12px; }

    .ribbon {
      display: flex;
      align-items: center;
      gap: 10px;
      min-height: 48px;
      padding: 0 12px;
      overflow: hidden;
      border: 1px solid rgba(38, 48, 73, .12);
      border-radius: 8px;
      background: repeating-linear-gradient(115deg, rgba(114, 199, 255, .16) 0 13px, rgba(255, 255, 255, .72) 13px 27px);
    }

    .live {
      flex: 0 0 auto;
      border-radius: 999px;
      padding: 6px 10px;
      color: #fff;
      font-weight: 820;
      background: linear-gradient(135deg, var(--aqua), var(--sky));
      box-shadow: 0 10px 20px rgba(88, 223, 207, .22);
    }

    .bubble, .pill {
      border: 1px solid rgba(38, 48, 73, .11);
      border-radius: 999px;
      padding: 6px 9px;
      color: #526175;
      font-size: 12px;
      background: rgba(255, 255, 255, .72);
      white-space: nowrap;
    }

    .pills { display: flex; flex-wrap: wrap; gap: 7px; justify-content: end; }

    .grid {
      position: relative;
      z-index: 1;
      display: grid;
      grid-template-columns: 270px minmax(0, 1fr) 330px;
      min-height: 650px;
    }

    aside, main { padding: 18px; }
    .left { border-right: 1px solid var(--line); }
    .right { border-left: 1px solid var(--line); }

    .label {
      margin-bottom: 10px;
      color: #7a8698;
      font-size: 11px;
      letter-spacing: .1em;
      text-transform: uppercase;
    }

    .tab, .score, .card, .panel, .story {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--panel);
      box-shadow: 0 8px 20px rgba(55, 65, 92, .06);
    }

    .tab {
      display: flex;
      align-items: center;
      gap: 10px;
      min-height: 42px;
      padding: 0 12px;
      margin-bottom: 8px;
      color: #526175;
      font-weight: 650;
    }

    .tab.active {
      color: #8b3857;
      background: linear-gradient(90deg, rgba(255, 143, 179, .18), rgba(114, 199, 255, .14));
    }

    .star {
      width: 15px;
      height: 15px;
      clip-path: polygon(50% 0, 61% 34%, 98% 35%, 68% 55%, 80% 91%, 50% 70%, 20% 91%, 32% 55%, 2% 35%, 39% 34%);
      background: var(--lemon);
    }

    .score { padding: 13px; margin-top: 12px; }
    .score strong, .stat strong { font-size: 27px; line-height: 1; }
    .score span, .stat label { display: block; margin-top: 8px; color: var(--muted); font-size: 12px; }

    .meter {
      height: 9px;
      border-radius: 999px;
      margin-top: 10px;
      background: linear-gradient(90deg, var(--aqua), var(--sky), var(--lemon), var(--pink));
    }

    .stats { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 14px; }
    .stat { min-height: 78px; padding: 13px; }

    .story { padding: 14px; margin-bottom: 14px; }
    .steps { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 10px; }
    .step { min-height: 86px; padding: 12px; border: 1px solid rgba(38, 48, 73, .10); border-radius: 8px; background: rgba(255, 255, 255, .74); }
    .step b { display: block; margin-bottom: 12px; }

    .chip, .tag {
      display: inline-flex;
      max-width: 100%;
      border-radius: 999px;
      padding: 5px 8px;
      color: #23655e;
      font-size: 11px;
      font-weight: 760;
      background: rgba(88, 223, 207, .18);
    }

    .logs { display: grid; gap: 10px; }

    .log {
      display: grid;
      grid-template-columns: 92px minmax(120px, .7fr) 94px minmax(0, 1fr) 84px;
      gap: 12px;
      align-items: center;
      width: 100%;
      min-height: 60px;
      padding: 0 13px;
      color: inherit;
      text-align: left;
      font-size: 12px;
      cursor: pointer;
    }

    .log.hot {
      border-color: rgba(255, 143, 179, .36);
      background: linear-gradient(90deg, rgba(255, 143, 179, .14), rgba(255, 255, 255, .88) 42%);
    }

    .empty {
      min-height: 160px;
      display: grid;
      place-items: center;
      color: var(--muted);
    }

    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }

    .panel { padding: 13px; margin-bottom: 12px; }
    .panel h4 { margin: 0 0 12px; font-size: 16px; }

    .kv {
      display: grid;
      grid-template-columns: 88px minmax(0, 1fr);
      gap: 8px;
      padding: 7px 0;
      border-top: 1px solid rgba(38, 48, 73, .07);
      font-size: 12px;
    }

    .kv:first-of-type { border-top: 0; }
    .kv span:first-child { color: var(--muted); }

    pre {
      margin: 0;
      max-height: 280px;
      padding: 11px;
      overflow: auto;
      border-radius: 8px;
      color: #eaf0fb;
      font-size: 11px;
      background: #263244;
    }

    @media (prefers-reduced-motion: no-preference) {
      .mascot { animation: floaty 3.2s ease-in-out infinite; }
      .live { animation: pulse 1.8s ease-in-out infinite; }
      @keyframes floaty { 0%, 100% { transform: translateY(0) rotate(-2deg); } 50% { transform: translateY(-7px) rotate(2deg); } }
      @keyframes pulse { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.05); } }
    }

    @media (max-width: 1180px) {
      .top, .grid { grid-template-columns: 1fr; }
      .pills { justify-content: start; }
      .left, .right { border: 0; }
      .stats { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .steps { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .log { grid-template-columns: 78px 1fr 88px; }
      .log div:nth-child(4), .log div:nth-child(5) { display: none; }
    }

    @media (max-width: 640px) {
      .shell { padding: 10px; }
      aside, main, .top { padding: 12px; }
      .stats, .steps { grid-template-columns: 1fr; }
      .bubble { white-space: normal; }
      .brand strong { font-size: 18px; }
    }
  </style>
</head>
<body>
  <div class="shell">
    <div class="stage">
      <header class="top">
        <div class="brand">
          <div class="mascot" aria-hidden="true"></div>
          <div>
            <strong>Memory-chan Live Desk</strong>
            <span>agent-llm-mm operation diary</span>
          </div>
        </div>
        <div class="ribbon">
          <span class="live">LIVE</span>
          <span class="bubble" id="live-operation">waiting for events</span>
          <span class="bubble">read-only observability</span>
        </div>
        <div class="pills">
          <span class="pill">stdio MCP</span>
          <span class="pill">production dashboard</span>
          <span class="pill">stdout protected</span>
        </div>
      </header>
      <div class="grid">
        <aside class="left">
          <div class="label">Live channels</div>
          <div class="tab active"><span class="star"></span>Operation Diary</div>
          <div class="tab"><span class="star"></span>Reflection Magic</div>
          <div class="tab"><span class="star"></span>Decision Cards</div>
          <div class="tab"><span class="star"></span>Snapshot Album</div>
          <div class="tab"><span class="star"></span>Doctor Check</div>
          <div class="score"><strong id="total-score">0</strong><span>operations this session</span><div class="meter"></div></div>
          <div class="score"><strong id="reflection-score">0</strong><span>reflection events</span><div class="meter"></div></div>
          <div class="score"><strong id="failed-score">0</strong><span>failed operations</span><div class="meter"></div></div>
        </aside>
        <main>
          <div class="stats">
            <div class="card stat"><label>MCP tools</label><strong id="tool-count">0</strong></div>
            <div class="card stat"><label>events</label><strong id="event-count">0</strong></div>
            <div class="card stat"><label>reflections</label><strong id="reflection-count">0</strong></div>
            <div class="card stat"><label>status</label><strong id="status-count">ok</strong></div>
          </div>
          <section class="story">
            <div class="label">Operation story</div>
            <div class="steps">
              <div class="step"><b>ingest</b><span class="chip">watching</span></div>
              <div class="step"><b>trigger</b><span class="chip">watching</span></div>
              <div class="step"><b>proposal</b><span class="chip">governed</span></div>
              <div class="step"><b>write path</b><span class="chip">run_reflection</span></div>
              <div class="step"><b>snapshot</b><span class="chip">watching</span></div>
            </div>
          </section>
          <section class="logs" id="logs"><div class="card empty">waiting for runtime events</div></section>
        </main>
        <aside class="right">
          <div class="panel">
            <h4>Selected operation</h4>
            <div id="selected-operation"><div class="kv"><span>status</span><span>waiting</span></div></div>
          </div>
          <div class="panel">
            <h4>Payload inspector</h4>
            <pre id="payload">{}</pre>
          </div>
          <div class="panel">
            <h4>Runtime note</h4>
            <div class="kv"><span>mode</span><span>production dashboard</span></div>
            <div class="kv"><span>actions</span><span>read only</span></div>
            <div class="kv"><span>stdout</span><span>MCP only</span></div>
          </div>
        </aside>
      </div>
    </div>
  </div>
  <script>
    const api = (path) => path;

    function escapeHtml(value) {
      return String(value ?? "").replace(/[&<>"']/g, (char) => ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;"
      }[char]));
    }

    async function refresh() {
      const [summary, events] = await Promise.all([
        fetch(api("api/summary")).then((response) => response.json()),
        fetch(api("api/events?limit=25")).then((response) => response.json())
      ]);

      document.getElementById("total-score").textContent = summary.total_events;
      document.getElementById("reflection-score").textContent = summary.reflection_events;
      document.getElementById("failed-score").textContent = summary.failed_events;
      document.getElementById("tool-count").textContent = summary.tool_events;
      document.getElementById("event-count").textContent = summary.total_events;
      document.getElementById("reflection-count").textContent = summary.reflection_events;
      document.getElementById("status-count").textContent = summary.failed_events > 0 ? "check" : "ok";

      const logs = document.getElementById("logs");
      if (!events.length) {
        logs.innerHTML = '<div class="card empty">waiting for runtime events</div>';
        return;
      }

      logs.innerHTML = events.map((event, index) => `
        <button class="card log ${index === events.length - 1 ? "hot" : ""}" data-index="${index}">
          <div class="mono">${escapeHtml(new Date(event.timestamp).toLocaleTimeString())}</div>
          <div>${escapeHtml(event.operation)}</div>
          <div><span class="tag">${escapeHtml(event.kind)}</span></div>
          <div>${escapeHtml(event.summary)}</div>
          <div>${escapeHtml(event.status)}</div>
        </button>
      `).join("");

      logs.querySelectorAll(".log").forEach((node) => {
        node.addEventListener("click", () => selectEvent(events[Number(node.dataset.index)]));
      });
      selectEvent(events[events.length - 1]);
    }

    function selectEvent(event) {
      document.getElementById("live-operation").textContent = event.operation;
      document.getElementById("selected-operation").innerHTML = `
        <div class="kv"><span>id</span><span class="mono">${escapeHtml(event.id)}</span></div>
        <div class="kv"><span>kind</span><span>${escapeHtml(event.kind)}</span></div>
        <div class="kv"><span>status</span><span>${escapeHtml(event.status)}</span></div>
        <div class="kv"><span>namespace</span><span>${escapeHtml(event.namespace || "-")}</span></div>
      `;
      document.getElementById("payload").textContent = JSON.stringify(event.payload || {}, null, 2);
    }

    refresh().catch(() => {});
    setInterval(() => refresh().catch(() => {}), 2500);
  </script>
</body>
</html>"#;
