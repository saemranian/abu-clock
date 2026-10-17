//! Embedded single-page studio UI (`INDEX_HTML`) for the **Abu Clock** PC Application (`abu-clock`).
//!
//! Features:
//! - Dynamic window title bar with live version (`Abu Clock v0.0.0.13 — ...`) and custom SVG favicon
//! - Crisp vector SVG icons across all navigation tabs, status badges, KPI cards, and action buttons
//! - 1-Click Studio Scenario Presets & Live 8 Hz Auto-Sync Stream toggle
//! - Interactive 4-section Visual Help, Engineering Fundamentals & Developer SDK Hub

pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Abu Clock v0.0.0.13 — Precision Master Clock, Timecode &amp; Sync Studio</title>
  <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='%230b1325' stroke='%2338bdf8' stroke-width='4'/%3E%3Ccircle cx='32' cy='32' r='20' fill='none' stroke='%2338bdf8' stroke-width='3.5'/%3E%3Cpath d='M32 18v14l10 6' fill='none' stroke='%2310b981' stroke-width='4' stroke-linecap='round'/%3E%3Ccircle cx='32' cy='32' r='3.5' fill='%23f8fafc'/%3E%3C/svg%3E" />
  <style>
    :root {
      --bg-0: #060911;
      --bg-1: #0c1322;
      --bg-2: #121c31;
      --bg-3: #1b2844;
      --border: #243453;
      --border-hi: #38bdf8;
      --text: #f8fafc;
      --muted: #94a3b8;
      --cyan: #38bdf8;
      --emerald: #10b981;
      --amber: #f59e0b;
      --rose: #f43f5e;
      --violet: #a855f7;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      background: radial-gradient(circle at 85% 0%, #0f1d36 0%, var(--bg-0) 65%);
      color: var(--text);
      font-family: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
      min-height: 100vh;
    }
    .app-shell {
      max-width: 1520px;
      margin: 0 auto;
      padding: 14px 22px 44px;
    }
    svg.ico {
      width: 16px;
      height: 16px;
      vertical-align: -3px;
      fill: none;
      stroke: currentColor;
      stroke-width: 2;
      stroke-linecap: round;
      stroke-linejoin: round;
      flex-shrink: 0;
    }
    svg.ico-lg {
      width: 22px;
      height: 22px;
    }

    /* Toast Notification */
    #toast-banner {
      position: fixed;
      bottom: 20px;
      right: 24px;
      background: linear-gradient(135deg, #0284c7, #0f172a);
      border: 1px solid #38bdf8;
      color: #fff;
      padding: 10px 16px;
      border-radius: 10px;
      font-size: 0.84rem;
      font-weight: 600;
      box-shadow: 0 10px 28px rgba(0,0,0,0.65);
      display: flex;
      align-items: center;
      gap: 8px;
      opacity: 0;
      transform: translateY(12px);
      pointer-events: none;
      transition: all 0.2s ease;
      z-index: 9999;
    }
    #toast-banner.show {
      opacity: 1;
      transform: translateY(0);
    }

    /* Top Header */
    .top-bar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 12px;
      padding-bottom: 12px;
      border-bottom: 1px solid var(--border);
    }
    .brand-left {
      display: flex;
      align-items: center;
      gap: 13px;
    }
    .brand-logo {
      width: 42px;
      height: 42px;
      border-radius: 11px;
      background: linear-gradient(135deg, rgba(56,189,248,0.18), rgba(16,185,129,0.15));
      border: 1px solid rgba(56,189,248,0.45);
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--cyan);
      box-shadow: 0 0 20px rgba(56,189,248,0.18);
    }
    .brand-title-row {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .brand-title-row h1 {
      margin: 0;
      font-size: 1.48rem;
      letter-spacing: 0.07em;
      font-weight: 800;
      background: linear-gradient(90deg, #38bdf8, #a5b4fc);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    .version-pill {
      background: rgba(56,189,248,0.16);
      color: #7dd3fc;
      border: 1px solid rgba(56,189,248,0.45);
      padding: 2px 9px;
      border-radius: 999px;
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 0.76rem;
      font-weight: 700;
    }
    .subtitle {
      font-size: 0.8rem;
      color: var(--muted);
      margin-top: 2px;
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
    }
    .badge-row {
      display: flex;
      gap: 8px;
      align-items: center;
      flex-wrap: wrap;
    }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 5px;
      padding: 5px 11px;
      border-radius: 999px;
      font-size: 0.73rem;
      font-weight: 700;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      border: 1px solid transparent;
    }
    .badge.Locked { background: rgba(16,185,129,0.15); color: #34d399; border-color: rgba(16,185,129,0.42); }
    .badge.Locking, .badge.WarmingUp { background: rgba(245,158,11,0.15); color: #fbbf24; border-color: rgba(245,158,11,0.42); }
    .badge.Holdover { background: rgba(168,85,247,0.15); color: #c084fc; border-color: rgba(168,85,247,0.42); }
    .badge.Unlocked { background: rgba(244,63,94,0.15); color: #fb7185; border-color: rgba(244,63,94,0.42); }
    .badge.info { background: rgba(56,189,248,0.13); color: #38bdf8; border-color: rgba(56,189,248,0.35); }
    .dot-pulse {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      background: currentColor;
      box-shadow: 0 0 8px currentColor;
    }

    /* Master Digital Slate Ribbon */
    .digital-slate {
      margin-top: 12px;
      background: linear-gradient(180deg, #090e1a 0%, #050811 100%);
      border: 1px solid #1e2d4a;
      border-radius: 14px;
      padding: 15px 22px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 16px;
      box-shadow: 0 12px 30px rgba(0,0,0,0.5);
    }
    .tc-main {
      font-family: "JetBrains Mono", "Cascadia Mono", "Consolas", monospace;
      font-size: clamp(2.1rem, 4.3vw, 3.85rem);
      font-weight: 800;
      letter-spacing: 0.05em;
      color: var(--cyan);
      text-shadow: 0 0 24px rgba(56,189,248,0.32);
      line-height: 1.05;
    }
    .tc-meta {
      margin-top: 7px;
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 0.81rem;
      color: var(--muted);
      display: flex;
      gap: 14px;
      flex-wrap: wrap;
    }
    .tc-meta span {
      background: rgba(15,23,42,0.75);
      border: 1px solid #1e293b;
      padding: 3px 8px;
      border-radius: 6px;
    }
    .tc-meta span strong { color: var(--text); }
    .slate-right {
      display: flex;
      align-items: center;
      gap: 18px;
    }
    .phase-box {
      width: 185px;
      background: rgba(12,19,34,0.8);
      border: 1px solid var(--border);
      padding: 9px 12px;
      border-radius: 10px;
    }
    .phase-bar-bg {
      width: 100%;
      height: 9px;
      background: #1e293b;
      border-radius: 99px;
      overflow: hidden;
      margin-top: 5px;
    }
    .phase-bar-fill {
      height: 100%;
      background: linear-gradient(90deg, #38bdf8, #10b981);
      transition: width 0.04s linear;
    }
    .optical-beacon {
      width: 68px;
      height: 68px;
      border-radius: 12px;
      background: #0b1120;
      border: 2px solid #283756;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 3px;
      font-size: 0.64rem;
      font-weight: 800;
      color: #64748b;
      text-align: center;
      transition: all 0.04s ease;
    }
    .optical-beacon.active {
      background: #ffffff;
      color: #020617;
      border-color: #38bdf8;
      box-shadow: 0 0 36px #ffffff, 0 0 70px #38bdf8;
    }

    /* Single-Row 5-Tab Navigation Bar */
    .tabs-nav {
      display: grid;
      grid-template-columns: repeat(5, 1fr);
      gap: 8px;
      margin-top: 14px;
    }
    @media (max-width: 1100px) {
      .tabs-nav { grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); }
    }
    .tab-btn {
      background: var(--bg-1);
      color: var(--muted);
      border: 1px solid var(--border);
      padding: 10px 12px;
      border-radius: 10px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 10px;
      text-align: left;
      transition: all 0.15s ease;
    }
    .tab-btn:hover {
      color: var(--text);
      background: var(--bg-2);
      border-color: #3b5280;
    }
    .tab-btn.active {
      background: linear-gradient(135deg, rgba(56,189,248,0.18), rgba(37,99,235,0.14));
      color: var(--cyan);
      border-color: var(--cyan);
      box-shadow: 0 4px 18px rgba(56,189,248,0.15);
    }
    .tab-icon-box {
      width: 32px;
      height: 32px;
      border-radius: 8px;
      background: rgba(15,23,42,0.75);
      border: 1px solid var(--border);
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }
    .tab-btn.active .tab-icon-box {
      background: rgba(56,189,248,0.2);
      border-color: rgba(56,189,248,0.5);
      color: #fff;
    }
    .tab-title {
      font-size: 0.83rem;
      font-weight: 700;
      color: var(--text);
      display: block;
    }
    .tab-sub {
      font-size: 0.7rem;
      color: var(--muted);
      display: block;
      margin-top: 1px;
    }

    /* Quick Scenario Preset Strip */
    .preset-strip {
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
      background: rgba(12,19,34,0.75);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 9px 14px;
      margin-bottom: 14px;
    }
    .preset-label {
      font-size: 0.76rem;
      font-weight: 700;
      color: var(--cyan);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-right: 4px;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .preset-chip {
      background: var(--bg-2);
      color: #e2e8f0;
      border: 1px solid var(--border);
      border-radius: 7px;
      padding: 6px 11px;
      font-size: 0.77rem;
      font-weight: 600;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      transition: all 0.14s ease;
    }
    .preset-chip:hover {
      border-color: var(--cyan);
      background: var(--bg-3);
      color: #fff;
    }

    /* Grid & Cards */
    .tab-pane { display: none; margin-top: 14px; }
    .tab-pane.active { display: block; }
    .grid-2 {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(460px, 1fr));
      gap: 16px;
    }
    .grid-3 {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(310px, 1fr));
      gap: 14px;
    }
    .card {
      background: var(--bg-1);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 18px;
    }
    .card h2 {
      margin: 0 0 12px 0;
      font-size: 0.98rem;
      color: var(--cyan);
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 8px;
    }
    .card-title-left {
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .card h3 {
      margin: 14px 0 8px 0;
      font-size: 0.85rem;
      color: #cbd5e1;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 6px;
    }
    .section-divider {
      font-size: 0.73rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: #7dd3fc;
      margin: 12px 0 8px 0;
      padding-bottom: 4px;
      border-bottom: 1px solid rgba(36,52,83,0.7);
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .form-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 10px 14px;
    }
    label {
      display: flex;
      flex-direction: column;
      gap: 5px;
      font-size: 0.78rem;
      color: var(--muted);
      font-weight: 500;
    }
    input, select {
      background: var(--bg-0);
      color: var(--text);
      border: 1px solid var(--border);
      border-radius: 7px;
      padding: 8px 10px;
      font-size: 0.84rem;
      font-family: inherit;
    }
    input:focus, select:focus {
      outline: none;
      border-color: var(--cyan);
      box-shadow: 0 0 0 2px rgba(56,189,248,0.18);
    }
    .btn-row {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
      margin-top: 12px;
    }
    button.btn {
      background: var(--bg-3);
      color: var(--text);
      border: 1px solid #3b4f78;
      border-radius: 8px;
      padding: 8px 13px;
      font-size: 0.8rem;
      font-weight: 600;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      transition: all 0.14s ease;
    }
    button.btn:hover {
      background: #283a5e;
      border-color: var(--cyan);
    }
    button.btn.sm {
      padding: 4px 9px;
      font-size: 0.73rem;
    }
    button.btn.primary {
      background: linear-gradient(135deg, #0284c7, #2563eb);
      border-color: #38bdf8;
      color: #fff;
    }
    button.btn.primary:hover { filter: brightness(1.12); }
    button.btn.emerald {
      background: rgba(16,185,129,0.18);
      border-color: #10b981;
      color: #34d399;
    }
    button.btn.amber {
      background: rgba(245,158,11,0.18);
      border-color: #f59e0b;
      color: #fbbf24;
    }
    button.btn.rose {
      background: rgba(244,63,94,0.18);
      border-color: #f43f5e;
      color: #fb7185;
    }

    /* Telemetry KPI boxes */
    .kpi-grid {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 10px;
      margin-bottom: 12px;
    }
    @media (max-width: 700px) {
      .kpi-grid { grid-template-columns: repeat(2, 1fr); }
    }
    .kpi-box {
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 9px;
      padding: 10px 12px;
    }
    .kpi-box .kpi-label {
      font-size: 0.72rem;
      color: var(--muted);
      display: flex;
      align-items: center;
      gap: 5px;
    }
    .kpi-box .kpi-val {
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 1.08rem;
      font-weight: 700;
      margin-top: 4px;
      color: var(--text);
    }

    /* Code & Hex Blocks */
    pre.code-box {
      background: #050811;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 11px 13px;
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 0.77rem;
      color: #a5f3fc;
      overflow-x: auto;
      max-height: 240px;
      margin: 6px 0 0 0;
      line-height: 1.45;
    }
    table.data-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.79rem;
      margin-top: 8px;
    }
    table.data-table th, table.data-table td {
      border-bottom: 1px solid var(--border);
      padding: 7px 9px;
      text-align: left;
      font-family: "JetBrains Mono", "Consolas", sans-serif;
    }
    table.data-table th {
      color: var(--muted);
      font-weight: 600;
      background: rgba(15,23,42,0.6);
    }

    /* Redesigned Help Sub-Navigation & Visual Cards */
    .help-subnav {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
      margin-bottom: 14px;
      background: var(--bg-1);
      padding: 8px;
      border-radius: 10px;
      border: 1px solid var(--border);
    }
    .help-subbtn {
      background: transparent;
      color: var(--muted);
      border: 1px solid transparent;
      padding: 8px 14px;
      border-radius: 8px;
      font-size: 0.82rem;
      font-weight: 700;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 7px;
    }
    .help-subbtn:hover { color: var(--text); background: var(--bg-2); }
    .help-subbtn.active {
      background: rgba(56,189,248,0.16);
      color: var(--cyan);
      border-color: rgba(56,189,248,0.45);
    }
    .help-section { display: none; }
    .help-section.active { display: block; }
    .tour-step-card {
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 14px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      gap: 10px;
    }
    .tour-step-hdr {
      display: flex;
      align-items: center;
      gap: 10px;
      font-weight: 700;
      font-size: 0.9rem;
      color: var(--text);
    }
    .step-num {
      width: 28px;
      height: 28px;
      border-radius: 8px;
      background: linear-gradient(135deg, #38bdf8, #2563eb);
      color: #fff;
      font-weight: 800;
      font-size: 0.84rem;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }
    .tour-step-body {
      font-size: 0.82rem;
      color: #cbd5e1;
      line-height: 1.55;
    }
  </style>
</head>
<body>
  <!-- Floating Toast Feedback -->
  <div id="toast-banner">
    <svg class="ico" viewBox="0 0 24 24"><path d="M20 6L9 17l-5-5"/></svg>
    <span id="toast-text">Settings applied to Rust engine</span>
  </div>

  <div class="app-shell">
    <!-- Top Header -->
    <header class="top-bar">
      <div class="brand-left">
        <div class="brand-logo" title="Abu Clock Master Precision Oscillators">
          <svg class="ico ico-lg" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="9"/>
            <polyline points="12 7 12 12 15.5 14"/>
          </svg>
        </div>
        <div>
          <div class="brand-title-row">
            <h1>ABU CLOCK</h1>
            <span id="hdr-version" class="version-pill">v0.0.0.13</span>
          </div>
          <div class="subtitle">
            <span>Precision Master Clock, Timecode &amp; Multi-Sensor Sync Studio</span>
            <span>&bull;</span>
            <span>Oscillator: <strong id="hdr-backend" style="color:#e2e8f0;">Windows::QueryPerformanceCounter</strong></span>
            <span>&bull;</span>
            <span>Socket: <strong id="hdr-udp" style="color:#38bdf8;">127.0.0.1:50319 (TX: 0 / RX: 0)</strong></span>
          </div>
        </div>
      </div>

      <div class="badge-row">
        <span id="badge-lock" class="badge Locked">
          <span class="dot-pulse"></span>
          <span id="badge-lock-text">LOCKED</span>
        </span>
        <span id="badge-port" class="badge info">
          <svg class="ico" viewBox="0 0 24 24"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>
          <span id="badge-port-text">PORT: LISTENING</span>
        </span>
        <span id="badge-transport" class="badge info">
          <svg class="ico" viewBox="0 0 24 24"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
          <span id="badge-transport-text">WIRED LAN</span>
        </span>
        <span id="badge-wakelock" class="badge Locked">
          <svg class="ico" viewBox="0 0 24 24"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
          <span id="badge-wakelock-text">WAKE-LOCK ON</span>
        </span>
      </div>
    </header>

    <!-- Master SMPTE ST 12-1 Digital Slate -->
    <section class="digital-slate" aria-label="Master SMPTE ST 12-1 Digital Slate">
      <div>
        <div id="slate-tc" class="tc-main">00:00:00;00.00</div>
        <div class="tc-meta">
          <span>Frame <strong id="slate-frame">#0</strong></span>
          <span>BCD (32b) <strong id="slate-bcd">0x00000000</strong></span>
          <span>Rate <strong id="slate-rate">29.970 Hz (DF)</strong></span>
          <span>PTP Domain <strong id="slate-domain">1</strong></span>
          <span>TAI (ns) <strong id="slate-tai">1729180800000000000</strong></span>
        </div>
      </div>
      <div class="slate-right">
        <div class="phase-box">
          <div style="display:flex;justify-content:space-between;font-size:0.74rem;color:var(--muted);">
            <span>Sub-Frame Phase (&phi;)</span>
            <strong id="slate-phase-text" style="color:var(--cyan);">0.000</strong>
          </div>
          <div class="phase-bar-bg">
            <div id="slate-phase-bar" class="phase-bar-fill" style="width:0%"></div>
          </div>
          <div style="display:flex;justify-content:space-between;font-size:0.71rem;color:var(--muted);margin-top:6px;">
            <span>User Phase Trim:</span>
            <strong id="slate-trim" style="color:var(--text);">0 ns</strong>
          </div>
        </div>
        <div id="optical-beacon" class="optical-beacon" title="Optical Sync Flash Beacon: Flashes pure white on Frame 00 for high-speed camera validation">
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/></svg>
          <span>FRAME 00<br/>FLASH</span>
        </div>
      </div>
    </section>

    <!-- Single-Row 5-Tab Navigation Bar with Vector Icons -->
    <nav class="tabs-nav">
      <button id="nav-tab-dashboard" class="tab-btn active" onclick="switchTab('tab-dashboard', this)">
        <div class="tab-icon-box">
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        </div>
        <div>
          <span class="tab-title">1. Clock &amp; Kalman PLL</span>
          <span class="tab-sub">Settings, Presets &amp; Servo Scope</span>
        </div>
      </button>

      <button id="nav-tab-ptp" class="tab-btn" onclick="switchTab('tab-ptp', this)">
        <div class="tab-icon-box">
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
        </div>
        <div>
          <span class="tab-title">2. PTP &amp; ST 2059-2 TLV</span>
          <span class="tab-sub">BMCA Failover &amp; 20B Wire Hex</span>
        </div>
      </button>

      <button id="nav-tab-trigger" class="tab-btn" onclick="switchTab('tab-trigger', this)">
        <div class="tab-icon-box">
          <svg class="ico" viewBox="0 0 24 24"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>
        </div>
        <div>
          <span class="tab-title">3. Trigger &amp; Telemetry</span>
          <span class="tab-sub">+50ms Barrier, FreeD &amp; Manifest</span>
        </div>
      </button>

      <button id="nav-tab-appliance" class="tab-btn" onclick="switchTab('tab-appliance', this)">
        <div class="tab-icon-box">
          <svg class="ico" viewBox="0 0 24 24"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
        </div>
        <div>
          <span class="tab-title">4. Grandmaster Appliance</span>
          <span class="tab-sub">Pi CM4 + DS3231 TCXO + PHC</span>
        </div>
      </button>

      <button id="nav-tab-help" class="tab-btn" onclick="switchTab('tab-help', this)">
        <div class="tab-icon-box">
          <svg class="ico" viewBox="0 0 24 24"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg>
        </div>
        <div>
          <span class="tab-title">5. Visual Guide &amp; SDK</span>
          <span class="tab-sub">Interactive Manual &amp; Dev Docs</span>
        </div>
      </button>
    </nav>

    <!-- TAB 1: DASHBOARD, SETTINGS & KALMAN PLL -->
    <div id="tab-dashboard" class="tab-pane active">
      <!-- 1-Click Scenario Presets -->
      <div class="preset-strip">
        <span class="preset-label">
          <svg class="ico" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
          1-Click Studio Presets:
        </span>
        <button class="preset-chip" onclick="applyStudioPreset('Master','SmpteSt2059_2',1,64,128,'24,1,false',false,'Virtual Production (24 fps Cinema + ST 2059-2 GM)')">
          <svg class="ico" viewBox="0 0 24 24"><rect x="2" y="2" width="20" height="20" rx="2.18"/><line x1="7" y1="2" x2="7" y2="22"/><line x1="17" y1="2" x2="17" y2="22"/></svg>
          Virtual Production (24 fps + ST 2059-2)
        </button>
        <button class="preset-chip" onclick="applyStudioPreset('AutoBmca','SmpteSt2059_2',1,128,128,'30000,1001,true',false,'Broadcast NTSC (29.97 Drop-Frame + Domain 1)')">
          <svg class="ico" viewBox="0 0 24 24"><rect x="2" y="7" width="20" height="15" rx="2"/><polyline points="17 2 12 7 7 2"/></svg>
          Broadcast NTSC (29.97 DF)
        </button>
        <button class="preset-chip" onclick="applyStudioPreset('Master','Ieee802_1AsGptp',0,64,128,'10,1,false',false,'LiDAR Array (10 Hz Sweep + IEEE 802.1AS gPTP)')">
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>
          RoboSense / XGRIDS LiDAR (10 Hz gPTP)
        </button>
        <button class="preset-chip" onclick="applyStudioPreset('AutoBmca','AbuSyncCsptp',1,128,128,'60,1,false',false,'UE5 / NVIDIA Omniverse (60 fps + AbuSync CSPTP)')">
          <svg class="ico" viewBox="0 0 24 24"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>
          UE5 &amp; Omniverse (60 fps)
        </button>
        <button class="preset-chip" onclick="applyStudioPreset('Follower','AbuSyncCsptp',1,128,128,'30000,1001,true',true,'Mobile Wi-Fi Follower Rig (AbuSync UDP 50319 + Wi-Fi EMA)')">
          <svg class="ico" viewBox="0 0 24 24"><rect x="5" y="2" width="14" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
          Mobile Wi-Fi Follower Rig
        </button>
      </div>

      <div class="grid-2">
        <!-- Parameter Settings Card -->
        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="1" y1="14" x2="7" y2="14"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="17" y1="16" x2="23" y2="16"/></svg>
              Master / Follower &amp; Timing Parameter Settings
            </span>
            <span class="badge info">LIVE RUST ENGINE</span>
          </h2>

          <div class="section-divider">
            <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/></svg>
            Network Role, Profile &amp; BMCA Identity
          </div>
          <div class="form-grid">
            <label>
              Operating Role (BMCA Mode)
              <select id="cfg-role">
                <option value="AutoBmca">Auto (IEEE 1588 BMCA Election)</option>
                <option value="Master">Forced Grandmaster</option>
                <option value="Follower">Follower Only</option>
              </select>
            </label>
            <label>
              Synchronization Profile
              <select id="cfg-profile">
                <option value="AbuSyncCsptp">AbuSync Unprivileged CSPTP (UDP 50319/50320)</option>
                <option value="SmpteSt2059_2">SMPTE ST 2059-2 Broadcast (224.0.1.129:319/320)</option>
                <option value="Ieee802_1AsGptp">IEEE 802.1AS gPTP LiDAR (224.0.0.107 L2 P2P)</option>
                <option value="Aes67Harmonized">AES67 + ST 2059-2 Harmonized (AES-R16)</option>
              </select>
            </label>
            <label>
              PTP Domain (0–127, Default 1)
              <input id="cfg-domain" type="number" min="0" max="127" value="1" />
            </label>
            <label>
              BMCA Priority 1 / Priority 2
              <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
                <input id="cfg-prio1" type="number" min="0" max="255" value="128" title="BMCA Priority 1 (Lower wins)" />
                <input id="cfg-prio2" type="number" min="0" max="255" value="128" title="BMCA Priority 2 (Tiebreaker)" />
              </div>
            </label>
          </div>

          <div class="section-divider">
            <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
            SMPTE ST 12-1 Timecode &amp; Barrier Cadence
          </div>
          <div class="form-grid">
            <label>
              Rational Frame Rate / LiDAR Cadence
              <select id="cfg-fps">
                <option value="24000,1001,false">23.976 fps (24000/1001 NDF)</option>
                <option value="24,1,false">24.000 fps (True Cinema NDF)</option>
                <option value="25,1,false">25.000 fps (PAL Broadcast NDF)</option>
                <option value="30000,1001,true" selected>29.97 fps Drop-Frame (30000/1001 DF)</option>
                <option value="30000,1001,false">29.97 fps Non-Drop (30000/1001 NDF)</option>
                <option value="30,1,false">30.000 fps (NDF)</option>
                <option value="50,1,false">50.000 fps (PAL HFR NDF)</option>
                <option value="60000,1001,true">59.94 fps Drop-Frame (60000/1001 DF)</option>
                <option value="60000,1001,false">59.94 fps Non-Drop (60000/1001 NDF)</option>
                <option value="60,1,false">60.000 fps (UE5 / Omniverse NDF)</option>
                <option value="120,1,false">120.000 fps (High-Frame-Rate Slate)</option>
                <option value="10,1,false">10.000 Hz (RoboSense E1R / XGRIDS LiDAR)</option>
              </select>
            </label>
            <label>
              User Phase Trim Offset (ns)
              <input id="cfg-offset-ns" type="number" step="100" value="0" />
            </label>
            <label>
              Barrier Trigger Lead Time (ms)
              <input id="cfg-lead-ms" type="number" min="2" max="2000" value="50" />
            </label>
            <label>
              Network Medium &amp; Wake-Lock
              <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
                <select id="cfg-wireless" title="Transport Medium">
                  <option value="false">Wired LAN (&lt;100ns)</option>
                  <option value="true">Wi-Fi (&lt;500&mu;s)</option>
                </select>
                <select id="cfg-wakelock" title="Screen Wake-Lock">
                  <option value="true">Wake-Lock ON</option>
                  <option value="false">Wake-Lock OFF</option>
                </select>
              </div>
            </label>
          </div>

          <div class="btn-row">
            <button class="btn primary" onclick="applyConfig()">
              <svg class="ico" viewBox="0 0 24 24"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/></svg>
              Apply Settings to Rust Engine
            </button>
            <button id="btn-autosync" class="btn emerald" onclick="clockAction('toggle_autosync', 'Toggled 8 Hz Live Servo Stream')">
              <svg class="ico" viewBox="0 0 24 24"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
              <span id="btn-autosync-text">8 Hz Auto-Sync Stream: ON</span>
            </button>
          </div>
        </section>

        <!-- Kalman PLL & Oscilloscope Card -->
        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
              2-State Kalman PLL &amp; 7.0s ST 2059 Warm-Up Gate
            </span>
            <span id="kalman-gate-label" class="badge Locked">7.0S WARM-UP: 100%</span>
          </h2>
          <div class="kpi-grid">
            <div class="kpi-box">
              <div class="kpi-label">
                <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
                Phase Offset (&theta;)
              </div>
              <div id="kpi-offset" class="kpi-val">0 ns</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">
                <svg class="ico" viewBox="0 0 24 24"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/></svg>
                Crystal Drift (&alpha;)
              </div>
              <div id="kpi-drift" class="kpi-val">0.00 ppb</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">
                <svg class="ico" viewBox="0 0 24 24"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
                One-Way Delay (d)
              </div>
              <div id="kpi-delay" class="kpi-val">380 ns</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">
                <svg class="ico" viewBox="0 0 24 24"><path d="M12 20V10M18 20V4M6 20v-4"/></svg>
                Kalman Variance
              </div>
              <div id="kpi-var" class="kpi-val">0.0 ns&sup2;</div>
            </div>
          </div>

          <!-- Live SVG Oscilloscope -->
          <div style="background:#050811;border:1px solid var(--border);border-radius:9px;padding:10px;">
            <div style="display:flex;justify-content:space-between;font-size:0.74rem;color:var(--muted);margin-bottom:4px;">
              <span>Live Phase Offset Oscilloscope (&theta; in ns, 40-sample sliding window)</span>
              <span id="osc-latest-text" style="color:var(--cyan);font-weight:700;">Latest: 0 ns</span>
            </div>
            <svg id="osc-svg" viewBox="0 0 600 130" style="width:100%;height:130px;display:block;">
              <line x1="0" y1="30" x2="600" y2="30" stroke="#111a2e" stroke-width="1" />
              <line x1="0" y1="65" x2="600" y2="65" stroke="#1e293b" stroke-dasharray="4 4" />
              <line x1="0" y1="100" x2="600" y2="100" stroke="#111a2e" stroke-width="1" />
              <polyline id="osc-line" fill="none" stroke="#38bdf8" stroke-width="2.5" points="0,65 600,65" />
            </svg>
          </div>

          <h3>
            <span>Interactive Clock Discipline &amp; Fault Injection Test Bench</span>
          </h3>
          <div class="btn-row">
            <button class="btn emerald" onclick="clockAction('tick_clean', 'Injected Clean Wired PTP Sample (±9 ns)')">
              <svg class="ico" viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12"/></svg>
              Inject Clean PTP Tick (&plusmn;9 ns)
            </button>
            <button class="btn amber" onclick="clockAction('tick_wifi', 'Injected Wi-Fi Delay Spike (+1,850 ns)')">
              <svg class="ico" viewBox="0 0 24 24"><path d="M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>
              Inject Wi-Fi Spike (+1,850 ns)
            </button>
            <button class="btn amber" onclick="clockAction('step_50us', 'Injected +50 µs Phase Disturbance')">
              <svg class="ico" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
              Inject +50 &mu;s Disturbance
            </button>
            <button class="btn primary" onclick="clockAction('fast_forward_7s', 'Completed 7.0s ST 2059 Warm-Up Gate -> LOCKED')">
              <svg class="ico" viewBox="0 0 24 24"><polygon points="13 19 22 12 13 5 13 19"/><polygon points="2 19 11 12 2 5 2 19"/></svg>
              Fast-Forward 7.0s Warm-Up (Converge &amp; Lock)
            </button>
            <button class="btn" onclick="clockAction('holdover', 'Entered Oscillator Holdover State')">
              <svg class="ico" viewBox="0 0 24 24"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
              Simulate Holdover
            </button>
            <button class="btn rose" onclick="clockAction('unlock', 'Simulated Clock Unlock Fault')">
              <svg class="ico" viewBox="0 0 24 24"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
              Simulate Unlock Fault
            </button>
          </div>
        </section>
      </div>
    </div>

    <!-- TAB 2: PTP ENGINE & SMPTE ST 2059-2 TLV INSPECTOR -->
    <div id="tab-ptp" class="tab-pane">
      <div class="grid-2">
        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
              IEEE 1588-2019 Best Master Clock Algorithm (BMCA) Simulator
            </span>
            <span id="bmca-port-badge" class="badge info">PORT: LISTENING</span>
          </h2>
          <p style="font-size:0.82rem;color:var(--muted);margin-top:0;">
            Inject a simulated remote <code>Announce</code> frame carrying a SMPTE ST 2059-2 TLV.
            Lower values win in IEEE 1588 BMCA: if the remote node has a lower <code>Priority 1</code> (e.g. <code>64</code> vs <code>128</code>),
            <code>DualModePtpEngine</code> transitions to <code>SLAVE</code> and automatically locks to the remote SMPTE frame rate.
          </p>
          <div class="form-grid">
            <label>
              Remote Priority 1 (e.g. 64 beats 128)
              <input id="bmca-prio1" type="number" min="0" max="255" value="64" />
            </label>
            <label>
              Remote Clock Class (6 = GNSS GM, 248 = Default)
              <input id="bmca-class" type="number" min="1" max="255" value="6" />
            </label>
            <label>
              Remote Priority 2
              <input id="bmca-prio2" type="number" min="0" max="255" value="128" />
            </label>
            <label>
              Remote Advertised ST 2059-2 Frame Rate
              <select id="bmca-fps">
                <option value="24,1,false">24.000 fps (True Cinema NDF)</option>
                <option value="25,1,false">25.000 fps (PAL Broadcast NDF)</option>
                <option value="30000,1001,true">29.97 fps Drop-Frame (DF)</option>
                <option value="60,1,false">60.000 fps (UE5 / Omniverse NDF)</option>
              </select>
            </label>
          </div>
          <div class="btn-row">
            <button class="btn primary" onclick="runBmcaTest()">
              <svg class="ico" viewBox="0 0 24 24"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/></svg>
              Inject Remote Announce &amp; Run BMCA Election
            </button>
          </div>
          <h3>BMCA Election Audit Log</h3>
          <pre id="bmca-log-box" class="code-box"></pre>
        </section>

        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
              Live SMPTE ST 2059-2 Synchronization Metadata TLV Inspector
            </span>
            <span class="badge Locked">20 Bytes Big-Endian</span>
          </h2>
          <div style="display:flex;justify-content:space-between;align-items:center;">
            <span style="font-size:0.81rem;color:var(--muted);">
              Live wire dump of <code>SmpteSyncMetadataTlv::to_bytes()</code>:
            </span>
            <button class="btn sm" onclick="copyElementText('tlv-hex-box', 'Copied 20-byte ST 2059-2 TLV hex')">
              <svg class="ico" viewBox="0 0 24 24"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
              Copy Hex
            </button>
          </div>
          <pre id="tlv-hex-box" class="code-box" style="font-size:0.95rem;color:#38bdf8;font-weight:700;"></pre>
          <table class="data-table">
            <thead>
              <tr>
                <th>Byte Offset</th>
                <th>Field Name</th>
                <th>Live Decoded Value</th>
              </tr>
            </thead>
            <tbody>
              <tr><td>Bytes 0..1</td><td>tlvType</td><td id="tlv-type">0x0003 (Organization Extension)</td></tr>
              <tr><td>Bytes 2..3</td><td>lengthField</td><td>0x0010 (16 bytes payload)</td></tr>
              <tr><td>Bytes 4..6</td><td>organizationId</td><td id="tlv-org">00-0E-9B (SMPTE OUI)</td></tr>
              <tr><td>Bytes 7..9</td><td>organizationSubType</td><td id="tlv-sub">00-00-01 (ST 2059-2 SM)</td></tr>
              <tr><td>Bytes 10..11</td><td>currentUtcOffset</td><td id="tlv-utc">+37 s (TAI - UTC)</td></tr>
              <tr><td>Bytes 12..15</td><td>defaultSystemFrameRate (Num)</td><td id="tlv-num">30000</td></tr>
              <tr><td>Bytes 16..18</td><td>defaultSystemFrameRate (Den 24-bit)</td><td id="tlv-den">1001</td></tr>
              <tr><td>Byte 19</td><td>timecodeFlags</td><td id="tlv-flags">0x01 (Drop-Frame)</td></tr>
            </tbody>
          </table>
        </section>
      </div>
    </div>

    <!-- TAB 3: BARRIER TRIGGER, TELEMETRY & MANIFEST LAB -->
    <div id="tab-trigger" class="tab-pane">
      <div class="grid-2">
        <!-- Barrier Trigger Card -->
        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>
              Multi-Sensor Barrier Trigger (<code>SCHEDULE_CAPTURE_AT</code>)
            </span>
            <span class="badge Locked">Sub-50&mu;s Spinlock</span>
          </h2>
          <p style="font-size:0.81rem;color:var(--muted);margin-top:0;">
            Eliminates Wi-Fi/LAN packet transit jitter by broadcasting a future target TAI nanosecond ($T_{\text{now}} + \Delta T_{\text{lead}}$).
            Every rig sleeps until $T_{\text{exec}} - 500\ \mu\text{s}$ and spins on its hardware monotonic counter to trigger at the exact same instant.
          </p>
          <div class="btn-row">
            <button class="btn primary" onclick="fireTrigger('single', 1)">
              <svg class="ico" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
              Fire SCHEDULE_CAPTURE_AT (+Lead Time)
            </button>
            <button class="btn emerald" onclick="fireTrigger('burst', 5)">
              <svg class="ico" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
              Schedule 5-Frame Synchronized Burst Scan
            </button>
          </div>
          <h3>Latest Multi-Rig Barrier Receipts</h3>
          <div id="trigger-summary" style="font-size:0.8rem;color:var(--cyan);margin-bottom:6px;font-family:monospace;"></div>
          <table class="data-table">
            <thead>
              <tr>
                <th>Sensor Node</th>
                <th>Platform</th>
                <th>Jitter (ns)</th>
                <th>Deterministic Filename</th>
              </tr>
            </thead>
            <tbody id="trigger-receipts-body"></tbody>
          </table>
        </section>

        <!-- 6-DoF Camera Telemetry Card -->
        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>
              6-DoF Camera Tracking Telemetry (OpenTrackIO &amp; FreeD)
            </span>
            <span id="telem-seq-badge" class="badge info">SEQ #1</span>
          </h2>
          <div class="form-grid">
            <label>Camera ID (0–255)<input id="pose-cam" type="number" value="1" /></label>
            <label>Pan (&deg;)<input id="pose-pan" type="number" step="0.5" value="12.5" /></label>
            <label>Tilt (&deg;)<input id="pose-tilt" type="number" step="0.5" value="-4.25" /></label>
            <label>Roll (&deg;)<input id="pose-roll" type="number" step="0.25" value="0.15" /></label>
            <label>Pos X (mm)<input id="pose-x" type="number" step="10" value="1250" /></label>
            <label>Pos Y (mm)<input id="pose-y" type="number" step="10" value="-400" /></label>
            <label>Pos Z (mm)<input id="pose-z" type="number" step="10" value="1700" /></label>
            <label>Lens Zoom / Focus<input id="pose-zoom" type="number" value="2048" /></label>
          </div>
          <div class="btn-row">
            <button class="btn primary" onclick="updateTelemetryPose()">
              <svg class="ico" viewBox="0 0 24 24"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
              Synthesize OpenTrackIO &amp; FreeD Packets
            </button>
          </div>
          <h3>
            <span>Standard 29-Byte FreeD (<code>0xD1</code>) + Checksum</span>
            <button class="btn sm" onclick="copyElementText('freed-d1-box', 'Copied 29-byte FreeD 0xD1 hex')">Copy</button>
          </h3>
          <pre id="freed-d1-box" class="code-box"></pre>
          <h3>
            <span>Extended 41-Byte AbuTrack FreeD-TC (<code>0xD2</code>)</span>
            <button class="btn sm" onclick="copyElementText('freed-d2-box', 'Copied 41-byte FreeD-TC 0xD2 hex')">Copy</button>
          </h3>
          <pre id="freed-d2-box" class="code-box"></pre>
          <h3>
            <span>SMPTE RIS OpenTrackIO (<code>OTrk</code>) JSON Payload</span>
            <button class="btn sm" onclick="copyElementText('otrk-json-box', 'Copied OpenTrackIO JSON')">Copy</button>
          </h3>
          <pre id="otrk-json-box" class="code-box"></pre>
        </section>
      </div>

      <!-- Session Manifest Exporter -->
      <section class="card" style="margin-top:16px;">
        <h2>
          <span class="card-title-left">
            <svg class="ico" viewBox="0 0 24 24"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
            Synchronized Session Manifest Exporter (<code>.abuscan.json</code> &amp; Colmap <code>images.txt</code>)
          </span>
          <span class="badge info">RealityCapture / 3DGS / Postshot Ready</span>
        </h2>
        <div class="btn-row" style="margin-top:0;margin-bottom:10px;">
          <input id="manifest-sensor" type="text" value="SonyA1_Rig03" style="width:180px;" placeholder="Sensor ID" />
          <input id="manifest-ext" type="text" value="arw" style="width:80px;" placeholder="Ext" />
          <button class="btn emerald" onclick="addManifestRecord()">
            <svg class="ico" viewBox="0 0 24 24"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
            Stamp &amp; Append Sensor Capture to Manifest
          </button>
        </div>
        <div class="grid-2">
          <div>
            <h3>
              <span><code>.abuscan.json</code> Session Manifest</span>
              <button class="btn sm" onclick="copyElementText('manifest-json-box', 'Copied .abuscan.json manifest')">Copy JSON</button>
            </h3>
            <pre id="manifest-json-box" class="code-box"></pre>
          </div>
          <div>
            <h3>
              <span>Colmap / Jawset Postshot <code>images.txt</code></span>
              <button class="btn sm" onclick="copyElementText('manifest-colmap-box', 'Copied Colmap images.txt')">Copy Colmap</button>
            </h3>
            <pre id="manifest-colmap-box" class="code-box"></pre>
          </div>
        </div>
      </section>
    </div>

    <!-- TAB 4: GRANDMASTER HARDWARE APPLIANCE MONITOR -->
    <div id="tab-appliance" class="tab-pane">
      <div class="grid-2">
        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/></svg>
              Phase 3 Hardware Grandmaster Appliance Monitor
            </span>
            <span id="app-lock-badge" class="badge Locked">GNSS / TCXO LOCKED (CLASS 6)</span>
          </h2>
          <p style="font-size:0.83rem;color:var(--muted);margin-top:0;">
            Monitors and simulates the dedicated <strong>Raspberry Pi CM4 / Pi 5 + Maxim DS3231 TCXO (&plusmn;2 ppm) + Intel I210-T1 PCIe PHC</strong>
            hardware Grandmaster appliance (<code>abu-daemon --appliance-mode</code>).
          </p>
          <div class="kpi-grid">
            <div class="kpi-box">
              <div class="kpi-label">PTP Clock Class</div>
              <div id="app-class" class="kpi-val">Class 6</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">Clock Accuracy</div>
              <div id="app-acc" class="kpi-val">0x20 (&lt;25ns)</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">Linux RT Priority</div>
              <div id="app-fifo" class="kpi-val">SCHED_FIFO 80</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">TAI - UTC Offset</div>
              <div id="app-tai-offset" class="kpi-val">+37 sec</div>
            </div>
          </div>
          <div class="btn-row">
            <button class="btn emerald" onclick="setApplianceLock(true)">
              <svg class="ico" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
              Lock Primary Reference (GNSS / DS3231 TCXO &rarr; Class 6)
            </button>
            <button class="btn amber" onclick="setApplianceLock(false)">
              <svg class="ico" viewBox="0 0 24 24"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/></svg>
              Simulate Antenna Loss / Holdover Mode (TCXO Holdover &rarr; Class 7)
            </button>
          </div>
        </section>

        <section class="card">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/></svg>
              Hardware PHC Devices &amp; Simultaneous Multicast Profiles
            </span>
          </h2>
          <table class="data-table">
            <tbody>
              <tr><th>Primary Studio PHC</th><td id="app-phc0">/dev/ptp0</td></tr>
              <tr><th>Secondary LiDAR PHC</th><td id="app-phc1">/dev/ptp1</td></tr>
              <tr><th>Battery TCXO RTC</th><td id="app-rtc">/dev/rtc0</td></tr>
              <tr><th>Studio LAN Interface</th><td id="app-eth0">eth0</td></tr>
              <tr><th>Isolated LiDAR Port</th><td id="app-eth1">eth1</td></tr>
              <tr><th>Seeded PHC TAI (ns)</th><td id="app-tai-seed">0</td></tr>
            </tbody>
          </table>
          <h3>Active Simultaneous Broadcast Profiles</h3>
          <pre id="app-profiles-box" class="code-box"></pre>
        </section>
      </div>
    </div>

    <!-- TAB 5: REDESIGNED INTERACTIVE HELP, VISUAL FUNDAMENTALS & DEVELOPER SDK HUB -->
    <div id="tab-help" class="tab-pane">
      <!-- Sub-navigation pills inside Help -->
      <div class="help-subnav">
        <button class="help-subbtn active" onclick="switchHelpSub('help-sub-tour', this)">
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/></svg>
          1. Interactive Visual Studio Tour (User Manual)
        </button>
        <button class="help-subbtn" onclick="switchHelpSub('help-sub-fundamentals', this)">
          <svg class="ico" viewBox="0 0 24 24"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
          2. Core Engineering Fundamentals &amp; Diagrams
        </button>
        <button class="help-subbtn" onclick="switchHelpSub('help-sub-dev', this)">
          <svg class="ico" viewBox="0 0 24 24"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
          3. Developer SDK &amp; Engine Integration Guide
        </button>
        <button class="help-subbtn" onclick="switchHelpSub('help-sub-check', this)">
          <svg class="ico" viewBox="0 0 24 24"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
          4. Studio Operator Checklist &amp; Troubleshooting
        </button>
      </div>

      <!-- HELP SUB-TAB 1: INTERACTIVE VISUAL STUDIO TOUR -->
      <div id="help-sub-tour" class="help-section active">
        <section class="card" style="margin-bottom:16px;">
          <h2>
            <span class="card-title-left">
              <svg class="ico" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>
              Annotated Visual Map of the Abu Clock Studio Interface
            </span>
            <span class="badge info">Interactive UI Overview</span>
          </h2>
          <div style="background:#050811;border:1px solid var(--border);border-radius:10px;padding:14px;overflow-x:auto;">
            <svg viewBox="0 0 1060 270" style="width:100%;min-width:760px;height:auto;display:block;">
              <!-- Region 1: Top Slate -->
              <rect x="15" y="12" width="1030" height="66" rx="10" fill="#0d1629" stroke="#38bdf8" stroke-width="2"/>
              <circle cx="42" cy="45" r="14" fill="#38bdf8"/><text x="37" y="50" fill="#020617" font-size="14" font-weight="bold">1</text>
              <text x="68" y="38" fill="#38bdf8" font-size="14" font-weight="bold">MASTER DIGITAL SLATE RIBBON (Always Visible)</text>
              <text x="68" y="58" fill="#94a3b8" font-size="12">Displays live SMPTE ST 12-1 Timecode, 32-bit BCD word, Sub-Frame Phase (&phi;), TAI nanoseconds &amp; Frame-00 Optical Flash Beacon.</text>

              <!-- Region 2: Presets & Params -->
              <rect x="15" y="92" width="505" height="80" rx="10" fill="#0d1629" stroke="#10b981" stroke-width="2"/>
              <circle cx="42" cy="132" r="14" fill="#10b981"/><text x="37" y="137" fill="#020617" font-size="14" font-weight="bold">2</text>
              <text x="68" y="122" fill="#34d399" font-size="13" font-weight="bold">TAB 1 LEFT: STUDIO PRESETS &amp; PARAMETER SETTINGS</text>
              <text x="68" y="142" fill="#94a3b8" font-size="11">1-Click Presets (VP 24fps, Broadcast 29.97 DF, LiDAR 10Hz, UE5 60fps)</text>
              <text x="68" y="158" fill="#94a3b8" font-size="11">+ Role, Profile, Domain, Priority 1/2, Phase Trim &amp; Lead Time.</text>

              <!-- Region 3: Kalman Scope -->
              <rect x="540" y="92" width="505" height="80" rx="10" fill="#0d1629" stroke="#f59e0b" stroke-width="2"/>
              <circle cx="567" cy="132" r="14" fill="#f59e0b"/><text x="562" y="137" fill="#020617" font-size="14" font-weight="bold">3</text>
              <text x="593" y="122" fill="#fbbf24" font-size="13" font-weight="bold">TAB 1 RIGHT: 2-STATE KALMAN PLL &amp; LIVE OSCILLOSCOPE</text>
              <text x="593" y="142" fill="#94a3b8" font-size="11">8 Hz real-time phase offset trace (&theta; in ns), crystal drift (&alpha; in ppb),</text>
              <text x="593" y="158" fill="#94a3b8" font-size="11">7.0s ST 2059 warm-up gate &amp; 1-click jitter/fault injection buttons.</text>

              <!-- Region 4, 5, 6 -->
              <rect x="15" y="186" width="330" height="72" rx="10" fill="#0d1629" stroke="#a855f7" stroke-width="2"/>
              <circle cx="42" cy="222" r="14" fill="#a855f7"/><text x="37" y="227" fill="#020617" font-size="14" font-weight="bold">4</text>
              <text x="66" y="215" fill="#c084fc" font-size="13" font-weight="bold">TAB 2: PTP &amp; ST 2059-2 TLV</text>
              <text x="66" y="235" fill="#94a3b8" font-size="11">BMCA Election Simulator &amp; 20B Hex Decoder</text>

              <rect x="365" y="186" width="330" height="72" rx="10" fill="#0d1629" stroke="#38bdf8" stroke-width="2"/>
              <circle cx="392" cy="222" r="14" fill="#38bdf8"/><text x="387" y="227" fill="#020617" font-size="14" font-weight="bold">5</text>
              <text x="416" y="215" fill="#38bdf8" font-size="13" font-weight="bold">TAB 3: BARRIER &amp; TELEMETRY</text>
              <text x="416" y="235" fill="#94a3b8" font-size="11">+50ms Trigger, FreeD/OTrk &amp; .abuscan.json</text>

              <rect x="715" y="186" width="330" height="72" rx="10" fill="#0d1629" stroke="#10b981" stroke-width="2"/>
              <circle cx="742" cy="222" r="14" fill="#10b981"/><text x="737" y="227" fill="#020617" font-size="14" font-weight="bold">6</text>
              <text x="766" y="215" fill="#34d399" font-size="13" font-weight="bold">TAB 4: HARDWARE APPLIANCE</text>
              <text x="766" y="235" fill="#94a3b8" font-size="11">Pi CM4 + DS3231 TCXO Class 6/7 Monitor</text>
            </svg>
          </div>
        </section>

        <!-- 6 Interactive Step Cards with "Try It Now" Jump Buttons -->
        <div class="grid-3">
          <div class="tour-step-card">
            <div>
              <div class="tour-step-hdr">
                <span class="step-num">1</span>
                <span>Read the Master Digital Slate</span>
              </div>
              <p class="tour-step-body">
                The top banner runs continuously regardless of which tab is open. Notice the semicolon <code>;</code> in <code>HH:MM:SS;FF.sf</code> when Drop-Frame is active vs. colon <code>:</code> for Non-Drop-Frame, and watch the <strong>FRAME 00 FLASH</strong> beacon pulse white at the start of every second.
              </p>
            </div>
            <button class="btn sm primary" onclick="document.getElementById('nav-tab-dashboard').click()">
              View Slate &amp; Dashboard &rarr;
            </button>
          </div>

          <div class="tour-step-card">
            <div>
              <div class="tour-step-hdr">
                <span class="step-num">2</span>
                <span>Apply a 1-Click Studio Preset</span>
              </div>
              <p class="tour-step-body">
                In <strong>Tab 1</strong>, click any of the 5 Studio Preset chips (e.g. <em>Virtual Production 24 fps</em> or <em>RoboSense LiDAR 10 Hz</em>) or customize Role, Profile, Domain, and Frame Rate manually and click <strong>Apply Settings to Rust Engine</strong>.
              </p>
            </div>
            <button class="btn sm emerald" onclick="document.getElementById('nav-tab-dashboard').click(); applyStudioPreset('Master','SmpteSt2059_2',1,64,128,'24,1,false',false,'Virtual Production (24 fps Cinema + ST 2059-2 GM)')">
              Try 24 fps Cinema Preset Live &rarr;
            </button>
          </div>

          <div class="tour-step-card">
            <div>
              <div class="tour-step-hdr">
                <span class="step-num">3</span>
                <span>Perturb &amp; Lock the Kalman Servo</span>
              </div>
              <p class="tour-step-body">
                In <strong>Tab 1 Right</strong>, inject a <code>+50 &mu;s Disturbance</code> or <code>Unlock Fault</code> to see how the 2-state Kalman PLL reacts on the live SVG oscilloscope, then click <strong>Fast-Forward 7.0s Warm-Up</strong> to re-lock below <code>100 ns</code>.
              </p>
            </div>
            <button class="btn sm amber" onclick="document.getElementById('nav-tab-dashboard').click(); clockAction('step_50us', 'Injected +50 µs Disturbance')">
              Inject +50 &mu;s Disturbance Now &rarr;
            </button>
          </div>

          <div class="tour-step-card">
            <div>
              <div class="tour-step-hdr">
                <span class="step-num">4</span>
                <span>Run BMCA Election &amp; Inspect TLV</span>
              </div>
              <p class="tour-step-body">
                In <strong>Tab 2</strong>, inspect the live 20-byte big-endian <code>SmpteSyncMetadataTlv</code> hex packet (`00 03 00 10 00 0E 9B...`) and simulate a remote Grandmaster <code>Announce</code> to verify automatic slave failover.
              </p>
            </div>
            <button class="btn sm primary" onclick="document.getElementById('nav-tab-ptp').click()">
              Open PTP &amp; TLV Inspector &rarr;
            </button>
          </div>

          <div class="tour-step-card">
            <div>
              <div class="tour-step-hdr">
                <span class="step-num">5</span>
                <span>Fire +50ms Multi-Rig Barrier Capture</span>
              </div>
              <p class="tour-step-body">
                In <strong>Tab 3</strong>, click <strong>Fire SCHEDULE_CAPTURE_AT</strong> to schedule a deterministic multi-sensor capture across iOS, Windows, LiDAR, and UE5 rigs, and synthesize <strong>FreeD (`0xD1`/`0xD2`)</strong> &amp; <strong>OpenTrackIO</strong> packets.
              </p>
            </div>
            <button class="btn sm emerald" onclick="document.getElementById('nav-tab-trigger').click(); fireTrigger('single', 1)">
              Fire +50ms Barrier Capture Now &rarr;
            </button>
          </div>

          <div class="tour-step-card">
            <div>
              <div class="tour-step-hdr">
                <span class="step-num">6</span>
                <span>Test Hardware Appliance Holdover</span>
              </div>
              <p class="tour-step-body">
                In <strong>Tab 4</strong>, monitor the Raspberry Pi CM4 + Maxim DS3231 TCXO + Intel I210-T1 PHC Grandmaster appliance and test switching between <code>Class 6 (Locked)</code> and <code>Class 7 (Holdover)</code>.
              </p>
            </div>
            <button class="btn sm primary" onclick="document.getElementById('nav-tab-appliance').click()">
              Open Appliance Monitor &rarr;
            </button>
          </div>
        </div>
      </div>

      <!-- HELP SUB-TAB 2: CORE ENGINEERING FUNDAMENTALS & VISUAL DIAGRAMS -->
      <div id="help-sub-fundamentals" class="help-section">
        <div class="grid-2">
          <!-- DiagramCard A: Barrier Trigger Timeline Comparison -->
          <section class="card">
            <h2>
              <span class="card-title-left">
                <svg class="ico" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
                Why Future-Time Barrier Triggering Eliminates Network Jitter
              </span>
            </h2>
            <p style="font-size:0.82rem;color:#cbd5e1;line-height:1.55;">
              Wi-Fi and Ethernet switches introduce variable packet transit delay (`0.3 ms` to `15 ms`).
              Instead of broadcasting "capture now", <code>AbuScanCoordinator</code> broadcasts <code>ScheduleCaptureAt(T_now + 50ms)</code> so every node counts down against its local disciplined <code>OverlayClock</code>:
            </p>
            <div style="background:#050811;border:1px solid var(--border);border-radius:10px;padding:12px;">
              <svg viewBox="0 0 540 220" style="width:100%;height:auto;display:block;">
                <!-- Naive Trigger -->
                <text x="12" y="22" fill="#fb7185" font-size="12" font-weight="bold">&#10060; Naive "Capture Now" UDP Broadcast (15 ms Shutter Skew):</text>
                <line x1="20" y1="45" x2="520" y2="45" stroke="#334155" stroke-width="2"/>
                <circle cx="40" cy="45" r="5" fill="#38bdf8"/>
                <text x="25" y="64" fill="#94a3b8" font-size="10">Master TX</text>
                <circle cx="150" cy="45" r="6" fill="#fb7185"/>
                <text x="120" y="34" fill="#fb7185" font-size="10">LAN Cam (+0.4ms)</text>
                <circle cx="390" cy="45" r="6" fill="#fb7185"/>
                <text x="340" y="34" fill="#fb7185" font-size="10">Wi-Fi iPhone (+14.2ms MISMATCH!)</text>

                <!-- Abu Clock Barrier Trigger -->
                <text x="12" y="112" fill="#34d399" font-size="12" font-weight="bold">&#9989; Abu Clock SCHEDULE_CAPTURE_AT (T_exec = T_now + 50ms):</text>
                <line x1="20" y1="155" x2="520" y2="155" stroke="#334155" stroke-width="2"/>
                <circle cx="40" cy="155" r="5" fill="#38bdf8"/>
                <text x="22" y="175" fill="#94a3b8" font-size="10">T_now</text>

                <!-- Arrival markers -->
                <path d="M40 155 Q95 125 140 155" fill="none" stroke="#38bdf8" stroke-dasharray="3 3"/>
                <path d="M40 155 Q170 115 290 155" fill="none" stroke="#f59e0b" stroke-dasharray="3 3"/>
                <text x="145" y="145" fill="#94a3b8" font-size="9">Coarse Sleep &rarr; 500&mu;s QPC Spinlock</text>

                <!-- Exact Barrier Line -->
                <line x1="450" y1="120" x2="450" y2="195" stroke="#10b981" stroke-width="3"/>
                <circle cx="450" cy="145" r="6" fill="#34d399"/>
                <circle cx="450" cy="165" r="6" fill="#34d399"/>
                <text x="335" y="208" fill="#34d399" font-size="11" font-weight="bold">ALL RIGS FIRE AT T_exec (&lt; 50 &mu;s spread)</text>
              </svg>
            </div>
          </section>

          <!-- DiagramCard B: Kalman PLL & 7s Gate -->
          <section class="card">
            <h2>
              <span class="card-title-left">
                <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                User-Space OverlayClock &amp; 2-State Kalman PLL Equation
              </span>
            </h2>
            <p style="font-size:0.82rem;color:#cbd5e1;line-height:1.55;">
              <code>OverlayClock</code> never steps your OS kernel clock. It wraps the hardware monotonic counter
              (<code>QueryPerformanceCounter</code> / <code>mach_continuous_time</code> / <code>CLOCK_MONOTONIC_RAW</code>)
              using lock-free atomic sequence locking:
            </p>
            <pre class="code-box" style="color:#34d399;font-size:0.82rem;">T_TAI(t) = T_anchor + (C_mono(t) - C_mono(t0)) * (1 + alpha_freq) + theta_phase</pre>
            <div style="background:#050811;border:1px solid var(--border);border-radius:10px;padding:12px;margin-top:10px;">
              <svg viewBox="0 0 540 135" style="width:100%;height:auto;display:block;">
                <rect x="10" y="35" width="110" height="55" rx="8" fill="#1e1b2e" stroke="#f43f5e"/>
                <text x="26" y="60" fill="#fb7185" font-size="11" font-weight="bold">UNLOCKED</text>
                <text x="22" y="76" fill="#94a3b8" font-size="9">Offset &gt; Threshold</text>

                <line x1="120" y1="62" x2="155" y2="62" stroke="#94a3b8" stroke-width="2"/>
                <rect x="155" y="35" width="135" height="55" rx="8" fill="#231f10" stroke="#f59e0b"/>
                <text x="172" y="60" fill="#fbbf24" font-size="11" font-weight="bold">WARMING UP</text>
                <text x="168" y="76" fill="#94a3b8" font-size="9">0.0s .. 7.0s ST 2059 Gate</text>

                <line x1="290" y1="62" x2="325" y2="62" stroke="#10b981" stroke-width="2"/>
                <rect x="325" y="35" width="105" height="55" rx="8" fill="#0c241d" stroke="#10b981"/>
                <text x="352" y="60" fill="#34d399" font-size="11" font-weight="bold">LOCKED</text>
                <text x="336" y="76" fill="#94a3b8" font-size="9">&lt;100ns (Wired)</text>

                <line x1="430" y1="62" x2="455" y2="62" stroke="#a855f7" stroke-width="2"/>
                <rect x="455" y="35" width="78" height="55" rx="8" fill="#1f1433" stroke="#a855f7"/>
                <text x="462" y="60" fill="#c084fc" font-size="10" font-weight="bold">HOLDOVER</text>
                <text x="462" y="76" fill="#94a3b8" font-size="8">Drift &alpha; Held</text>
              </svg>
            </div>
          </section>
        </div>
      </div>

      <!-- HELP SUB-TAB 3: DEVELOPER SDK & INTEGRATION HUB -->
      <div id="help-sub-dev" class="help-section">
        <div class="grid-2">
          <section class="card">
            <h2>
              <span class="card-title-left">
                <svg class="ico" viewBox="0 0 24 24"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
                C / C++17 &amp; Python / Omniverse FFI Integration
              </span>
              <span class="badge info">target/release/abuclock.dll</span>
            </h2>
            <p style="font-size:0.81rem;color:#cbd5e1;margin-top:0;">
              Link against <code>crates/abu-ffi/include/abu_clock.h</code> or use the C++17 RAII wrapper in
              <code>integrations/abu-track-scan/windows/AbuClockWinClient.hpp</code>:
            </p>
            <pre class="code-box">// C / C++17 Zero-Overhead FFI Example
#include "abu_clock.h"

AbuClockOpaqueHandle* clk = abu_clock_create(1729180800000000000ULL, 30000, 1001, true);
AbuClockCFrameSnapshot snap;
if (abu_clock_synthesize_now(clk, &amp;snap)) {
    // snap.bcd_timecode (32-bit BCD), snap.tai_ns, snap.phase
}
uint8_t freed_d1[29];
abu_clock_pack_freed_d1(clk, &amp;pose, freed_d1);
abu_clock_destroy(clk);</pre>

            <h3>Python &amp; NVIDIA Omniverse Extension (<code>omni.abu.clock</code>)</h3>
            <pre class="code-box">from abu_clock import AbuClockClient
client = AbuClockClient("target/release/abuclock.dll")
client.tick_network(offset_ns=12.0, delay_ns=380.0)
print("Live SMPTE:", client.now_smpte(), "| Locked:", client.status())</pre>
          </section>

          <section class="card">
            <h2>
              <span class="card-title-left">
                <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/></svg>
                Local HTTP JSON Control API (<code>127.0.0.1:7878</code>)
              </span>
              <span class="badge Locked">CORS Enabled</span>
            </h2>
            <p style="font-size:0.81rem;color:#cbd5e1;margin-top:0;">
              External scripts, StreamDeck buttons, or studio automation tools can query and control the running <strong>Abu Clock</strong> PC application via REST JSON:
            </p>
            <table class="data-table">
              <thead>
                <tr><th>Endpoint</th><th>Method</th><th>Description</th></tr>
              </thead>
              <tbody>
                <tr><td><code>/api/state</code></td><td>GET</td><td>Full snapshot of Clock, SMPTE, PTP TLV, Receipts &amp; Manifests</td></tr>
                <tr><td><code>/api/config</code></td><td>POST</td><td>Update Role, Profile, Domain, FrameRate, Trim &amp; Lead Time</td></tr>
                <tr><td><code>/api/clock</code></td><td>POST</td><td>Inject PTP samples, fast-forward 7s gate, or toggle 8Hz auto-sync</td></tr>
                <tr><td><code>/api/bmca</code></td><td>POST</td><td>Inject remote Announce frame &amp; trigger IEEE 1588 BMCA election</td></tr>
                <tr><td><code>/api/trigger</code></td><td>POST</td><td>Broadcast <code>SCHEDULE_CAPTURE_AT</code> single or burst trigger</td></tr>
                <tr><td><code>/api/telemetry</code></td><td>POST</td><td>Update 6-DoF camera pose &amp; synthesize OpenTrackIO / FreeD</td></tr>
                <tr><td><code>/api/manifest</code></td><td>POST</td><td>Append synchronized sensor record to <code>.abuscan.json</code></td></tr>
              </tbody>
            </table>
            <h3>Repository Documentation Files</h3>
            <pre class="code-box">docs/USER_MANUAL.md              # Complete Studio Operator Manual
docs/DEVELOPER_GUIDE.md          # Complete Developer &amp; SDK Integration Guide
docs/ABU_CLOCK_USER_GUIDE.md     # Fundamentals &amp; Visual Architecture Reference
docs/RUNNING_AND_TESTING_GUIDE.md# CLI &amp; Test Verification Commands</pre>
          </section>
        </div>
      </div>

      <!-- HELP SUB-TAB 4: OPERATOR CHECKLIST & TROUBLESHOOTING -->
      <div id="help-sub-check" class="help-section">
        <div class="grid-2">
          <section class="card">
            <h2>Studio Pre-Shoot Synchronization Checklist</h2>
            <table class="data-table">
              <thead>
                <tr><th>Step</th><th>Check Item</th><th>Target / Expected Value</th></tr>
              </thead>
              <tbody>
                <tr><td>1</td><td>Verify Oscillator Backend</td><td><code>Windows::QueryPerformanceCounter</code> or <code>CLOCK_MONOTONIC_RAW</code></td></tr>
                <tr><td>2</td><td>Confirm 7.0s ST 2059 Warm-Up</td><td>Top-right badge shows green <code>LOCKED</code> (100%)</td></tr>
                <tr><td>3</td><td>Verify Frame Rate &amp; DF Flag</td><td>Matches camera project rate (e.g. <code>29.97 DF</code> uses <code>;</code>)</td></tr>
                <tr><td>4</td><td>Check Phase Offset (&theta;)</td><td><code>&lt; 100 ns</code> on Wired LAN / <code>&lt; 500 &mu;s</code> on Wi-Fi</td></tr>
                <tr><td>5</td><td>Fire Test Barrier Capture</td><td>All sensor receipts in Tab 3 show green jitter within budget</td></tr>
              </tbody>
            </table>
          </section>

          <section class="card">
            <h2>Common Troubleshooting Solutions</h2>
            <ul style="font-size:0.83rem;color:#cbd5e1;line-height:1.65;padding-left:18px;margin:0;">
              <li><strong>Privileged PTP Ports (319/320) Blocked on Windows/macOS:</strong> Use the default <code>AbuSync Unprivileged CSPTP (UDP 50319/50320)</code> profile — it requires zero Administrator/root privileges and automatically binds via <code>TimestampedUdpSocket::bind_dual_mode</code>.</li>
              <li><strong>Clock Status Stuck in <code>WARMING UP</code> or <code>UNLOCKED</code>:</strong> Ensure <code>8 Hz Auto-Sync Stream</code> is enabled in Tab 1 or click <strong>Fast-Forward 7.0s Warm-Up</strong> to complete the mandatory SMPTE ST 2059-2 7-second convergence window.</li>
              <li><strong>Timecode Drift Against External NLE / Audio Recorder:</strong> Check whether your NLE timeline is Drop-Frame (<code>29.97 DF</code>, semicolon <code>;</code>) vs. Non-Drop-Frame (<code>29.97 NDF</code>, colon <code>:</code>).</li>
            </ul>
          </section>
        </div>
      </div>
    </div>
  </div>

  <script>
    const API_BASE = "http://127.0.0.1:7878";
    let formInitialized = false;
    let toastTimer = null;

    function showToast(msg) {
      const banner = document.getElementById('toast-banner');
      document.getElementById('toast-text').textContent = msg;
      banner.classList.add('show');
      if (toastTimer) clearTimeout(toastTimer);
      toastTimer = setTimeout(() => banner.classList.remove('show'), 2600);
    }

    function copyElementText(elId, msg) {
      const text = document.getElementById(elId).textContent || '';
      if (navigator.clipboard) {
        navigator.clipboard.writeText(text);
      }
      showToast(msg || 'Copied to clipboard');
    }

    function switchTab(tabId, btnEl) {
      document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
      document.getElementById(tabId).classList.add('active');
      btnEl.classList.add('active');
    }

    function switchHelpSub(subId, btnEl) {
      document.querySelectorAll('.help-section').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.help-subbtn').forEach(el => el.classList.remove('active'));
      document.getElementById(subId).classList.add('active');
      btnEl.classList.add('active');
    }

    function syncFormFields(snap) {
      const activeEl = document.activeElement;
      const isEditingForm = activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'SELECT');
      if (isEditingForm && formInitialized) return;

      document.getElementById('cfg-role').value = snap.role;
      document.getElementById('cfg-profile').value = snap.profile;
      document.getElementById('cfg-domain').value = snap.ptp_domain;
      document.getElementById('cfg-prio1').value = snap.priority1;
      document.getElementById('cfg-prio2').value = snap.priority2;
      document.getElementById('cfg-offset-ns').value = snap.user_offset_ns;
      document.getElementById('cfg-lead-ms').value = snap.trigger_lead_time_ms;
      document.getElementById('cfg-wireless').value = String(snap.convergence.is_wireless);
      document.getElementById('cfg-wakelock').value = String(snap.keep_screen_on_active);

      const fpsKey = `${snap.frame_rate_num},${snap.frame_rate_den},${snap.drop_frame}`;
      const fpsSelect = document.getElementById('cfg-fps');
      if ([...fpsSelect.options].some(o => o.value === fpsKey)) {
        fpsSelect.value = fpsKey;
      }
      formInitialized = true;
    }

    function renderSnapshot(snap) {
      if (!snap || !snap.version) return;

      // Update top white native window title bar with version number
      const expectedTitle = `Abu Clock ${snap.version} — Precision Master Clock, Timecode & Sync Studio`;
      if (document.title !== expectedTitle) {
        document.title = expectedTitle;
      }

      document.getElementById('hdr-version').textContent = snap.version;
      document.getElementById('hdr-backend').textContent = snap.monotonic_backend;
      if (snap.udp_socket) {
        document.getElementById('hdr-udp').textContent =
          `${snap.udp_socket.bound_addr} (TX: ${snap.udp_socket.tx_packets} / RX: ${snap.udp_socket.rx_packets})`;
      }

      // Badges
      const lockBadge = document.getElementById('badge-lock');
      document.getElementById('badge-lock-text').textContent = snap.convergence.lock_state.toUpperCase();
      lockBadge.className = 'badge ' + snap.convergence.lock_state;

      document.getElementById('badge-port-text').textContent = 'PORT: ' + String(snap.ptp_port_state).toUpperCase();
      document.getElementById('badge-transport-text').textContent = snap.convergence.is_wireless ? 'WIRELESS Wi-Fi' : 'WIRED LAN';
      document.getElementById('badge-wakelock-text').textContent = snap.keep_screen_on_active ? 'WAKE-LOCK ON' : 'WAKE-LOCK OFF';

      // Auto-sync button indicator
      const autoBtn = document.getElementById('btn-autosync');
      const autoTxt = document.getElementById('btn-autosync-text');
      if (snap.auto_sync_enabled) {
        autoTxt.textContent = '8 Hz Auto-Sync Stream: ON';
        autoBtn.className = 'btn emerald';
      } else {
        autoTxt.textContent = '8 Hz Auto-Sync Stream: PAUSED';
        autoBtn.className = 'btn amber';
      }

      // Master Digital Slate
      document.getElementById('slate-tc').textContent = snap.slate_timecode;
      document.getElementById('slate-frame').textContent = '#' + snap.frame_index;
      document.getElementById('slate-bcd').textContent = snap.bcd_hex;
      document.getElementById('slate-rate').textContent =
        snap.frame_rate_hz.toFixed(3) + ' Hz (' + (snap.drop_frame ? 'DF' : 'NDF') + ')';
      document.getElementById('slate-domain').textContent = snap.ptp_domain;
      document.getElementById('slate-tai').textContent = snap.tai_ns;
      document.getElementById('slate-phase-text').textContent = snap.phase.toFixed(3);
      document.getElementById('slate-phase-bar').style.width = (snap.phase * 100).toFixed(1) + '%';
      document.getElementById('slate-trim').textContent = snap.user_offset_ns + ' ns';

      const beacon = document.getElementById('optical-beacon');
      if (snap.flash_marker_active) {
        beacon.classList.add('active');
      } else {
        beacon.classList.remove('active');
      }

      syncFormFields(snap);

      // Kalman KPIs
      document.getElementById('kpi-offset').textContent = snap.convergence.offset_ns + ' ns';
      document.getElementById('kpi-drift').textContent = snap.convergence.drift_ppb.toFixed(2) + ' ppb';
      document.getElementById('kpi-delay').textContent = snap.convergence.path_delay_ns + ' ns';
      document.getElementById('kpi-var').textContent = snap.convergence.offset_variance_ns2.toFixed(1) + ' ns²';

      const gateLabel = document.getElementById('kalman-gate-label');
      gateLabel.textContent = '7.0S WARM-UP: ' + (snap.convergence.warmup_progress * 100).toFixed(0) + '%';
      gateLabel.className = 'badge ' + snap.convergence.lock_state;

      // Oscilloscope polyline
      const hist = snap.offset_history_ns || [];
      if (hist.length > 0) {
        document.getElementById('osc-latest-text').textContent = 'Latest: ' + hist[hist.length - 1] + ' ns';
        const maxAbs = Math.max(40, ...hist.map(v => Math.abs(v)));
        const pts = hist.map((val, idx) => {
          const x = (idx / Math.max(1, hist.length - 1)) * 600;
          const y = 65 - (val / maxAbs) * 50;
          return `${x.toFixed(1)},${y.toFixed(1)}`;
        }).join(' ');
        document.getElementById('osc-line').setAttribute('points', pts);
      }

      // Tab 2: PTP & ST 2059-2 TLV
      document.getElementById('bmca-port-badge').textContent = 'PORT: ' + String(snap.ptp_port_state).toUpperCase();
      document.getElementById('bmca-log-box').textContent = snap.last_bmca_log;
      if (snap.smpte_tlv) {
        document.getElementById('tlv-hex-box').textContent = snap.smpte_tlv.wire_hex;
        document.getElementById('tlv-type').textContent = snap.smpte_tlv.tlv_type_hex;
        document.getElementById('tlv-org').textContent = snap.smpte_tlv.org_id_hex;
        document.getElementById('tlv-sub').textContent = snap.smpte_tlv.org_subtype_hex;
        document.getElementById('tlv-utc').textContent = '+' + snap.smpte_tlv.current_utc_offset_sec + ' s (TAI - UTC)';
        document.getElementById('tlv-num').textContent = snap.smpte_tlv.default_system_frame_rate_num;
        document.getElementById('tlv-den').textContent = snap.smpte_tlv.default_system_frame_rate_den;
        document.getElementById('tlv-flags').textContent =
          snap.smpte_tlv.timecode_flags_hex + (snap.smpte_tlv.drop_frame ? ' (Drop-Frame)' : ' (Non-Drop-Frame)');
      }

      // Tab 3: Trigger, Telemetry & Manifest
      const logs = snap.capture_logs || [];
      if (logs.length > 0) {
        const latest = logs[0];
        document.getElementById('trigger-summary').textContent =
          `Seq #${latest.sequence_id} | ${latest.mode_label} | Target TC: ${latest.smpte_timecode} (${latest.bcd_hex}) | Target TAI: ${latest.target_tai_ns}`;
        const tbody = document.getElementById('trigger-receipts-body');
        tbody.innerHTML = latest.receipts.map(r => `
          <tr>
            <td><strong>${r.sensor_id}</strong></td>
            <td>${r.platform}</td>
            <td style="color:${r.within_subframe_budget ? '#34d399' : '#fb7185'};font-weight:700;">${r.jitter_ns >= 0 ? '+' : ''}${r.jitter_ns} ns</td>
            <td><code>${r.filename}</code></td>
          </tr>
        `).join('');
      }

      if (snap.telemetry) {
        document.getElementById('telem-seq-badge').textContent = 'SEQ #' + snap.telemetry.sequence_num;
        document.getElementById('freed-d1-box').textContent =
          `${snap.telemetry.freed_d1_hex}\n[Verified Modulo-256 Checksum Byte 28 = ${snap.telemetry.freed_d1_checksum_hex}]`;
        document.getElementById('freed-d2-box').textContent =
          `${snap.telemetry.freed_d2_hex}\n[Verified Modulo-256 Checksum Byte 40 = ${snap.telemetry.freed_d2_checksum_hex}]`;
        document.getElementById('otrk-json-box').textContent =
          `// Wire Header (14B OTrk): ${snap.telemetry.opentrackio_wire_hex_prefix} (${snap.telemetry.opentrackio_total_bytes} bytes total)\n` +
          snap.telemetry.opentrackio_json;
      }

      document.getElementById('manifest-json-box').textContent = snap.manifest_abuscan_json || '';
      document.getElementById('manifest-colmap-box').textContent = snap.manifest_colmap_txt || '';

      // Tab 4: Appliance
      if (snap.appliance) {
        const appBadge = document.getElementById('app-lock-badge');
        if (snap.appliance.gnss_or_tcxo_locked) {
          appBadge.textContent = 'GNSS / TCXO LOCKED (CLASS 6)';
          appBadge.className = 'badge Locked';
        } else {
          appBadge.textContent = 'HOLDOVER ACTIVE (CLASS 7)';
          appBadge.className = 'badge Holdover';
        }
        document.getElementById('app-class').textContent = 'Class ' + snap.appliance.ptp_clock_class;
        document.getElementById('app-acc').textContent = snap.appliance.ptp_clock_accuracy_hex;
        document.getElementById('app-fifo').textContent = 'SCHED_FIFO ' + snap.appliance.sched_fifo_priority;
        document.getElementById('app-tai-offset').textContent = '+' + snap.appliance.tai_minus_utc_seconds + ' sec';
        document.getElementById('app-phc0').textContent = snap.appliance.primary_phc_device;
        document.getElementById('app-phc1').textContent = snap.appliance.secondary_lidar_phc_device;
        document.getElementById('app-rtc').textContent = snap.appliance.rtc_device;
        document.getElementById('app-eth0').textContent = snap.appliance.studio_interface;
        document.getElementById('app-eth1').textContent = snap.appliance.lidar_interface;
        document.getElementById('app-tai-seed').textContent = snap.appliance.seeded_phc_tai_ns;
        document.getElementById('app-profiles-box').textContent = snap.appliance.active_profiles.join('\n');
      }
    }

    async function pollState() {
      try {
        const res = await fetch(API_BASE + '/api/state');
        const snap = await res.json();
        renderSnapshot(snap);
      } catch (e) {}
    }

    async function postApi(path, payload, toastMsg) {
      try {
        const res = await fetch(API_BASE + path, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        const snap = await res.json();
        formInitialized = false;
        renderSnapshot(snap);
        if (toastMsg) showToast(toastMsg);
      } catch (e) {}
    }

    function applyConfig() {
      const fpsParts = document.getElementById('cfg-fps').value.split(',');
      postApi('/api/config', {
        role: document.getElementById('cfg-role').value,
        profile: document.getElementById('cfg-profile').value,
        ptp_domain: Number(document.getElementById('cfg-domain').value),
        priority1: Number(document.getElementById('cfg-prio1').value),
        priority2: Number(document.getElementById('cfg-prio2').value),
        fps_num: Number(fpsParts[0]),
        fps_den: Number(fpsParts[1]),
        drop_frame: fpsParts[2] === 'true',
        user_offset_ns: Number(document.getElementById('cfg-offset-ns').value),
        trigger_lead_time_ms: Number(document.getElementById('cfg-lead-ms').value),
        is_wireless: document.getElementById('cfg-wireless').value === 'true',
        keep_screen_on_active: document.getElementById('cfg-wakelock').value === 'true',
        auto_sync_enabled: true
      }, 'Applied timing & profile configuration to Rust engine');
    }

    function applyStudioPreset(role, profile, domain, prio1, prio2, fpsStr, wireless, label) {
      const fpsParts = fpsStr.split(',');
      postApi('/api/config', {
        role,
        profile,
        ptp_domain: domain,
        priority1: prio1,
        priority2: prio2,
        fps_num: Number(fpsParts[0]),
        fps_den: Number(fpsParts[1]),
        drop_frame: fpsParts[2] === 'true',
        user_offset_ns: 0,
        trigger_lead_time_ms: 50,
        is_wireless: wireless,
        keep_screen_on_active: true,
        auto_sync_enabled: true
      }, 'Loaded Preset: ' + label);
    }

    function clockAction(action, toastMsg) {
      postApi('/api/clock', { action }, toastMsg);
    }

    function runBmcaTest() {
      const fpsParts = document.getElementById('bmca-fps').value.split(',');
      postApi('/api/bmca', {
        remote_priority1: Number(document.getElementById('bmca-prio1').value),
        remote_clock_class: Number(document.getElementById('bmca-class').value),
        remote_priority2: Number(document.getElementById('bmca-prio2').value),
        remote_fps_num: Number(fpsParts[0]),
        remote_fps_den: Number(fpsParts[1]),
        remote_drop_frame: fpsParts[2] === 'true'
      }, 'Executed IEEE 1588-2019 BMCA Election');
    }

    function fireTrigger(mode, frames) {
      postApi('/api/trigger', { mode, frames },
        mode === 'burst' ? 'Scheduled 5-Frame Synchronized Burst Scan' : 'Broadcast SCHEDULE_CAPTURE_AT (+Lead Time)');
    }

    function updateTelemetryPose() {
      postApi('/api/telemetry', {
        camera_id: Number(document.getElementById('pose-cam').value),
        pan_deg: Number(document.getElementById('pose-pan').value),
        tilt_deg: Number(document.getElementById('pose-tilt').value),
        roll_deg: Number(document.getElementById('pose-roll').value),
        x_mm: Number(document.getElementById('pose-x').value),
        y_mm: Number(document.getElementById('pose-y').value),
        z_mm: Number(document.getElementById('pose-z').value),
        zoom_raw: Number(document.getElementById('pose-zoom').value),
        focus_raw: 4096
      }, 'Synthesized OpenTrackIO & FreeD 0xD1/0xD2 packets');
    }

    function addManifestRecord() {
      postApi('/api/manifest', {
        sensor_id: document.getElementById('manifest-sensor').value,
        ext: document.getElementById('manifest-ext').value
      }, 'Appended capture record to .abuscan.json & Colmap manifest');
    }

    function setApplianceLock(gnss_locked) {
      postApi('/api/appliance', { gnss_locked },
        gnss_locked ? 'Appliance Locked to Primary GNSS/TCXO (Class 6)' : 'Appliance Switched to TCXO Holdover (Class 7)');
    }

    pollState();
    setInterval(pollState, 50);
  </script>
</body>
</html>
"##;
