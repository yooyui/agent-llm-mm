pub const MEMORY_CHAN_HERO_PNG: &[u8] = include_bytes!("static/memory_chan_hero.png");
pub const MEMORY_CHAN_SIDEBAR_PNG: &[u8] = include_bytes!("static/memory_chan_sidebar.png");

pub const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Memory-chan Live Desk</title>
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

    .star-shape {
      display: inline-block;
      width: 16px;
      height: 16px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: var(--lemon);
      filter: drop-shadow(0 2px 0 rgba(255, 255, 255, .86));
    }

    .heart-shape {
      display: inline-block;
      width: 17px;
      height: 17px;
      border-radius: 12px 12px 4px 12px;
      background: var(--pink);
      transform: rotate(45deg);
      box-shadow: inset 0 0 0 3px rgba(255, 255, 255, .55), 0 8px 14px rgba(255, 111, 160, .16);
    }

    .x-spark {
      position: relative;
      width: 18px;
      height: 18px;
    }

    .x-spark::before,
    .x-spark::after {
      content: "";
      position: absolute;
      left: 8px;
      top: 1px;
      width: 2px;
      height: 16px;
      border-radius: 999px;
      background: rgba(44, 153, 238, .55);
    }

    .x-spark::after { transform: rotate(90deg); }

    .float-decor {
      position: absolute;
      z-index: 3;
      pointer-events: none;
    }

    .float-decor.star-a,
    .float-decor.star-b {
      width: 22px;
      height: 22px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      filter: drop-shadow(0 4px 0 rgba(255, 255, 255, .82));
    }

    .float-decor.star-a {
      left: 248px;
      top: 118px;
      background: var(--lemon);
    }

    .float-decor.star-b {
      right: 18px;
      bottom: 20px;
      width: 18px;
      height: 18px;
      background: #9aa7ff;
    }

    .float-decor.heart-a {
      right: 76px;
      bottom: 32px;
      width: 18px;
      height: 18px;
      border-radius: 12px 12px 4px 12px;
      background: var(--pink);
      transform: rotate(45deg);
      box-shadow: inset 0 0 0 3px rgba(255, 255, 255, .68);
    }

    .float-decor.zap-a {
      right: 48px;
      bottom: 70px;
      width: 30px;
      height: 42px;
      clip-path: polygon(48% 0, 92% 0, 63% 36%, 100% 36%, 36% 100%, 48% 56%, 8% 56%);
      background: linear-gradient(180deg, #ffe27d, #ff9e47);
      filter: drop-shadow(0 0 0 rgba(255, 255, 255, .90)) drop-shadow(0 8px 13px rgba(255, 158, 71, .20));
      transform: rotate(13deg);
    }

    .cockpit {
      position: relative;
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

    .cockpit::before,
    .cockpit::after {
      content: "";
      position: absolute;
      z-index: 0;
      width: 22px;
      height: 22px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      pointer-events: none;
    }

    .cockpit::before {
      right: 24px;
      bottom: 22px;
      background: #8c9cff;
      opacity: .58;
    }

    .cockpit::after {
      left: 250px;
      top: 124px;
      background: var(--lemon);
      opacity: .75;
    }

    .sidebar {
      position: relative;
      z-index: 1;
      display: flex;
      flex-direction: column;
      min-width: 0;
      border-right: 1px solid var(--line);
      background: rgba(255, 255, 255, .72);
      backdrop-filter: blur(18px);
    }

    .brand {
      position: relative;
      display: flex;
      align-items: center;
      gap: 12px;
      min-height: 98px;
      padding: 18px 22px;
      border-bottom: 1px solid var(--line);
    }

    .brand::after {
      content: "";
      position: absolute;
      right: 24px;
      bottom: 18px;
      width: 11px;
      height: 11px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: var(--pink);
      opacity: .75;
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

    .view.active::after {
      content: "";
      position: absolute;
      right: 22px;
      width: 12px;
      height: 12px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(255, 255, 255, .90);
      filter: drop-shadow(0 0 10px rgba(31, 200, 198, .55));
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

    .side-confetti {
      position: absolute;
      top: 122px;
      right: 22px;
      width: 58px;
      height: 268px;
      pointer-events: none;
    }

    .side-confetti > * {
      position: absolute;
    }

    .side-confetti .star-shape:nth-child(1) { right: 28px; top: 8px; }
    .side-confetti .heart-shape { right: 0; top: 24px; width: 12px; height: 12px; }
    .side-confetti .x-spark { right: 10px; bottom: 46px; transform: rotate(45deg); }
    .side-confetti .star-shape:nth-child(4) { right: 6px; bottom: 0; width: 28px; height: 28px; background: #ffda80; }

    .side-cards {
      display: grid;
      gap: 14px;
      padding: 0 12px 12px;
    }

    .side-card {
      position: relative;
      min-height: 126px;
      padding: 18px 20px;
      border: 1px solid var(--line);
      border-radius: 14px;
      background: rgba(255, 255, 255, .86);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .side-card::after {
      content: "";
      position: absolute;
      right: 18px;
      top: 18px;
      width: 70px;
      height: 34px;
      border-radius: 999px;
      background:
        linear-gradient(135deg, transparent 0 45%, rgba(255, 111, 160, .55) 46% 50%, transparent 51% 100%),
        linear-gradient(160deg, transparent 0 45%, rgba(255, 111, 160, .35) 46% 50%, transparent 51% 100%);
      opacity: .72;
    }

    .side-card:nth-child(2)::after {
      background:
        linear-gradient(135deg, transparent 0 45%, rgba(255, 158, 71, .58) 46% 50%, transparent 51% 100%),
        linear-gradient(160deg, transparent 0 45%, rgba(255, 158, 71, .35) 46% 50%, transparent 51% 100%);
    }

    .side-card:nth-child(3)::after {
      background:
        linear-gradient(135deg, transparent 0 45%, rgba(92, 207, 109, .58) 46% 50%, transparent 51% 100%),
        linear-gradient(160deg, transparent 0 45%, rgba(92, 207, 109, .35) 46% 50%, transparent 51% 100%);
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
      position: relative;
      margin-top: auto;
      padding: 0 18px 14px;
    }

    .sidebar-sticker::before,
    .sidebar-sticker::after {
      content: "";
      position: absolute;
      pointer-events: none;
      z-index: 2;
    }

    .sidebar-sticker::before {
      right: 18px;
      top: -6px;
      width: 18px;
      height: 18px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: #ffd67a;
      filter: drop-shadow(0 3px 0 rgba(255, 255, 255, .86));
    }

    .sidebar-sticker::after {
      right: 8px;
      bottom: 34px;
      width: 16px;
      height: 16px;
      border: 3px solid rgba(255, 111, 160, .42);
      border-radius: 7px;
      transform: rotate(24deg);
    }

    .sidebar-sticker img {
      display: block;
      width: 100%;
      height: 186px;
      object-fit: contain;
      object-position: 50% 100%;
      border-radius: 16px;
      mix-blend-mode: multiply;
    }

    .workspace {
      position: relative;
      z-index: 1;
      min-width: 0;
      padding: 22px 10px 0;
    }

    .workspace::before,
    .workspace::after {
      content: "";
      position: absolute;
      z-index: -1;
      pointer-events: none;
    }

    .workspace::before {
      right: 18px;
      top: 86px;
      width: 70px;
      height: 70px;
      border: 12px solid rgba(255, 111, 160, .12);
      border-radius: 28px;
      transform: rotate(18deg);
    }

    .workspace::after {
      left: 34%;
      bottom: 22px;
      width: 110px;
      height: 28px;
      border-radius: 999px;
      background: linear-gradient(90deg, rgba(31, 200, 198, .14), rgba(255, 111, 160, .13));
      transform: rotate(-16deg);
    }

    .topbar {
      position: relative;
      display: grid;
      grid-template-columns: minmax(0, 1fr);
      gap: 12px;
      align-items: center;
      padding: 0 24px 18px 32px;
    }

    .topbar::after {
      content: "";
      position: absolute;
      right: 34px;
      bottom: 2px;
      width: 34px;
      height: 34px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(255, 200, 90, .70);
      filter: drop-shadow(0 4px 0 rgba(255, 255, 255, .80));
    }

    .live-strip {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      min-height: 55px;
      gap: 0;
      overflow: visible;
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
      flex-wrap: wrap;
      justify-content: flex-start;
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

    .hero::before,
    .hero::after {
      content: "";
      position: absolute;
      z-index: 2;
      pointer-events: none;
    }

    .hero::before {
      right: 35px;
      top: 30px;
      width: 46px;
      height: 46px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(255, 218, 109, .92);
      filter: drop-shadow(0 4px 0 rgba(255, 255, 255, .84));
    }

    .hero::after {
      right: 72px;
      bottom: 33px;
      width: 88px;
      height: 88px;
      border: 6px solid rgba(255, 111, 160, .66);
      border-radius: 28px;
      transform: rotate(18deg);
      box-shadow: inset 0 0 0 4px rgba(255, 255, 255, .50);
    }

    .hero img {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
      object-position: 50% 28%;
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

    .hero-deco {
      position: absolute;
      z-index: 3;
      pointer-events: none;
    }

    .hero-deco.one {
      left: 340px;
      top: 28px;
      width: 22px;
      height: 22px;
      background: var(--lemon);
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
    }

    .hero-deco.two {
      left: 318px;
      bottom: 34px;
      width: 18px;
      height: 18px;
      border: 3px solid var(--pink);
      border-radius: 7px;
      transform: rotate(26deg);
    }

    .hero-deco.three {
      right: 226px;
      top: 46px;
      width: 18px;
      height: 18px;
      border-radius: 9px 9px 3px 9px;
      background: var(--pink);
      transform: rotate(45deg);
      box-shadow: inset 0 0 0 3px rgba(255, 255, 255, .68);
    }

    .hero-deco.four {
      right: 172px;
      bottom: 38px;
      width: 28px;
      height: 28px;
      border: 3px solid rgba(44, 153, 238, .68);
      border-radius: 10px;
      transform: rotate(18deg);
      background: rgba(255, 255, 255, .28);
    }

    .hero-deco.five {
      left: 456px;
      bottom: 64px;
      width: 62px;
      height: 18px;
      border-radius: 999px;
      background: linear-gradient(90deg, rgba(255, 111, 160, .75), rgba(255, 200, 90, .78));
      transform: rotate(-14deg);
      box-shadow: 0 0 0 5px rgba(255, 255, 255, .52);
    }

    .main-grid {
      display: grid;
      grid-template-columns: minmax(0, 1fr) 336px;
      gap: 16px;
      padding: 0 10px 16px;
    }

    .metrics {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
      gap: 14px;
      margin-bottom: 12px;
    }

    .metric {
      position: relative;
      display: grid;
      grid-template-columns: 54px minmax(0, 1fr) 76px;
      align-items: center;
      min-height: 98px;
      overflow: hidden;
      padding: 14px;
      border: 1px solid var(--line);
      border-radius: 15px;
      background: var(--panel);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .metric .mini-spark {
      position: absolute;
      left: 70px;
      bottom: 14px;
      width: 15px;
      height: 15px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(44, 153, 238, .24);
    }

    .metric:nth-child(3) .mini-spark { background: rgba(155, 119, 238, .26); }
    .metric:nth-child(4) .mini-spark { background: rgba(255, 111, 160, .28); }

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
      line-height: 1.15;
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

    .chain::before,
    .chain::after {
      content: "";
      position: absolute;
      pointer-events: none;
    }

    .chain::before {
      right: 22px;
      top: 22px;
      width: 18px;
      height: 18px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(44, 153, 238, .38);
    }

    .chain::after {
      right: 34px;
      top: 34px;
      width: 15px;
      height: 15px;
      border: 2px solid rgba(155, 119, 238, .42);
      border-radius: 6px;
      transform: rotate(20deg);
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
      grid-template-columns: repeat(auto-fit, minmax(156px, 1fr));
      gap: 12px;
    }

    .step {
      position: relative;
      min-height: 108px;
      padding: 18px 14px 12px 76px;
      border: 1px solid rgba(60, 93, 152, .28);
      border-radius: 13px;
      background: rgba(255, 255, 255, .82);
    }

    .step::before {
      content: "";
      position: absolute;
      right: 14px;
      top: 13px;
      width: 12px;
      height: 12px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(255, 111, 160, .36);
    }

    .step-glint {
      display: none;
      position: absolute;
      right: -31px;
      top: 47px;
      width: 13px;
      height: 13px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(44, 153, 238, .62);
      filter: drop-shadow(0 2px 0 rgba(255, 255, 255, .82));
    }

    .step:last-child .step-glint { display: none; }
    .step:nth-child(2) .step-glint { background: rgba(255, 200, 90, .78); }
    .step:nth-child(3) .step-glint { background: rgba(155, 119, 238, .62); }
    .step:nth-child(4) .step-glint { background: rgba(255, 111, 160, .60); }

    .step:not(:last-child)::after {
      content: "";
      display: none;
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

    .event-id {
      display: block;
      width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .chip.warn { color: #a46712; background: rgba(255, 200, 90, .20); }
    .chip.purple { color: #704fc0; background: rgba(155, 119, 238, .16); }
    .chip.blue { color: #1d6db8; background: rgba(44, 153, 238, .14); }
    .chip.pink { color: #d84777; background: rgba(255, 111, 160, .16); }

    .log-table {
      position: relative;
      margin-top: 10px;
      overflow: hidden;
      border: 1px solid var(--line);
      border-radius: 15px;
      background: var(--panel);
      box-shadow: 0 10px 26px rgba(58, 76, 112, .06);
    }

    .log-table::after {
      content: "";
      position: absolute;
      right: 18px;
      top: -8px;
      width: 50px;
      height: 16px;
      border-radius: 3px;
      background: rgba(255, 200, 90, .60);
      transform: rotate(12deg);
    }

    .row {
      display: grid;
      grid-template-columns: 22px minmax(86px, .7fr) minmax(130px, 1fr) minmax(96px, .75fr) minmax(180px, 1.4fr) minmax(84px, .7fr);
      gap: 12px;
      align-items: center;
      min-height: 51px;
      padding: 0 18px;
      border-top: 1px solid rgba(64, 86, 126, .11);
      color: #2d3d58;
      font-size: 13px;
    }

    .row > div {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .row > div:nth-child(5) {
      white-space: normal;
      overflow-wrap: anywhere;
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

    .panel-deco {
      position: absolute;
      pointer-events: none;
      z-index: 2;
    }

    .panel-deco.star {
      right: 16px;
      top: 12px;
      width: 36px;
      height: 36px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: #ff9fd0;
      filter: drop-shadow(0 0 0 rgba(255, 255, 255, .86)) drop-shadow(0 7px 13px rgba(255, 111, 160, .20));
    }

    .panel-deco.tape {
      right: 16px;
      top: 10px;
      width: 48px;
      height: 28px;
      border-radius: 5px;
      background:
        linear-gradient(45deg, rgba(255, 255, 255, .40) 0 22%, transparent 22% 43%, rgba(255, 255, 255, .35) 43% 64%, transparent 64% 100%),
        linear-gradient(135deg, #ffd98a, #ffb7cf);
      transform: rotate(19deg);
      box-shadow: 0 7px 13px rgba(255, 158, 71, .15);
    }

    .panel-deco.zap {
      right: 18px;
      top: 8px;
      width: 26px;
      height: 40px;
      clip-path: polygon(48% 0, 90% 0, 62% 37%, 100% 37%, 34% 100%, 47% 58%, 7% 58%);
      background: linear-gradient(180deg, #ffe783, #ff9e47);
      transform: rotate(13deg);
      filter: drop-shadow(0 0 0 rgba(255, 255, 255, .90)) drop-shadow(0 7px 12px rgba(255, 158, 71, .18));
    }

    .panel-deco.star::after,
    .panel-deco.zap::after {
      content: "";
      position: absolute;
      inset: 5px;
      background: rgba(255, 255, 255, .34);
      clip-path: inherit;
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
      position: relative;
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: flex-start;
      gap: 22px;
      min-height: 50px;
      padding: 0 12px 0;
      color: #7d8dab;
      font-size: 15px;
      font-weight: 840;
    }

    .bottom-stickers::before,
    .bottom-stickers::after {
      content: "";
      position: absolute;
      pointer-events: none;
    }

    .bottom-stickers::before {
      left: 104px;
      bottom: 8px;
      width: 26px;
      height: 26px;
      clip-path: polygon(50% 0, 61% 35%, 98% 50%, 61% 65%, 50% 100%, 39% 65%, 2% 50%, 39% 35%);
      background: rgba(255, 200, 90, .74);
      filter: drop-shadow(0 3px 0 rgba(255, 255, 255, .80));
    }

    .bottom-stickers::after {
      right: 20px;
      bottom: 9px;
      width: 44px;
      height: 16px;
      border-radius: 999px;
      background: linear-gradient(90deg, var(--lemon), var(--orange));
      transform: rotate(42deg);
      box-shadow: 0 0 0 4px rgba(255, 255, 255, .72);
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

    .sticker-word.keep {
      margin-left: auto;
      color: #a77cff;
      border-color: rgba(155, 119, 238, .26);
      transform: rotate(-7deg);
    }

    .bottom-stickers .star-shape {
      width: 26px;
      height: 26px;
      background: #ffd67a;
    }

    .bottom-stickers .heart-shape {
      width: 20px;
      height: 20px;
    }

    .bubble-sticker {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 34px;
      height: 28px;
      border: 2px solid rgba(155, 119, 238, .22);
      border-radius: 16px 16px 16px 5px;
      background: rgba(255, 255, 255, .88);
      transform: rotate(8deg);
      box-shadow: 0 8px 16px rgba(58, 76, 112, .08);
    }

    .arrow-sticker {
      width: 54px;
      height: 34px;
      position: relative;
      transform: rotate(-20deg);
    }

    .arrow-sticker::before,
    .arrow-sticker::after {
      content: "";
      position: absolute;
      background: var(--pink);
    }

    .arrow-sticker::before {
      left: 2px;
      top: 14px;
      width: 40px;
      height: 9px;
      border-radius: 999px;
    }

    .arrow-sticker::after {
      right: 5px;
      top: 5px;
      width: 24px;
      height: 24px;
      clip-path: polygon(0 0, 100% 50%, 0 100%, 28% 50%);
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
      .hero-deco, .float-decor, .panel-deco { animation: decor-float 4.6s ease-in-out infinite; }
      .hero-deco.two, .panel-deco.tape { animation-delay: -1.6s; }
      .hero-deco.four, .float-decor.zap-a { animation-delay: -2.4s; }
      @keyframes hero-drift { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.018); } }
      @keyframes mascot-bob { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-4px); } }
      @keyframes sticker-pop { 0%, 100% { transform: rotate(-10deg) scale(1); } 50% { transform: rotate(-7deg) scale(1.04); } }
      @keyframes decor-float { 0%, 100% { translate: 0 0; } 50% { translate: 0 -5px; } }
    }

    @media (max-width: 1500px) {
      .workspace { padding-top: 14px; }
      .topbar, .main-grid { grid-template-columns: minmax(0, 1fr); }
      .top-pills { justify-content: flex-start; flex-wrap: wrap; }
      .hero-title strong { font-size: 44px; }
      .hero-title span { font-size: 30px; }
      .steps { grid-template-columns: repeat(auto-fit, minmax(156px, 1fr)); gap: 12px; }
      .step::after { display: none; }
      .row { grid-template-columns: 22px 90px 160px 120px minmax(0, 1fr); }
      .row div:nth-child(6) { display: none; }
    }

    @media (max-width: 980px) {
      .cockpit { grid-template-columns: 1fr; border-radius: 0; }
      .sidebar { display: none; }
    }

    @media (max-width: 760px) {
      .float-decor { display: none; }
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
      .steps { grid-template-columns: minmax(0, 1fr); }
      .step-glint { display: none; }
      .row { grid-template-columns: 18px 76px minmax(0, 1fr) 84px; }
      .row div:nth-child(4), .row div:nth-child(5) { display: none; }
    }
  </style>
</head>
<body>
  <div class="cockpit">
    <span class="float-decor star-a" aria-hidden="true"></span>
    <span class="float-decor heart-a" aria-hidden="true"></span>
    <span class="float-decor zap-a" aria-hidden="true"></span>
    <span class="float-decor star-b" aria-hidden="true"></span>
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark" aria-hidden="true"></div>
        <div>
          <strong>agent-llm-mm</strong>
          <span><b>live</b> desk</span>
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
      <div class="side-confetti" aria-hidden="true">
        <span class="star-shape"></span>
        <span class="heart-shape"></span>
        <span class="x-spark"></span>
        <span class="star-shape"></span>
      </div>
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
          <span>Live Desk</span>
        </div>
        <div class="hero-note">Observing · Reflecting · Evolving</div>
        <span class="hero-deco one" aria-hidden="true"></span>
        <span class="hero-deco two" aria-hidden="true"></span>
        <span class="hero-deco three" aria-hidden="true"></span>
        <span class="hero-deco four" aria-hidden="true"></span>
        <span class="hero-deco five" aria-hidden="true"></span>
      </section>

      <div class="main-grid">
        <main>
          <section class="metrics">
            <div class="metric"><span class="mini-spark" aria-hidden="true"></span><div class="metric-icon">C</div><div><label>MCP tools</label><strong id="tool-count">0</strong></div><div class="bars"></div></div>
            <div class="metric"><span class="mini-spark" aria-hidden="true"></span><div class="metric-icon">M</div><div><label>events</label><strong id="event-count">0</strong></div><div class="bars"></div></div>
            <div class="metric"><span class="mini-spark" aria-hidden="true"></span><div class="metric-icon">*</div><div><label>reflections</label><strong id="reflection-count">0</strong></div><div class="bars"></div></div>
            <div class="metric"><span class="mini-spark" aria-hidden="true"></span><div class="metric-icon">V</div><div><label>status</label><strong id="status-count">ok</strong></div><div class="bars"></div></div>
          </section>

          <section class="chain">
            <div class="chain-title">Current operation chain</div>
            <div class="steps">
              <div class="step"><span class="step-glint" aria-hidden="true"></span><span class="step-icon">I</span><b>ingest</b><span class="chip">accepted</span></div>
              <div class="step"><span class="step-glint" aria-hidden="true"></span><span class="step-icon">T</span><b>trigger</b><span class="chip warn">conflict</span></div>
              <div class="step"><span class="step-glint" aria-hidden="true"></span><span class="step-icon">P</span><b>proposal</b><span class="chip purple">governed</span></div>
              <div class="step"><span class="step-glint" aria-hidden="true"></span><span class="step-icon">W</span><b>write path</b><span class="chip blue">run_reflection</span></div>
              <div class="step"><span class="step-glint" aria-hidden="true"></span><span class="step-icon">S</span><b>snapshot</b><span class="chip pink">updated</span></div>
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
            <span class="star-shape"></span>
            <span class="heart-shape"></span>
            <span class="bubble-sticker">!</span>
            <span class="sticker-word keep">KEEP GOING!</span>
            <span class="arrow-sticker"></span>
          </div>
        </main>

        <aside class="right-col">
          <section class="panel">
            <h4 class="panel-title">Selected operation</h4>
            <span class="panel-deco star" aria-hidden="true"></span>
            <div id="selected-operation"><div class="kv"><span>status</span><span>waiting</span></div></div>
          </section>
          <section class="panel">
            <h4 class="panel-title">Payload inspector</h4>
            <span class="panel-deco tape" aria-hidden="true"></span>
            <pre id="payload">{}</pre>
          </section>
          <section class="panel">
            <h4 class="panel-title">Boundaries</h4>
            <span class="panel-deco zap" aria-hidden="true"></span>
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
        <div class="kv"><span>id</span><span><span class="tag mono event-id" title="${escapeHtml(event.id)}">${escapeHtml(event.id)}</span></span></div>
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
