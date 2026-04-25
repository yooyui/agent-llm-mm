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
      --muted: #66738d;
      --line: rgba(45, 58, 92, .13);
      --glass: rgba(255, 255, 255, .74);
      --glass-strong: rgba(255, 255, 255, .90);
      --pink: #ff79ad;
      --rose: #ff5f93;
      --peach: #ffb17e;
      --lemon: #ffe37b;
      --mint: #62e4cb;
      --sky: #67c8ff;
      --blue: #6790ff;
      --violet: #b9a4ff;
      --shadow: 0 24px 62px rgba(68, 77, 112, .16);
    }

    * { box-sizing: border-box; }

    html { min-width: 320px; }

    body {
      min-width: 320px;
      margin: 0;
      color: var(--ink);
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background:
        radial-gradient(circle at 12% 14%, rgba(255, 121, 173, .26), transparent 27%),
        radial-gradient(circle at 86% 16%, rgba(98, 228, 203, .27), transparent 26%),
        radial-gradient(circle at 68% 88%, rgba(103, 144, 255, .16), transparent 30%),
        linear-gradient(135deg, #fff0f7 0%, #ecfbff 45%, #fffbe7 100%);
    }

    body::before,
    body::after {
      content: "";
      position: fixed;
      inset: 0;
      pointer-events: none;
    }

    body::before {
      background:
        radial-gradient(circle at 18px 18px, rgba(255, 121, 173, .25) 0 2px, transparent 2.5px),
        radial-gradient(circle at 58px 48px, rgba(103, 200, 255, .24) 0 2px, transparent 2.5px),
        linear-gradient(115deg, transparent 0 47%, rgba(255, 255, 255, .54) 47% 48%, transparent 48% 100%);
      background-size: 76px 76px, 76px 76px, 190px 190px;
      mask-image: linear-gradient(180deg, #000, transparent 78%);
    }

    body::after {
      opacity: .7;
      background:
        conic-gradient(from 100deg at 16% 24%, transparent, rgba(255, 121, 173, .24), transparent 18%),
        conic-gradient(from 230deg at 82% 22%, transparent, rgba(98, 228, 203, .22), transparent 18%),
        conic-gradient(from 20deg at 62% 78%, transparent, rgba(103, 144, 255, .16), transparent 20%);
      filter: blur(22px);
    }

    button, pre { font: inherit; }

    .shell {
      position: relative;
      min-height: 100vh;
      padding: 18px;
    }

    .stage {
      position: relative;
      overflow: hidden;
      min-height: calc(100vh - 36px);
      border: 1px solid rgba(255, 255, 255, .78);
      border-radius: 20px;
      background: linear-gradient(145deg, rgba(255, 255, 255, .73), rgba(255, 255, 255, .50));
      box-shadow: var(--shadow);
      backdrop-filter: blur(24px);
    }

    .stage::before {
      content: "";
      position: absolute;
      inset: 0;
      pointer-events: none;
      background:
        linear-gradient(102deg, transparent 0 8%, rgba(255, 255, 255, .54) 8% 8.8%, transparent 8.8% 100%),
        linear-gradient(108deg, transparent 0 69%, rgba(255, 255, 255, .42) 69% 70%, transparent 70% 100%);
      animation: shimmer 9s ease-in-out infinite;
    }

    .stage::after {
      content: "";
      position: absolute;
      top: 124px;
      left: 27%;
      width: 78%;
      height: 16px;
      border-radius: 999px;
      background: linear-gradient(90deg, rgba(98, 228, 203, .40), rgba(255, 227, 123, .46), rgba(255, 121, 173, .34));
      transform: rotate(-10deg);
      transform-origin: left center;
      filter: blur(.2px);
      pointer-events: none;
    }

    .hero {
      position: relative;
      z-index: 1;
      display: grid;
      grid-template-columns: minmax(330px, 1fr) 380px;
      gap: 22px;
      align-items: stretch;
      padding: 22px;
      border-bottom: 1px solid var(--line);
    }

    .hero-copy {
      min-width: 0;
      padding: 20px;
      border: 1px solid rgba(255, 255, 255, .76);
      border-radius: 18px;
      background: linear-gradient(135deg, rgba(255, 255, 255, .88), rgba(255, 255, 255, .58));
      box-shadow: 0 16px 38px rgba(68, 77, 112, .10);
    }

    .kicker {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
      margin-bottom: 14px;
    }

    .live {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      min-height: 31px;
      border-radius: 999px;
      padding: 0 12px;
      color: #fff;
      font-size: 12px;
      font-weight: 860;
      letter-spacing: .04em;
      background: linear-gradient(135deg, var(--rose), var(--peach));
      box-shadow: 0 12px 22px rgba(255, 121, 173, .28);
    }

    .live::before {
      content: "";
      width: 7px;
      height: 7px;
      border-radius: 999px;
      background: #fff;
      box-shadow: 0 0 0 5px rgba(255, 255, 255, .26);
    }

    .bubble, .pill {
      display: inline-flex;
      align-items: center;
      min-height: 31px;
      border: 1px solid rgba(45, 58, 92, .10);
      border-radius: 999px;
      padding: 0 11px;
      color: #526077;
      font-size: 12px;
      font-weight: 700;
      background: rgba(255, 255, 255, .70);
      box-shadow: 0 7px 18px rgba(68, 77, 112, .07);
      white-space: nowrap;
    }

    h1 {
      margin: 0;
      max-width: 860px;
      font-size: clamp(32px, 4.4vw, 62px);
      line-height: .96;
      letter-spacing: 0;
    }

    .hero-line {
      display: block;
      color: transparent;
      background: linear-gradient(120deg, var(--pink), var(--blue) 52%, #17bea7);
      background-clip: text;
      -webkit-background-clip: text;
    }

    .hero-sub {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      margin: 16px 0 0;
    }

    .hero-facts {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 10px;
      margin-top: 18px;
    }

    .fact {
      position: relative;
      min-height: 78px;
      overflow: hidden;
      padding: 12px;
      border: 1px solid rgba(255, 255, 255, .78);
      border-radius: 16px;
      background: rgba(255, 255, 255, .64);
      box-shadow: 0 12px 24px rgba(68, 77, 112, .08);
    }

    .fact::after {
      content: "";
      position: absolute;
      right: -18px;
      bottom: -24px;
      width: 72px;
      height: 72px;
      border-radius: 28px;
      background: linear-gradient(135deg, rgba(103, 200, 255, .22), rgba(255, 121, 173, .17));
      transform: rotate(18deg);
    }

    .fact label {
      display: block;
      color: var(--muted);
      font-size: 11px;
      font-weight: 780;
      text-transform: uppercase;
    }

    .fact strong {
      display: block;
      margin-top: 8px;
      font-size: 26px;
      line-height: 1;
    }

    .visual {
      position: relative;
      min-height: 282px;
      overflow: hidden;
      border: 1px solid rgba(255, 255, 255, .80);
      border-radius: 20px;
      background:
        radial-gradient(circle at 19% 22%, rgba(255, 255, 255, .96), transparent 18%),
        linear-gradient(145deg, rgba(255, 184, 215, .84), rgba(181, 234, 255, .84) 48%, rgba(255, 241, 155, .82));
      box-shadow: 0 18px 44px rgba(103, 144, 255, .18);
    }

    .visual::before {
      content: "";
      position: absolute;
      left: -38px;
      right: -38px;
      bottom: -66px;
      height: 166px;
      border-radius: 50% 50% 0 0;
      background: rgba(255, 255, 255, .44);
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, .72);
    }

    .sun-ring {
      position: absolute;
      top: 26px;
      right: 28px;
      width: 76px;
      height: 76px;
      border-radius: 999px;
      border: 10px solid rgba(255, 255, 255, .64);
      box-shadow: 0 0 0 18px rgba(255, 227, 123, .23);
    }

    .swoosh {
      position: absolute;
      left: 30px;
      right: 22px;
      height: 12px;
      border-radius: 999px;
      transform: rotate(-17deg);
      background: linear-gradient(90deg, rgba(255, 255, 255, .94), rgba(98, 228, 203, .78), rgba(103, 144, 255, .56));
      box-shadow: 0 12px 20px rgba(103, 144, 255, .18);
    }

    .swoosh.one { top: 68px; }
    .swoosh.two { top: 129px; left: 94px; opacity: .76; }

    .spark {
      position: absolute;
      width: 15px;
      height: 15px;
      clip-path: polygon(50% 0, 62% 36%, 100% 50%, 62% 64%, 50% 100%, 38% 64%, 0 50%, 38% 36%);
      background: #fff;
      box-shadow: 0 0 18px rgba(255, 255, 255, .75);
    }

    .spark.a { left: 34px; top: 34px; }
    .spark.b { right: 46px; bottom: 62px; width: 20px; height: 20px; background: var(--lemon); }
    .spark.c { right: 116px; top: 108px; background: var(--pink); }

    .mascot {
      position: absolute;
      left: 90px;
      bottom: 20px;
      width: 172px;
      height: 214px;
      filter: drop-shadow(0 21px 30px rgba(67, 75, 114, .23));
    }

    .hair {
      position: absolute;
      left: 32px;
      top: 18px;
      width: 108px;
      height: 124px;
      border-radius: 56px 56px 40px 40px;
      background:
        linear-gradient(160deg, rgba(255, 255, 255, .42), transparent 30%),
        linear-gradient(145deg, #59dfd1, #6aa8ff 52%, #b9a4ff);
    }

    .hair::before,
    .hair::after {
      content: "";
      position: absolute;
      top: 35px;
      width: 58px;
      height: 106px;
      border-radius: 48px;
      background: linear-gradient(180deg, #58d9d2, #7ca8ff);
    }

    .hair::before { left: -36px; transform: rotate(18deg); }
    .hair::after { right: -36px; transform: rotate(-18deg); }

    .face {
      position: absolute;
      left: 48px;
      top: 49px;
      width: 76px;
      height: 82px;
      border-radius: 37px 37px 31px 31px;
      background:
        radial-gradient(circle at 22px 43px, rgba(255, 121, 173, .37) 0 8px, transparent 8.5px),
        radial-gradient(circle at 54px 43px, rgba(255, 121, 173, .37) 0 8px, transparent 8.5px),
        #fff0e8;
      box-shadow: inset 0 -5px 0 rgba(255, 209, 203, .33);
    }

    .face::before {
      content: "";
      position: absolute;
      top: 31px;
      left: 20px;
      width: 38px;
      height: 16px;
      background:
        radial-gradient(circle at 6px 5px, #263049 0 3px, transparent 3.5px),
        radial-gradient(circle at 32px 5px, #263049 0 3px, transparent 3.5px),
        radial-gradient(ellipse at 19px 14px, var(--pink) 0 7px, transparent 7.5px);
    }

    .face::after {
      content: "";
      position: absolute;
      left: 18px;
      top: 18px;
      width: 42px;
      height: 9px;
      border-radius: 999px;
      background: linear-gradient(90deg, var(--lemon), #fff7bd, var(--lemon));
      transform: rotate(-9deg);
      box-shadow: 0 7px 14px rgba(255, 227, 123, .25);
    }

    .body {
      position: absolute;
      left: 47px;
      top: 128px;
      width: 80px;
      height: 74px;
      border: 1px solid rgba(255, 255, 255, .86);
      border-radius: 25px 25px 18px 18px;
      background:
        linear-gradient(90deg, transparent 0 42%, rgba(255, 255, 255, .68) 42% 58%, transparent 58%),
        linear-gradient(145deg, #fff, #f3f8ff);
    }

    .body::before,
    .body::after {
      content: "";
      position: absolute;
      top: 8px;
      width: 37px;
      height: 12px;
      border-radius: 999px;
      background: #fff;
    }

    .body::before { left: -28px; transform: rotate(-25deg); }
    .body::after { right: -28px; transform: rotate(25deg); }

    .layout {
      position: relative;
      z-index: 1;
      display: grid;
      grid-template-columns: 246px minmax(0, 1fr) 332px;
      gap: 16px;
      padding: 16px;
      min-height: 590px;
    }

    aside, main { min-width: 0; }

    .left, .right, .main-board {
      border: 1px solid rgba(255, 255, 255, .78);
      border-radius: 18px;
      background: var(--glass);
      box-shadow: 0 14px 34px rgba(68, 77, 112, .10);
      backdrop-filter: blur(20px);
    }

    .left, .right { padding: 14px; }
    .main-board { padding: 14px; }

    .label {
      margin: 0 0 10px;
      color: #76829a;
      font-size: 11px;
      font-weight: 820;
      letter-spacing: .10em;
      text-transform: uppercase;
    }

    .tab {
      position: relative;
      display: flex;
      align-items: center;
      gap: 10px;
      min-height: 42px;
      margin-bottom: 8px;
      padding: 0 12px;
      overflow: hidden;
      border: 1px solid rgba(45, 58, 92, .08);
      border-radius: 15px;
      color: #56637a;
      font-size: 13px;
      font-weight: 770;
      background: rgba(255, 255, 255, .56);
    }

    .tab::after {
      content: "";
      position: absolute;
      inset: 0;
      background: linear-gradient(110deg, transparent, rgba(255, 255, 255, .66), transparent);
      transform: translateX(-120%);
    }

    .tab.active {
      color: #8e3457;
      background: linear-gradient(90deg, rgba(255, 121, 173, .22), rgba(103, 200, 255, .16));
    }

    .star {
      flex: 0 0 auto;
      width: 15px;
      height: 15px;
      clip-path: polygon(50% 0, 61% 34%, 98% 35%, 68% 55%, 80% 91%, 50% 70%, 20% 91%, 32% 55%, 2% 35%, 39% 34%);
      background: linear-gradient(135deg, var(--lemon), var(--pink));
      box-shadow: 0 0 0 5px rgba(255, 227, 123, .16);
    }

    .score {
      position: relative;
      min-height: 104px;
      margin-top: 10px;
      padding: 14px;
      overflow: hidden;
      border: 1px solid rgba(255, 255, 255, .80);
      border-radius: 18px;
      background: var(--glass-strong);
      box-shadow: 0 12px 24px rgba(68, 77, 112, .08);
    }

    .score::after {
      content: "";
      position: absolute;
      right: -22px;
      top: -18px;
      width: 74px;
      height: 74px;
      border-radius: 28px;
      background: linear-gradient(135deg, rgba(103, 200, 255, .24), rgba(255, 121, 173, .20));
      transform: rotate(18deg);
    }

    .score strong, .stat strong {
      display: block;
      font-size: 31px;
      line-height: 1;
      letter-spacing: 0;
    }

    .score span, .stat label {
      display: block;
      margin-top: 8px;
      color: var(--muted);
      font-size: 12px;
      font-weight: 700;
    }

    .meter {
      position: relative;
      height: 10px;
      margin-top: 12px;
      overflow: hidden;
      border-radius: 999px;
      background: linear-gradient(90deg, var(--mint), var(--sky), var(--lemon), var(--pink));
    }

    .meter::after {
      content: "";
      position: absolute;
      inset: 0;
      background: linear-gradient(90deg, transparent, rgba(255, 255, 255, .70), transparent);
      transform: translateX(-100%);
      animation: sweep 2.6s ease-in-out infinite;
    }

    .stats {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 12px;
      margin-bottom: 14px;
    }

    .stat {
      position: relative;
      min-height: 92px;
      overflow: hidden;
      padding: 14px;
      border: 1px solid rgba(255, 255, 255, .80);
      border-radius: 18px;
      background: var(--glass-strong);
      box-shadow: 0 12px 24px rgba(68, 77, 112, .08);
    }

    .stat::before {
      content: "";
      position: absolute;
      top: -18px;
      right: 18px;
      width: 15px;
      height: 82px;
      border-radius: 999px;
      background: rgba(255, 227, 123, .72);
      transform: rotate(64deg);
    }

    .story {
      position: relative;
      margin-bottom: 14px;
      padding: 14px;
      overflow: hidden;
      border: 1px solid rgba(255, 255, 255, .80);
      border-radius: 18px;
      background:
        linear-gradient(135deg, rgba(255, 255, 255, .86), rgba(255, 255, 255, .58)),
        radial-gradient(circle at 8% 20%, rgba(98, 228, 203, .16), transparent 30%);
    }

    .steps {
      display: grid;
      grid-template-columns: repeat(5, minmax(0, 1fr));
      gap: 10px;
    }

    .step {
      position: relative;
      min-height: 92px;
      padding: 12px;
      border: 1px solid rgba(45, 58, 92, .10);
      border-radius: 15px;
      background: rgba(255, 255, 255, .70);
    }

    .step::before {
      content: "";
      position: absolute;
      top: 0;
      left: 14px;
      width: 42px;
      height: 6px;
      border-radius: 999px;
      background: linear-gradient(90deg, var(--mint), var(--sky));
      transform: rotate(-24deg);
      transform-origin: left center;
    }

    .step b {
      display: block;
      margin: 8px 0 13px;
      font-size: 14px;
    }

    .chip, .tag {
      display: inline-flex;
      max-width: 100%;
      align-items: center;
      border-radius: 999px;
      padding: 6px 9px;
      color: #23665f;
      font-size: 11px;
      font-weight: 820;
      background: rgba(98, 228, 203, .20);
      box-shadow: inset 0 0 0 1px rgba(98, 228, 203, .16);
    }

    .logs {
      display: grid;
      gap: 10px;
    }

    .log {
      position: relative;
      display: grid;
      grid-template-columns: 88px minmax(126px, .72fr) 92px minmax(0, 1fr) 82px;
      gap: 12px;
      align-items: center;
      width: 100%;
      min-height: 66px;
      padding: 0 14px;
      border: 1px solid rgba(255, 255, 255, .80);
      border-radius: 18px;
      color: inherit;
      text-align: left;
      font-size: 12px;
      cursor: pointer;
      background: rgba(255, 255, 255, .76);
      box-shadow: 0 12px 24px rgba(68, 77, 112, .08);
      transition: transform .18s ease, box-shadow .18s ease, border-color .18s ease;
    }

    .log::before {
      content: "";
      position: absolute;
      left: 0;
      top: 14px;
      bottom: 14px;
      width: 5px;
      border-radius: 0 999px 999px 0;
      background: linear-gradient(180deg, var(--pink), var(--sky));
    }

    .log.hot {
      border-color: rgba(255, 121, 173, .45);
      background: linear-gradient(90deg, rgba(255, 121, 173, .16), rgba(255, 255, 255, .88) 44%);
    }

    .log:hover {
      transform: translateY(-2px);
      box-shadow: 0 16px 32px rgba(68, 77, 112, .13);
    }

    .empty {
      display: grid;
      min-height: 164px;
      place-items: center;
      border: 1px dashed rgba(103, 144, 255, .28);
      border-radius: 18px;
      color: var(--muted);
      background: rgba(255, 255, 255, .64);
    }

    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }

    .panel {
      position: relative;
      margin-bottom: 12px;
      padding: 14px;
      border: 1px solid rgba(255, 255, 255, .80);
      border-radius: 18px;
      background: var(--glass-strong);
      box-shadow: 0 12px 24px rgba(68, 77, 112, .08);
    }

    .panel::before {
      content: "";
      position: absolute;
      right: 22px;
      top: -9px;
      width: 48px;
      height: 18px;
      border-radius: 4px;
      background: rgba(255, 227, 123, .72);
      transform: rotate(8deg);
      box-shadow: 0 8px 16px rgba(255, 177, 126, .18);
    }

    .panel h4 {
      margin: 0 0 12px;
      font-size: 16px;
      line-height: 1.15;
    }

    .kv {
      display: grid;
      grid-template-columns: 86px minmax(0, 1fr);
      gap: 8px;
      padding: 8px 0;
      border-top: 1px solid rgba(45, 58, 92, .08);
      font-size: 12px;
    }

    .kv:first-of-type { border-top: 0; }
    .kv span:first-child { color: var(--muted); font-weight: 700; }
    .kv span:last-child { min-width: 0; overflow-wrap: anywhere; }

    pre {
      margin: 0;
      max-height: 292px;
      padding: 12px;
      overflow: auto;
      border-radius: 15px;
      color: #eff7ff;
      font-size: 11px;
      line-height: 1.55;
      background:
        linear-gradient(135deg, rgba(255, 121, 173, .16), transparent 36%),
        #263147;
      box-shadow: inset 0 0 0 1px rgba(255, 255, 255, .06);
    }

    @media (prefers-reduced-motion: no-preference) {
      .visual { animation: float-panel 4.8s ease-in-out infinite; }
      .mascot { animation: mascot-hop 3.4s ease-in-out infinite; }
      .spark { animation: twinkle 2.7s ease-in-out infinite; }
      .spark.b { animation-delay: .4s; }
      .spark.c { animation-delay: .8s; }
      .live { animation: pulse 1.9s ease-in-out infinite; }
      .tab:hover::after { animation: shine .85s ease; }
      @keyframes float-panel { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-6px); } }
      @keyframes mascot-hop { 0%, 100% { transform: translateY(0) rotate(-1deg); } 50% { transform: translateY(-8px) rotate(2deg); } }
      @keyframes twinkle { 0%, 100% { transform: scale(.78) rotate(0deg); opacity: .55; } 50% { transform: scale(1.2) rotate(18deg); opacity: 1; } }
      @keyframes pulse { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.045); } }
      @keyframes sweep { 0% { transform: translateX(-100%); } 70%, 100% { transform: translateX(100%); } }
      @keyframes shine { to { transform: translateX(120%); } }
      @keyframes shimmer { 0%, 100% { opacity: .55; } 50% { opacity: .95; } }
    }

    @media (max-width: 1220px) {
      .hero { grid-template-columns: 1fr; }
      .visual { min-height: 238px; }
      .layout { grid-template-columns: 1fr; }
      .left { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
      .left .label { grid-column: 1 / -1; }
      .score { margin-top: 0; }
      .stats { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .steps { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .log { grid-template-columns: 78px 1fr 88px; }
      .log div:nth-child(4), .log div:nth-child(5) { display: none; }
    }

    @media (max-width: 700px) {
      .shell { padding: 10px; }
      .stage { border-radius: 15px; }
      .hero, .layout { padding: 10px; }
      .hero-copy { padding: 14px; }
      .hero-facts, .stats, .steps, .left { grid-template-columns: 1fr; }
      .visual { min-height: 230px; }
      .mascot { left: 64px; transform: scale(.9); transform-origin: bottom center; }
      .bubble, .pill { white-space: normal; }
      .log { grid-template-columns: 74px minmax(0, 1fr); }
      .log div:nth-child(3) { display: none; }
      h1 { font-size: 32px; }
    }
  </style>
</head>
<body>
  <div class="shell">
    <div class="stage">
      <header class="hero">
        <section class="hero-copy">
          <div class="kicker">
            <span class="live">LIVE</span>
            <span class="bubble" id="live-operation">waiting for events</span>
            <span class="bubble">ingest_interaction</span>
            <span class="bubble">trigger: conflict</span>
            <span class="bubble">write path: run_reflection</span>
          </div>
          <h1><span class="hero-line">Memory-chan</span> Live Desk</h1>
          <div class="hero-sub">
            <span class="pill">stdio MCP</span>
            <span class="pill">local sidecar</span>
            <span class="pill">read-only</span>
          </div>
          <div class="hero-facts">
            <div class="fact"><label>MCP tools</label><strong id="tool-count">0</strong></div>
            <div class="fact"><label>Events</label><strong id="event-count">0</strong></div>
            <div class="fact"><label>Status</label><strong id="status-count">ok</strong></div>
          </div>
        </section>
        <section class="visual" aria-hidden="true">
          <div class="sun-ring"></div>
          <div class="swoosh one"></div>
          <div class="swoosh two"></div>
          <div class="spark a"></div>
          <div class="spark b"></div>
          <div class="spark c"></div>
          <div class="mascot">
            <div class="hair"></div>
            <div class="face"></div>
            <div class="body"></div>
          </div>
        </section>
      </header>
      <div class="layout">
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
        <main class="main-board">
          <div class="stats">
            <div class="stat"><label>reflections</label><strong id="reflection-count">0</strong></div>
            <div class="stat"><label>decisions</label><strong id="decision-count">0</strong></div>
            <div class="stat"><label>snapshots</label><strong id="snapshot-count">0</strong></div>
            <div class="stat"><label>failed</label><strong id="failed-count">0</strong></div>
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
          <section class="logs" id="logs"><div class="empty">waiting for runtime events</div></section>
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
            <h4>Boundaries</h4>
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
      document.getElementById("decision-count").textContent = summary.decision_events;
      document.getElementById("snapshot-count").textContent = summary.snapshot_events;
      document.getElementById("failed-count").textContent = summary.failed_events;
      document.getElementById("status-count").textContent = summary.failed_events > 0 ? "check" : "ok";

      const logs = document.getElementById("logs");
      if (!events.length) {
        logs.innerHTML = '<div class="empty">waiting for runtime events</div>';
        return;
      }

      logs.innerHTML = events.map((event, index) => `
        <button class="log ${index === events.length - 1 ? "hot" : ""}" data-index="${index}">
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
