pub const MEMORY_CHAN_HERO_PNG: &[u8] = include_bytes!("static/memory_chan_hero.png");
pub const MEMORY_CHAN_SIDEBAR_PNG: &[u8] = include_bytes!("static/memory_chan_sidebar.png");

pub const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Memory-chan Live Cockpit</title>
  <style>
    :root {
      color-scheme: light;
      --ink: #22304a;
      --muted: #60708d;
      --soft: #edf7ff;
      --line: rgba(64, 86, 126, .16);
      --panel: rgba(255, 255, 255, .88);
      --panel-soft: rgba(255, 255, 255, .72);
      --pink: #ff6fa0;
      --rose: #ff8faf;
      --aqua: #1fc8c6;
      --sky: #2c99ee;
      --blue: #337fe8;
      --violet: #9b77ee;
      --lemon: #ffc85a;
      --green: #5ccf6d;
      --orange: #ff9e47;
      --shadow: 0 18px 40px rgba(58, 76, 112, .10);
    }

    * { box-sizing: border-box; }

    html, body { min-width: 320px; min-height: 100%; }

    body {
      margin: 0;
      color: var(--ink);
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background:
        radial-gradient(circle at 12% 8%, rgba(58, 201, 215, .14), transparent 28%),
        radial-gradient(circle at 86% 10%, rgba(255, 111, 160, .13), transparent 28%),
        linear-gradient(180deg, #fafdff, #f7fbff);
    }

    button, pre { font: inherit; }

    .cockpit {
      min-height: 100vh;
      display: grid;
      grid-template-columns: 300px minmax(0, 1fr);
      border: 1px solid rgba(79, 99, 135, .20);
      border-radius: 18px;
      overflow: hidden;
      background:
        linear-gradient(115deg, rgba(255, 255, 255, .95), rgba(241, 251, 255, .82)),
        radial-gradient(circle at 78% 16%, rgba(255, 111, 160, .12), transparent 30%);
      box-shadow: 0 22px 70px rgba(58, 76, 112, .12);
    }

    .sidebar {
      position: relative;
      display: flex;
      flex-direction: column;
      min-width: 0;
      border-right: 1px solid var(--line);
      background: rgba(255, 255, 255, .72);
      backdrop-filter: blur(18px);
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      min-height: 98px;
      padding: 18px 22px;
      border-bottom: 1px solid var(--line);
    }

    .brand-mark {
      position: relative;
      flex: 0 0 auto;
      width: 64px;
      height: 64px;
      border: 1px solid rgba(64, 86, 126, .18);
      border-radius: 18px;
      background: #fff;
      box-shadow: var(--shadow);
    }

    .brand-mark::before {
      content: "";
      position: absolute;
      inset: 8px;
      border-radius: 15px 15px 12px 12px;
      background:
        radial-gradient(circle at 17px 21px, #22304a 0 2.4px, transparent 2.8px),
        radial-gradient(circle at 32px 21px, #22304a 0 2.4px, transparent 2.8px),
        linear-gradient(145deg, #56dbdc, #53d3c0 50%, #ff9cb0);
    }

    .brand strong {
      display: block;
      font-size: 23px;
      line-height: 1.05;
      letter-spacing: 0;
    }

    .brand span {
      display: block;
      margin-top: 4px;
      color: #65708a;
      font-size: 20px;
      line-height: 1;
    }

    .brand span b {
      color: var(--pink);
      font-weight: 700;
    }

    .views {
      margin: 0;
      padding: 18px 16px;
      list-style: none;
    }

    .label {
      margin: 0 0 12px;
      color: #6e7a94;
      font-size: 13px;
      font-weight: 760;
      letter-spacing: .08em;
      text-transform: uppercase;
    }

    .view {
      position: relative;
      display: flex;
      align-items: center;
      gap: 12px;
      min-height: 47px;
      margin: 0 0 8px;
      padding: 0 16px;
      border: 1px solid transparent;
      border-radius: 12px;
      color: #475978;
      font-size: 16px;
      font-weight: 650;
    }

    .view.active {
      color: #117c7b;
      border-color: rgba(31, 200, 198, .54);
      background: linear-gradient(90deg, rgba(31, 200, 198, .18), rgba(255, 255, 255, .72));
      box-shadow: inset 0 0 0 1px rgba(255, 255, 255, .70);
    }

    .view-icon {
      width: 25px;
      height: 25px;
      display: grid;
      place-items: center;
      color: #fff;
      font-size: 14px;
      font-weight: 900;
      border-radius: 8px;
      background: var(--sky);
      box-shadow: 0 8px 18px rgba(44, 153, 238, .20);
    }

    .view:nth-child(3) .view-icon { background: var(--pink); }
    .view:nth-child(4) .view-icon { background: var(--sky); }
    .view:nth-child(5) .view-icon { background: var(--violet); }
    .view:nth-child(6) .view-icon { background: #72cf4f; }

    .side-spark,
    .side-spark::before,
    .side-spark::after {
      position: absolute;
      width: 18px;
      height: 18px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: var(--lemon);
    }

    .side-spark {
      right: 26px;
      top: 116px;
    }

    .side-spark::before {
      content: "";
      right: -24px;
      top: 7px;
      width: 13px;
      height: 13px;
      background: var(--pink);
    }

    .side-spark::after {
      content: "";
      right: -12px;
      top: 270px;
      width: 24px;
      height: 24px;
      background: #ffd98a;
    }

    .side-cards {
      display: grid;
      gap: 14px;
      padding: 0 12px 12px;
    }

    .side-card {
      min-height: 126px;
      padding: 18px 20px;
      border: 1px solid var(--line);
      border-radius: 14px;
      background: rgba(255, 255, 255, .86);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .side-card strong {
      display: block;
      color: #21304b;
      font-size: 36px;
      line-height: 1;
    }

    .side-card span {
      display: block;
      margin-top: 8px;
      color: #60708d;
      font-size: 13px;
      font-weight: 620;
    }

    .mini-chart {
      height: 28px;
      margin-top: 14px;
      background:
        linear-gradient(90deg, var(--aqua), var(--sky), var(--rose), var(--lemon)) left 14px / 62% 9px no-repeat,
        repeating-linear-gradient(90deg, rgba(255, 111, 160, .34) 0 6px, transparent 6px 13px) right bottom / 78px 24px no-repeat;
      border-radius: 999px;
    }

    .side-card:nth-child(2) .mini-chart {
      background:
        linear-gradient(90deg, #f65367, #ffc653) left 14px / 62% 9px no-repeat,
        repeating-linear-gradient(90deg, rgba(255, 158, 71, .54) 0 6px, transparent 6px 13px) right bottom / 78px 30px no-repeat;
    }

    .side-card:nth-child(3) .mini-chart {
      background:
        linear-gradient(90deg, #72df9a, #50c95c) left 14px / 62% 9px no-repeat,
        repeating-linear-gradient(90deg, rgba(92, 207, 109, .48) 0 7px, transparent 7px 14px) right bottom / 78px 40px no-repeat;
    }

    .sidebar-sticker {
      margin-top: auto;
      padding: 0 18px 14px;
    }

    .sidebar-sticker img {
      display: block;
      width: 100%;
      height: 138px;
      object-fit: cover;
      object-position: 50% 28%;
      border-radius: 16px;
      mix-blend-mode: multiply;
    }

    .workspace {
      min-width: 0;
      padding: 22px 10px 0;
    }

    .topbar {
      display: grid;
      grid-template-columns: minmax(0, 830px) auto;
      gap: 18px;
      align-items: center;
      padding: 0 24px 18px 32px;
    }

    .live-strip {
      display: flex;
      align-items: center;
      min-height: 55px;
      overflow: hidden;
      border: 1px solid var(--line);
      border-radius: 15px;
      background: rgba(255, 255, 255, .86);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .07);
    }

    .strip-item {
      display: inline-flex;
      align-items: center;
      gap: 11px;
      min-height: 30px;
      padding: 0 25px;
      color: #34435d;
      font-size: 16px;
      font-weight: 680;
      border-left: 1px dashed rgba(64, 86, 126, .24);
      white-space: nowrap;
    }

    .strip-item:first-child {
      border-left: 0;
    }

    .dot {
      width: 10px;
      height: 10px;
      border-radius: 999px;
      background: var(--aqua);
      box-shadow: 0 0 0 4px rgba(31, 200, 198, .14);
    }

    .dot.live-dot { background: var(--pink); box-shadow: 0 0 0 4px rgba(255, 111, 160, .18); }
    .dot.warn-dot { background: var(--lemon); box-shadow: 0 0 0 4px rgba(255, 200, 90, .16); }
    .dot.path-dot { background: var(--peach, #ffb17e); box-shadow: 0 0 0 4px rgba(255, 177, 126, .16); }

    .live-pill {
      min-height: 34px;
      padding: 0 20px;
      border: 1px solid rgba(255, 111, 160, .36);
      border-radius: 10px;
      color: var(--pink);
      background: rgba(255, 111, 160, .08);
    }

    .top-pills {
      display: flex;
      justify-content: flex-end;
      gap: 14px;
      min-width: 0;
    }

    .pill {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 38px;
      padding: 0 24px;
      border: 1px solid rgba(108, 122, 162, .22);
      border-radius: 12px;
      color: #7a61bf;
      font-weight: 660;
      background: rgba(255, 255, 255, .76);
      white-space: nowrap;
    }

    .pill.primary {
      color: #fff;
      border-color: rgba(31, 200, 198, .70);
      background: linear-gradient(135deg, #14bfc0, #50d8cf);
      box-shadow: inset 0 0 0 2px rgba(255, 255, 255, .48), 0 10px 20px rgba(31, 200, 198, .18);
    }

    .pill.danger {
      color: var(--pink);
      border-color: rgba(255, 111, 160, .35);
      background: rgba(255, 111, 160, .08);
    }

    .hero {
      position: relative;
      height: 232px;
      margin: 0 0 14px;
      overflow: hidden;
      border: 1px solid rgba(31, 200, 198, .26);
      border-radius: 18px;
      background: #dff9ff;
      box-shadow: var(--shadow);
    }

    .hero img {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
      object-position: center;
    }

    .hero-title {
      position: absolute;
      left: 54px;
      top: 36px;
      max-width: 480px;
      color: #17315a;
      text-shadow:
        0 4px 0 rgba(255, 255, 255, .94),
        0 0 1px rgba(255, 255, 255, .96);
    }

    .hero-title strong {
      display: block;
      color: #ff7cad;
      font-size: 56px;
      line-height: .95;
      letter-spacing: 0;
      -webkit-text-stroke: 2px #183464;
      filter: drop-shadow(0 6px 0 rgba(255, 255, 255, .90));
    }

    .hero-title span {
      display: block;
      margin-top: 4px;
      font-size: 37px;
      font-weight: 900;
      color: #4150c9;
      -webkit-text-stroke: 1px rgba(255, 255, 255, .90);
    }

    .hero-note {
      position: absolute;
      left: 56px;
      bottom: 25px;
      display: inline-flex;
      gap: 9px;
      align-items: center;
      min-height: 32px;
      padding: 0 24px;
      border-radius: 999px;
      color: #264875;
      font-size: 14px;
      background: rgba(255, 255, 255, .70);
      box-shadow: 0 8px 20px rgba(58, 76, 112, .08);
      backdrop-filter: blur(6px);
    }

    .main-grid {
      display: grid;
      grid-template-columns: minmax(0, 1fr) 336px;
      gap: 16px;
      padding: 0 10px 16px;
    }

    .metrics {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 14px;
      margin-bottom: 12px;
    }

    .metric {
      position: relative;
      display: grid;
      grid-template-columns: 62px 1fr 84px;
      align-items: center;
      min-height: 98px;
      overflow: hidden;
      padding: 14px;
      border: 1px solid var(--line);
      border-radius: 15px;
      background: var(--panel);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .metric::after {
      content: "";
      position: absolute;
      right: -2px;
      top: -3px;
      width: 44px;
      height: 16px;
      border-radius: 2px;
      background: linear-gradient(90deg, #3ecddd, #78cbff);
      transform: rotate(36deg);
    }

    .metric:nth-child(2)::after { background: linear-gradient(90deg, #22c5c2, #7de6db); }
    .metric:nth-child(3)::after { background: linear-gradient(90deg, #9b77ee, #d0b7ff); }
    .metric:nth-child(4)::after { background: linear-gradient(90deg, #ff6fa0, #ff9fba); }

    .metric-icon {
      width: 48px;
      height: 48px;
      display: grid;
      place-items: center;
      border-radius: 15px;
      color: #fff;
      font-size: 24px;
      font-weight: 900;
      background: linear-gradient(145deg, var(--sky), var(--blue));
      box-shadow: inset 0 0 0 3px rgba(255, 255, 255, .72);
    }

    .metric:nth-child(2) .metric-icon { background: linear-gradient(145deg, var(--aqua), #3bded4); }
    .metric:nth-child(3) .metric-icon { background: linear-gradient(145deg, var(--violet), #c7b2ff); }
    .metric:nth-child(4) .metric-icon { background: linear-gradient(145deg, var(--pink), #ff8fab); }

    .metric label {
      display: block;
      color: var(--muted);
      font-size: 13px;
      line-height: 1;
    }

    .metric strong {
      display: block;
      margin-top: 8px;
      color: var(--blue);
      font-size: 32px;
      line-height: 1;
    }

    .metric:nth-child(2) strong { color: var(--aqua); }
    .metric:nth-child(3) strong { color: var(--violet); }
    .metric:nth-child(4) strong { color: var(--pink); }

    .bars {
      align-self: end;
      height: 28px;
      background: repeating-linear-gradient(90deg, rgba(44, 153, 238, .32) 0 4px, transparent 4px 9px) bottom / 68px 22px no-repeat;
    }

    .metric:nth-child(2) .bars { background-image: repeating-linear-gradient(90deg, rgba(31, 200, 198, .34) 0 4px, transparent 4px 9px); }
    .metric:nth-child(3) .bars { background-image: repeating-linear-gradient(90deg, rgba(155, 119, 238, .32) 0 4px, transparent 4px 9px); }
    .metric:nth-child(4) .bars { background-image: repeating-linear-gradient(90deg, rgba(255, 111, 160, .34) 0 4px, transparent 4px 9px); }

    .chain {
      position: relative;
      padding: 14px 16px 20px;
      border: 1px solid var(--line);
      border-radius: 15px;
      background: var(--panel);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .chain-title {
      display: flex;
      align-items: center;
      gap: 9px;
      margin-bottom: 14px;
      color: #405270;
      font-size: 13px;
      font-weight: 820;
      letter-spacing: .06em;
      text-transform: uppercase;
    }

    .chain-title::before,
    .panel-title::before {
      content: "";
      width: 15px;
      height: 15px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: var(--pink);
    }

    .steps {
      display: grid;
      grid-template-columns: repeat(5, minmax(0, 1fr));
      gap: 48px;
    }

    .step {
      position: relative;
      min-height: 108px;
      padding: 18px 14px 12px 76px;
      border: 1px solid rgba(60, 93, 152, .28);
      border-radius: 13px;
      background: rgba(255, 255, 255, .82);
    }

    .step:not(:last-child)::after {
      content: "";
      position: absolute;
      top: 53px;
      right: -50px;
      width: 50px;
      border-top: 2px dashed rgba(68, 148, 239, .55);
    }

    .step-icon {
      position: absolute;
      left: 16px;
      top: 28px;
      display: grid;
      place-items: center;
      width: 48px;
      height: 48px;
      border-radius: 18px;
      color: #fff;
      font-size: 24px;
      font-weight: 900;
      background: linear-gradient(145deg, var(--aqua), #71e5d8);
      box-shadow: inset 0 0 0 4px rgba(255, 255, 255, .60);
    }

    .step:nth-child(2) .step-icon { background: linear-gradient(145deg, var(--lemon), var(--orange)); }
    .step:nth-child(3) .step-icon { background: linear-gradient(145deg, var(--violet), #c8b9ff); }
    .step:nth-child(4) .step-icon { background: linear-gradient(145deg, var(--sky), var(--blue)); }
    .step:nth-child(5) .step-icon { background: linear-gradient(145deg, var(--pink), #ff9fba); }

    .step b {
      display: block;
      margin-bottom: 16px;
      font-size: 14px;
    }

    .chip, .tag, .status-pill {
      display: inline-flex;
      align-items: center;
      min-height: 24px;
      max-width: 100%;
      padding: 0 12px;
      border-radius: 999px;
      font-size: 12px;
      font-weight: 720;
      color: #177d7b;
      background: rgba(31, 200, 198, .16);
      white-space: nowrap;
    }

    .chip.warn { color: #a46712; background: rgba(255, 200, 90, .20); }
    .chip.purple { color: #704fc0; background: rgba(155, 119, 238, .16); }
    .chip.blue { color: #1d6db8; background: rgba(44, 153, 238, .14); }
    .chip.pink { color: #d84777; background: rgba(255, 111, 160, .16); }

    .log-table {
      margin-top: 10px;
      overflow: hidden;
      border: 1px solid var(--line);
      border-radius: 15px;
      background: var(--panel);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .row {
      display: grid;
      grid-template-columns: 22px 128px 190px 160px minmax(0, 1fr) 145px;
      gap: 12px;
      align-items: center;
      min-height: 51px;
      padding: 0 18px;
      border-top: 1px solid rgba(64, 86, 126, .11);
      color: #2d3d58;
      font-size: 13px;
    }

    .row.header {
      min-height: 48px;
      border-top: 0;
      color: #60708d;
      font-size: 12px;
      font-weight: 760;
      background: rgba(250, 253, 255, .82);
    }

    .row.button-row {
      width: 100%;
      border-left: 0;
      border-right: 0;
      border-bottom: 0;
      text-align: left;
      cursor: pointer;
      background: transparent;
    }

    .row.button-row:hover {
      background: linear-gradient(90deg, rgba(31, 200, 198, .07), rgba(255, 111, 160, .04));
    }

    .log-dot {
      width: 9px;
      height: 9px;
      border-radius: 999px;
      background: var(--aqua);
    }

    .row:nth-child(3n) .log-dot { background: var(--pink); }
    .row:nth-child(4n) .log-dot { background: var(--sky); }
    .row:nth-child(5n) .log-dot { background: #40cbb6; }

    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }

    .right-col {
      display: grid;
      gap: 12px;
      align-content: start;
    }

    .panel {
      position: relative;
      padding: 18px 16px;
      border: 1px solid rgba(31, 200, 198, .22);
      border-radius: 15px;
      background: rgba(255, 255, 255, .90);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .panel::after {
      content: "";
      position: absolute;
      right: 18px;
      top: -7px;
      width: 44px;
      height: 15px;
      border-radius: 3px;
      background: rgba(255, 200, 90, .65);
      transform: rotate(13deg);
    }

    .panel-title {
      display: flex;
      gap: 9px;
      align-items: center;
      margin: 0 0 14px;
      color: #2a3956;
      font-size: 17px;
      font-weight: 840;
    }

    .kv {
      display: grid;
      grid-template-columns: 88px minmax(0, 1fr);
      gap: 8px;
      padding: 8px 0;
      border-top: 1px solid rgba(64, 86, 126, .10);
      font-size: 13px;
    }

    .kv:first-of-type { border-top: 0; }
    .kv span:first-child { color: var(--muted); font-weight: 680; }
    .kv span:last-child { min-width: 0; overflow-wrap: anywhere; }

    pre {
      margin: 0;
      max-height: 168px;
      overflow: auto;
      padding: 16px;
      border-radius: 9px;
      color: #9ef2de;
      font-size: 12px;
      line-height: 1.55;
      background: linear-gradient(145deg, #18334f, #101f34);
      box-shadow: inset 0 0 0 1px rgba(255, 255, 255, .06);
    }

    .bottom-stickers {
      display: flex;
      align-items: center;
      justify-content: space-between;
      min-height: 50px;
      padding: 0 12px 0;
      color: #7d8dab;
      font-size: 15px;
      font-weight: 840;
    }

    .sticker-word {
      display: inline-flex;
      align-items: center;
      min-height: 30px;
      padding: 0 12px;
      border-radius: 999px;
      color: #ff6fa0;
      background: rgba(255, 255, 255, .88);
      border: 2px solid rgba(255, 111, 160, .24);
      transform: rotate(-10deg);
      text-shadow: 0 1px 0 #fff;
    }

    .empty {
      min-height: 160px;
      display: grid;
      place-items: center;
      color: var(--muted);
    }

    @media (prefers-reduced-motion: no-preference) {
      .hero img { animation: hero-drift 8s ease-in-out infinite; }
      .brand-mark::before { animation: mascot-bob 3.4s ease-in-out infinite; }
      .sticker-word { animation: sticker-pop 2.8s ease-in-out infinite; }
      @keyframes hero-drift { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.018); } }
      @keyframes mascot-bob { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-4px); } }
      @keyframes sticker-pop { 0%, 100% { transform: rotate(-10deg) scale(1); } 50% { transform: rotate(-7deg) scale(1.04); } }
    }

    @media (max-width: 1500px) {
      .workspace { padding-top: 14px; }
      .topbar, .main-grid { grid-template-columns: 1fr; }
      .top-pills { justify-content: flex-start; flex-wrap: wrap; }
      .hero-title strong { font-size: 44px; }
      .hero-title span { font-size: 30px; }
      .steps { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
      .step::after { display: none; }
      .row { grid-template-columns: 22px 90px 160px 120px minmax(0, 1fr); }
      .row div:nth-child(6) { display: none; }
    }

    @media (max-width: 980px) {
      .cockpit { grid-template-columns: 1fr; border-radius: 0; }
      .sidebar { display: none; }
    }

    @media (max-width: 760px) {
      .workspace { padding: 10px; }
      .topbar { padding: 0 0 10px; }
      .live-strip { overflow-x: auto; }
      .hero { height: 190px; }
      .hero-title { left: 24px; top: 26px; }
      .hero-title strong { font-size: 34px; -webkit-text-stroke-width: 1px; }
      .hero-title span { font-size: 24px; }
      .hero-note { left: 24px; bottom: 18px; }
      .metrics { grid-template-columns: 1fr; }
      .metric { grid-template-columns: 54px 1fr 80px; }
      .steps { grid-template-columns: 1fr; }
      .row { grid-template-columns: 18px 76px minmax(0, 1fr) 84px; }
      .row div:nth-child(4), .row div:nth-child(5) { display: none; }
    }
  </style>
</head>
<body>
  <div class="cockpit">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark" aria-hidden="true"></div>
        <div>
          <strong>agent-llm-mm</strong>
          <span><b>live</b> cockpit</span>
        </div>
      </div>
      <ul class="views">
        <li class="label">Views</li>
        <li class="view active"><span class="view-icon">H</span>Live Operations</li>
        <li class="view"><span class="view-icon">R</span>Reflections</li>
        <li class="view"><span class="view-icon">D</span>Decisions</li>
        <li class="view"><span class="view-icon">S</span>Snapshots</li>
        <li class="view"><span class="view-icon">+</span>Doctor</li>
      </ul>
      <div class="side-spark" aria-hidden="true"></div>
      <div class="side-cards">
        <div class="side-card"><strong id="side-total">0</strong><span>operations this session</span><div class="mini-chart"></div></div>
        <div class="side-card"><strong id="side-conflict">0</strong><span>handled conflict trigger</span><div class="mini-chart"></div></div>
        <div class="side-card"><strong id="side-failed">0</strong><span>failed durable writes</span><div class="mini-chart"></div></div>
      </div>
      <div class="sidebar-sticker"><img src="assets/memory-chan-sidebar.png" alt="" /></div>
    </aside>

    <section class="workspace">
      <header class="topbar">
        <div class="live-strip">
          <span class="strip-item"><span class="dot live-dot"></span><span class="live-pill">LIVE</span></span>
          <span class="strip-item"><span class="dot"></span><span id="live-operation">ingest_interaction</span></span>
          <span class="strip-item"><span class="dot warn-dot"></span>trigger: conflict</span>
          <span class="strip-item"><span class="dot path-dot"></span>write path: run_reflection</span>
        </div>
        <div class="top-pills">
          <span class="pill primary">stdio MCP</span>
          <span class="pill">local sidecar</span>
          <span class="pill danger">read-only</span>
        </div>
      </header>

      <section class="hero">
        <img src="assets/memory-chan-hero.png" alt="" />
        <div class="hero-title">
          <strong>Memory-chan</strong>
          <span>Live Cockpit</span>
        </div>
        <div class="hero-note">Observing · Reflecting · Evolving</div>
      </section>

      <div class="main-grid">
        <main>
          <section class="metrics">
            <div class="metric"><div class="metric-icon">C</div><div><label>MCP tools</label><strong id="tool-count">0</strong></div><div class="bars"></div></div>
            <div class="metric"><div class="metric-icon">M</div><div><label>events</label><strong id="event-count">0</strong></div><div class="bars"></div></div>
            <div class="metric"><div class="metric-icon">*</div><div><label>reflections</label><strong id="reflection-count">0</strong></div><div class="bars"></div></div>
            <div class="metric"><div class="metric-icon">V</div><div><label>status</label><strong id="status-count">ok</strong></div><div class="bars"></div></div>
          </section>

          <section class="chain">
            <div class="chain-title">Current operation chain</div>
            <div class="steps">
              <div class="step"><span class="step-icon">I</span><b>ingest</b><span class="chip">accepted</span></div>
              <div class="step"><span class="step-icon">T</span><b>trigger</b><span class="chip warn">conflict</span></div>
              <div class="step"><span class="step-icon">P</span><b>proposal</b><span class="chip purple">governed</span></div>
              <div class="step"><span class="step-icon">W</span><b>write path</b><span class="chip blue">run_reflection</span></div>
              <div class="step"><span class="step-icon">S</span><b>snapshot</b><span class="chip pink">updated</span></div>
            </div>
          </section>

          <section class="log-table">
            <div class="row header">
              <div></div><div>time</div><div>operation</div><div>kind</div><div>summary</div><div>status</div>
            </div>
            <div id="logs"><div class="empty">waiting for runtime events</div></div>
          </section>
          <div class="bottom-stickers">
            <span class="sticker-word">GOOD</span>
            <span>*</span>
            <span>heart</span>
            <span class="sticker-word">KEEP GOING!</span>
          </div>
        </main>

        <aside class="right-col">
          <section class="panel">
            <h4 class="panel-title">Selected operation</h4>
            <div id="selected-operation"><div class="kv"><span>status</span><span>waiting</span></div></div>
          </section>
          <section class="panel">
            <h4 class="panel-title">Payload inspector</h4>
            <pre id="payload">{}</pre>
          </section>
          <section class="panel">
            <h4 class="panel-title">Boundaries</h4>
            <div class="kv"><span>mode</span><span><span class="tag">local demo</span></span></div>
            <div class="kv"><span>actions</span><span><span class="tag">read only</span></span></div>
            <div class="kv"><span>stdout</span><span><span class="tag">MCP only</span></span></div>
          </section>
        </aside>
      </div>
    </section>
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

    function chipClass(kind) {
      const value = String(kind || "").toLowerCase();
      if (value.includes("decision")) return "tag chip warn";
      if (value.includes("snapshot")) return "tag chip blue";
      if (value.includes("reflection")) return "tag chip pink";
      return "tag";
    }

    async function refresh() {
      const [summary, events] = await Promise.all([
        fetch(api("api/summary")).then((response) => response.json()),
        fetch(api("api/events?limit=25")).then((response) => response.json())
      ]);

      document.getElementById("side-total").textContent = summary.total_events;
      document.getElementById("side-conflict").textContent = summary.reflection_events;
      document.getElementById("side-failed").textContent = summary.failed_events;
      document.getElementById("tool-count").textContent = summary.tool_events;
      document.getElementById("event-count").textContent = summary.total_events;
      document.getElementById("reflection-count").textContent = summary.reflection_events;
      document.getElementById("status-count").textContent = summary.failed_events > 0 ? "check" : "ok";

      const logs = document.getElementById("logs");
      if (!events.length) {
        logs.innerHTML = '<div class="empty">waiting for runtime events</div>';
        return;
      }

      logs.innerHTML = events.map((event, index) => `
        <button class="row button-row" data-index="${index}">
          <div><span class="log-dot"></span></div>
          <div class="mono">${escapeHtml(new Date(event.timestamp).toLocaleTimeString())}</div>
          <div>${escapeHtml(event.operation)}</div>
          <div><span class="${chipClass(event.kind)}">${escapeHtml(event.kind)}</span></div>
          <div>${escapeHtml(event.summary)}</div>
          <div><span class="status-pill">${escapeHtml(event.status)}</span></div>
        </button>
      `).join("");

      logs.querySelectorAll(".button-row").forEach((node) => {
        node.addEventListener("click", () => selectEvent(events[Number(node.dataset.index)]));
      });
      selectEvent(events[events.length - 1]);
    }

    function selectEvent(event) {
      document.getElementById("live-operation").textContent = event.operation;
      document.getElementById("selected-operation").innerHTML = `
        <div class="kv"><span>id</span><span><span class="tag mono">${escapeHtml(event.id)}</span></span></div>
        <div class="kv"><span>namespace</span><span><span class="tag">${escapeHtml(event.namespace || "-")}</span></span></div>
        <div class="kv"><span>trigger</span><span><span class="tag chip warn">${escapeHtml(event.kind)}</span></span></div>
        <div class="kv"><span>write path</span><span><span class="tag chip blue">${escapeHtml(event.operation)}</span></span></div>
      `;
      document.getElementById("payload").textContent = JSON.stringify(event.payload || {}, null, 2);
    }

    refresh().catch(() => {});
    setInterval(() => refresh().catch(() => {}), 2500);
  </script>
</body>
</html>"#;
