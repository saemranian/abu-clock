//! Embedded single-page studio UI (`INDEX_HTML`) for the **Abu Clock** PC Application (`abu-clock`).
//!
//! Provides a unified dark-mode Broadcast & Virtual Production Control Center with 5 interactive tabs:
//! 1. Clock, Parameters & Kalman PLL Test Bench
//! 2. PTP Engine, BMCA Simulator & SMPTE ST 2059-2 TLV Hex Inspector
//! 3. Multi-Sensor Barrier Trigger (`SCHEDULE_CAPTURE_AT`), OpenTrackIO/FreeD Telemetry & Manifest Exporter
//! 4. Phase 3 Hardware Grandmaster Appliance Monitor (`Pi CM4 + DS3231 TCXO + PHC`)
//! 5. Built-in Help, Core Fundamentals & Graphical Step-by-Step User Guide

pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Abu Clock — Precision Master Clock, Timecode & Sync Studio</title>
  <style>
    :root {
      --bg-0: #060911;
      --bg-1: #0d1322;
      --bg-2: #131c31;
      --bg-3: #1c2942;
      --border: #263554;
      --text: #f1f5f9;
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
      background: radial-gradient(circle at top right, #0f172a 0%, var(--bg-0) 70%);
      color: var(--text);
      font-family: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
      min-height: 100vh;
    }
    .app-shell {
      max-width: 1500px;
      margin: 0 auto;
      padding: 16px 22px 40px;
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
    .brand-title {
      display: flex;
      align-items: center;
      gap: 12px;
    }
    .brand-title h1 {
      margin: 0;
      font-size: 1.55rem;
      letter-spacing: 0.08em;
      font-weight: 800;
      background: linear-gradient(90deg, #38bdf8, #818cf8);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    .subtitle {
      font-size: 0.82rem;
      color: var(--muted);
    }
    .badge-row {
      display: flex;
      gap: 8px;
      align-items: center;
      flex-wrap: wrap;
    }
    .badge {
      padding: 5px 11px;
      border-radius: 999px;
      font-size: 0.74rem;
      font-weight: 700;
      letter-spacing: 0.05em;
      text-transform: uppercase;
      border: 1px solid transparent;
    }
    .badge.Locked { background: rgba(16,185,129,0.16); color: #34d399; border-color: rgba(16,185,129,0.4); }
    .badge.WarmingUp { background: rgba(245,158,11,0.16); color: #fbbf24; border-color: rgba(245,158,11,0.4); }
    .badge.Holdover { background: rgba(168,85,247,0.16); color: #c084fc; border-color: rgba(168,85,247,0.4); }
    .badge.Unlocked { background: rgba(244,63,94,0.16); color: #fb7185; border-color: rgba(244,63,94,0.4); }
    .badge.info { background: rgba(56,189,248,0.14); color: #38bdf8; border-color: rgba(56,189,248,0.35); }

    /* Master Digital Slate Ribbon */
    .digital-slate {
      margin-top: 14px;
      background: linear-gradient(180deg, #090d18 0%, #05070e 100%);
      border: 2px solid #1e293b;
      border-radius: 14px;
      padding: 16px 24px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 18px;
      box-shadow: 0 12px 32px rgba(0,0,0,0.55);
    }
    .tc-main {
      font-family: "JetBrains Mono", "Cascadia Mono", "Consolas", monospace;
      font-size: clamp(2.2rem, 4.6vw, 4.1rem);
      font-weight: 800;
      letter-spacing: 0.05em;
      color: var(--cyan);
      text-shadow: 0 0 24px rgba(56,189,248,0.35);
      line-height: 1.05;
    }
    .tc-meta {
      margin-top: 6px;
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 0.84rem;
      color: var(--muted);
      display: flex;
      gap: 16px;
      flex-wrap: wrap;
    }
    .tc-meta span strong { color: var(--text); }
    .slate-right {
      display: flex;
      align-items: center;
      gap: 20px;
    }
    .phase-box {
      width: 170px;
    }
    .phase-bar-bg {
      width: 100%;
      height: 10px;
      background: #1e293b;
      border-radius: 99px;
      overflow: hidden;
      margin-top: 4px;
    }
    .phase-bar-fill {
      height: 100%;
      background: linear-gradient(90deg, #38bdf8, #10b981);
      transition: width 0.05s linear;
    }
    .optical-beacon {
      width: 64px;
      height: 64px;
      border-radius: 12px;
      background: #0f172a;
      border: 2px solid #334155;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 0.68rem;
      font-weight: 700;
      color: #64748b;
      text-align: center;
      transition: all 0.04s ease;
    }
    .optical-beacon.active {
      background: #ffffff;
      color: #020617;
      border-color: #38bdf8;
      box-shadow: 0 0 38px #ffffff, 0 0 70px #38bdf8;
    }

    /* Navigation Tabs */
    .tabs-nav {
      display: flex;
      gap: 8px;
      margin-top: 16px;
      border-bottom: 1px solid var(--border);
      padding-bottom: 10px;
      flex-wrap: wrap;
    }
    .tab-btn {
      background: var(--bg-1);
      color: var(--muted);
      border: 1px solid var(--border);
      padding: 9px 16px;
      border-radius: 9px;
      cursor: pointer;
      font-weight: 600;
      font-size: 0.88rem;
      transition: all 0.15s ease;
    }
    .tab-btn:hover {
      color: var(--text);
      background: var(--bg-2);
    }
    .tab-btn.active {
      background: rgba(56,189,248,0.16);
      color: var(--cyan);
      border-color: var(--cyan);
      box-shadow: 0 0 16px rgba(56,189,248,0.2);
    }

    /* Grid & Cards */
    .tab-pane { display: none; margin-top: 16px; }
    .tab-pane.active { display: block; }
    .grid-2 {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(460px, 1fr));
      gap: 16px;
    }
    .grid-3 {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(330px, 1fr));
      gap: 16px;
    }
    .card {
      background: var(--bg-1);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 18px;
    }
    .card h2 {
      margin: 0 0 12px 0;
      font-size: 1.02rem;
      color: var(--cyan);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .card h3 {
      margin: 14px 0 8px 0;
      font-size: 0.9rem;
      color: #cbd5e1;
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
      font-size: 0.8rem;
      color: var(--muted);
    }
    input, select {
      background: var(--bg-0);
      color: var(--text);
      border: 1px solid var(--border);
      border-radius: 7px;
      padding: 8px 10px;
      font-size: 0.86rem;
      font-family: inherit;
    }
    input:focus, select:focus {
      outline: none;
      border-color: var(--cyan);
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
      font-size: 0.82rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.14s ease;
    }
    button.btn:hover {
      background: #283a5e;
      border-color: var(--cyan);
    }
    button.btn.primary {
      background: linear-gradient(135deg, #0284c7, #2563eb);
      border-color: #38bdf8;
      color: #fff;
    }
    button.btn.primary:hover {
      filter: brightness(1.12);
    }
    button.btn.emerald {
      background: rgba(16,185,129,0.2);
      border-color: #10b981;
      color: #34d399;
    }
    button.btn.amber {
      background: rgba(245,158,11,0.2);
      border-color: #f59e0b;
      color: #fbbf24;
    }
    button.btn.rose {
      background: rgba(244,63,94,0.2);
      border-color: #f43f5e;
      color: #fb7185;
    }

    /* Telemetry KPI boxes */
    .kpi-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(145px, 1fr));
      gap: 10px;
      margin-bottom: 12px;
    }
    .kpi-box {
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 9px;
      padding: 10px 12px;
    }
    .kpi-box .kpi-label {
      font-size: 0.73rem;
      color: var(--muted);
    }
    .kpi-box .kpi-val {
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 1.12rem;
      font-weight: 700;
      margin-top: 3px;
      color: var(--text);
    }

    /* Code & Hex Blocks */
    pre.code-box {
      background: #050811;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 11px 13px;
      font-family: "JetBrains Mono", "Consolas", monospace;
      font-size: 0.78rem;
      color: #a5f3fc;
      overflow-x: auto;
      max-height: 250px;
      margin: 6px 0 0 0;
      line-height: 1.45;
    }
    table.data-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.8rem;
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
    .help-article p, .help-article li {
      color: #cbd5e1;
      line-height: 1.6;
      font-size: 0.9rem;
    }
    .step-badge {
      display: inline-flex;
      width: 24px;
      height: 24px;
      border-radius: 50%;
      background: var(--cyan);
      color: #020617;
      font-weight: 800;
      font-size: 0.8rem;
      align-items: center;
      justify-content: center;
      margin-right: 8px;
    }
  </style>
</head>
<body>
  <div class="app-shell">
    <!-- Top Header -->
    <header class="top-bar">
      <div class="brand-title">
        <div>
          <h1>ABU CLOCK</h1>
          <div class="subtitle">
            Unified PC Master Clock, Timecode &amp; Multi-Sensor Synchronization Studio &bull;
            <span id="hdr-version">v0.0.0.12</span> &bull;
            Backend: <strong id="hdr-backend">QueryPerformanceCounter</strong>
          </div>
        </div>
      </div>
      <div class="badge-row">
        <span id="badge-lock" class="badge Locked">LOCKED</span>
        <span id="badge-port" class="badge info">PORT: LISTENING</span>
        <span id="badge-transport" class="badge info">WIRED LAN</span>
        <span id="badge-wakelock" class="badge Locked">WAKE-LOCK ON</span>
      </div>
    </header>

    <!-- Master SMPTE ST 12-1 Digital Slate -->
    <section class="digital-slate" aria-label="Master SMPTE ST 12-1 Digital Slate">
      <div>
        <div id="slate-tc" class="tc-main">00:00:00;00.00</div>
        <div class="tc-meta">
          <span>Frame: <strong id="slate-frame">#0</strong></span>
          <span>BCD (32-bit): <strong id="slate-bcd">0x00000000</strong></span>
          <span>Rate: <strong id="slate-rate">29.970 Hz (DF)</strong></span>
          <span>Domain: <strong id="slate-domain">1</strong></span>
          <span>TAI (ns): <strong id="slate-tai">1729180800000000000</strong></span>
        </div>
      </div>
      <div class="slate-right">
        <div class="phase-box">
          <div style="display:flex;justify-content:space-between;font-size:0.75rem;color:var(--muted);">
            <span>Sub-Frame Phase (&phi;)</span>
            <strong id="slate-phase-text">0.000</strong>
          </div>
          <div class="phase-bar-bg">
            <div id="slate-phase-bar" class="phase-bar-fill" style="width:0%"></div>
          </div>
          <div style="font-size:0.72rem;color:var(--muted);margin-top:6px;">
            User Trim: <strong id="slate-trim">0 ns</strong>
          </div>
        </div>
        <div id="optical-beacon" class="optical-beacon" title="Fires full-white optical flash on Frame 00 for high-speed camera validation">
          FRAME 00<br/>FLASH
        </div>
      </div>
    </section>

    <!-- Navigation Tabs -->
    <nav class="tabs-nav">
      <button class="tab-btn active" onclick="switchTab('tab-dashboard', this)">
        1. Clock, Settings &amp; Kalman PLL
      </button>
      <button class="tab-btn" onclick="switchTab('tab-ptp', this)">
        2. PTP Engine &amp; ST 2059-2 TLV
      </button>
      <button class="tab-btn" onclick="switchTab('tab-trigger', this)">
        3. Barrier Trigger, Telemetry &amp; Manifests
      </button>
      <button class="tab-btn" onclick="switchTab('tab-appliance', this)">
        4. Grandmaster Hardware Appliance
      </button>
      <button class="tab-btn" onclick="switchTab('tab-help', this)">
        5. Help, Fundamentals &amp; Graphical Guide
      </button>
    </nav>

    <!-- TAB 1: DASHBOARD, SETTINGS & KALMAN PLL -->
    <div id="tab-dashboard" class="tab-pane active">
      <div class="grid-2">
        <!-- Parameter Settings Card -->
        <section class="card">
          <h2>
            <span>Master / Follower &amp; Timing Parameter Settings</span>
            <span class="badge info">Live Rust Engine</span>
          </h2>
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
              PTP Domain (0–127, Default 1)
              <input id="cfg-domain" type="number" min="0" max="127" value="1" />
            </label>
            <label>
              BMCA Priority 1 (0–255)
              <input id="cfg-prio1" type="number" min="0" max="255" value="128" />
            </label>
            <label>
              BMCA Priority 2 (0–255)
              <input id="cfg-prio2" type="number" min="0" max="255" value="128" />
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
              Network Medium (Kalman Variance)
              <select id="cfg-wireless">
                <option value="false">Wired Ethernet (&lt; 100 ns Lock Gate)</option>
                <option value="true">Wireless Wi-Fi (&lt; 500 &mu;s EMA Lock Gate)</option>
              </select>
            </label>
            <label>
              Display Wake-Lock
              <select id="cfg-wakelock">
                <option value="true">Keep Screen Awake (Active)</option>
                <option value="false">Allow OS Sleep</option>
              </select>
            </label>
          </div>
          <div class="btn-row">
            <button class="btn primary" onclick="applyConfig()">Apply Settings to Rust Engine</button>
          </div>
        </section>

        <!-- Kalman PLL & Oscilloscope Card -->
        <section class="card">
          <h2>
            <span>2-State Kalman PLL &amp; 7.0s ST 2059 Warm-Up Gate</span>
            <span id="kalman-gate-label" class="badge Locked">WARM-UP: 100%</span>
          </h2>
          <div class="kpi-grid">
            <div class="kpi-box">
              <div class="kpi-label">Phase Offset (&theta;)</div>
              <div id="kpi-offset" class="kpi-val">0 ns</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">Crystal Drift (&alpha;)</div>
              <div id="kpi-drift" class="kpi-val">0.00 ppb</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">One-Way Delay (d)</div>
              <div id="kpi-delay" class="kpi-val">380 ns</div>
            </div>
            <div class="kpi-box">
              <div class="kpi-label">Kalman Variance (P00)</div>
              <div id="kpi-var" class="kpi-val">0.0 ns&sup2;</div>
            </div>
          </div>

          <!-- Live SVG Oscilloscope -->
          <div style="background:#050811;border:1px solid var(--border);border-radius:9px;padding:10px;">
            <div style="display:flex;justify-content:space-between;font-size:0.75rem;color:var(--muted);margin-bottom:4px;">
              <span>Live Phase Offset History (&theta; in nanoseconds, last 40 servo samples)</span>
              <span id="osc-latest-text">Latest: 0 ns</span>
            </div>
            <svg id="osc-svg" viewBox="0 0 600 130" style="width:100%;height:130px;display:block;">
              <line x1="0" y1="65" x2="600" y2="65" stroke="#1e293b" stroke-dasharray="4 4" />
              <polyline id="osc-line" fill="none" stroke="#38bdf8" stroke-width="2.5" points="0,65 600,65" />
            </svg>
          </div>

          <h3>Interactive Clock Discipline &amp; Fault Injection Test Bench</h3>
          <div class="btn-row">
            <button class="btn emerald" onclick="clockAction('tick_clean')">
              + Inject Clean Wired PTP Tick (&plusmn;9 ns)
            </button>
            <button class="btn amber" onclick="clockAction('tick_wifi')">
              + Inject Wi-Fi Delay Spike (+1,850 ns)
            </button>
            <button class="btn amber" onclick="clockAction('step_50us')">
              + Inject +50 &mu;s Phase Disturbance
            </button>
            <button class="btn primary" onclick="clockAction('fast_forward_7s')">
              &raquo; Fast-Forward 7.0s ST 2059 Warm-Up (Converge &amp; Lock)
            </button>
            <button class="btn" onclick="clockAction('holdover')">
              Simulate Holdover
            </button>
            <button class="btn rose" onclick="clockAction('unlock')">
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
            <span>IEEE 1588-2019 Best Master Clock Algorithm (BMCA) Simulator</span>
            <span id="bmca-port-badge" class="badge info">LISTENING</span>
          </h2>
          <p style="font-size:0.83rem;color:var(--muted);margin-top:0;">
            Inject a simulated remote <code>Announce</code> frame carrying a SMPTE ST 2059-2 TLV.
            If the remote node has a lower (better) <code>Priority 1</code> or <code>Clock Class</code> than your local node,
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
              Inject Remote Announce &amp; Run BMCA Election
            </button>
          </div>
          <h3>BMCA Election Audit Log</h3>
          <pre id="bmca-log-box" class="code-box"></pre>
        </section>

        <section class="card">
          <h2>
            <span>Live SMPTE ST 2059-2 Synchronization Metadata TLV Inspector</span>
            <span class="badge Locked">20 Bytes Big-Endian</span>
          </h2>
          <p style="font-size:0.83rem;color:var(--muted);margin-top:0;">
            Real-time hexadecimal dump of <code>SmpteSyncMetadataTlv::to_bytes()</code> appended to outbound PTP <code>Announce</code> packets:
          </p>
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
              <tr>
                <td>Bytes 0..1</td>
                <td>tlvType</td>
                <td id="tlv-type">0x0003 (Organization Extension)</td>
              </tr>
              <tr>
                <td>Bytes 2..3</td>
                <td>lengthField</td>
                <td>0x0010 (16 bytes payload)</td>
              </tr>
              <tr>
                <td>Bytes 4..6</td>
                <td>organizationId</td>
                <td id="tlv-org">00-0E-9B (SMPTE OUI)</td>
              </tr>
              <tr>
                <td>Bytes 7..9</td>
                <td>organizationSubType</td>
                <td id="tlv-sub">00-00-01 (ST 2059-2 SM)</td>
              </tr>
              <tr>
                <td>Bytes 10..11</td>
                <td>currentUtcOffset</td>
                <td id="tlv-utc">+37 s (TAI - UTC)</td>
              </tr>
              <tr>
                <td>Bytes 12..15</td>
                <td>defaultSystemFrameRate (Num)</td>
                <td id="tlv-num">30000</td>
              </tr>
              <tr>
                <td>Bytes 16..18</td>
                <td>defaultSystemFrameRate (Den 24-bit)</td>
                <td id="tlv-den">1001</td>
              </tr>
              <tr>
                <td>Byte 19</td>
                <td>timecodeFlags</td>
                <td id="tlv-flags">0x01 (Drop-Frame)</td>
              </tr>
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
            <span>Multi-Sensor Barrier Trigger (<code>SCHEDULE_CAPTURE_AT</code>)</span>
            <span class="badge Locked">Sub-50&mu;s Spinlock</span>
          </h2>
          <p style="font-size:0.82rem;color:var(--muted);margin-top:0;">
            Instead of sending an immediate "capture now" packet that suffers from variable Wi-Fi/LAN transit delay,
            <code>AbuScanCoordinator</code> broadcasts a future target TAI nanosecond ($T_{\text{now}} + \Delta T_{\text{lead}}$).
            Every rig counts down locally against its disciplined <code>OverlayClock</code> and fires at the exact same physical instant.
          </p>
          <div class="btn-row">
            <button class="btn primary" onclick="fireTrigger('single', 1)">
              &#9889; Fire SCHEDULE_CAPTURE_AT (+Lead Time)
            </button>
            <button class="btn emerald" onclick="fireTrigger('burst', 5)">
              &#128248; Schedule 5-Frame Synchronized Burst Scan
            </button>
          </div>
          <h3>Latest Multi-Rig Barrier Receipts</h3>
          <div id="trigger-summary" style="font-size:0.82rem;color:var(--cyan);margin-bottom:6px;"></div>
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
            <span>6-DoF Camera Tracking Telemetry (OpenTrackIO &amp; FreeD)</span>
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
              Synthesize OpenTrackIO &amp; FreeD Packets
            </button>
          </div>
          <h3>Standard 29-Byte FreeD (<code>0xD1</code>) Wire Hex + Modulo-256 Checksum</h3>
          <pre id="freed-d1-box" class="code-box"></pre>
          <h3>Extended 41-Byte AbuTrack FreeD-TC (<code>0xD2</code>) with Embedded BCD Timecode &amp; TAI</h3>
          <pre id="freed-d2-box" class="code-box"></pre>
          <h3>SMPTE RIS OpenTrackIO (<code>OTrk</code>) JSON Payload</h3>
          <pre id="otrk-json-box" class="code-box"></pre>
        </section>
      </div>

      <!-- Session Manifest Exporter -->
      <section class="card" style="margin-top:16px;">
        <h2>
          <span>Synchronized Session Manifest Exporter (<code>.abuscan.json</code> &amp; Colmap <code>images.txt</code>)</span>
          <span class="badge info">RealityCapture / 3DGS / Postshot Ready</span>
        </h2>
        <div class="btn-row" style="margin-top:0;margin-bottom:10px;">
          <input id="manifest-sensor" type="text" value="SonyA1_Rig03" style="width:180px;" />
          <input id="manifest-ext" type="text" value="arw" style="width:80px;" />
          <button class="btn emerald" onclick="addManifestRecord()">
            + Stamp &amp; Append Sensor Capture to Manifest
          </button>
        </div>
        <div class="grid-2">
          <div>
            <h3><code>.abuscan.json</code> Session Manifest</h3>
            <pre id="manifest-json-box" class="code-box"></pre>
          </div>
          <div>
            <h3>Colmap / Jawset Postshot <code>images.txt</code></h3>
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
            <span>Phase 3 Hardware Grandmaster Appliance Monitor</span>
            <span id="app-lock-badge" class="badge Locked">GNSS / TCXO LOCKED (CLASS 6)</span>
          </h2>
          <p style="font-size:0.84rem;color:var(--muted);margin-top:0;">
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
              Lock Primary Reference (GNSS / DS3231 TCXO &rarr; Class 6)
            </button>
            <button class="btn amber" onclick="setApplianceLock(false)">
              Simulate Antenna Loss / Holdover Mode (TCXO Holdover &rarr; Class 7)
            </button>
          </div>
        </section>

        <section class="card">
          <h2>Hardware PHC Devices &amp; Simultaneous Multicast Profiles</h2>
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

    <!-- TAB 5: HELP, FUNDAMENTALS & GRAPHICAL GUIDE -->
    <div id="tab-help" class="tab-pane help-article">
      <section class="card" style="margin-bottom:16px;">
        <h2>Visual Architecture &amp; End-to-End Signal Flow Diagram</h2>
        <p>
          The diagram below illustrates how <strong>Abu Clock</strong> locks hardware monotonic counters across PC, Mobile, LiDAR, Unreal Engine 5, and NVIDIA Omniverse to a single International Atomic Time (TAI) timeline:
        </p>
        <div style="background:#050811;border:1px solid var(--border);border-radius:10px;padding:14px;overflow-x:auto;">
          <svg viewBox="0 0 1100 320" style="width:100%;min-width:780px;height:auto;display:block;">
            <!-- Tier 1 Box -->
            <rect x="20" y="30" width="240" height="260" rx="12" fill="#0d1527" stroke="#38bdf8" stroke-width="2"/>
            <text x="40" y="60" fill="#38bdf8" font-size="15" font-weight="bold">TIER 1: CORE CLOCK ENGINE</text>
            <rect x="38" y="80" width="204" height="52" rx="8" fill="#13203a" stroke="#263554"/>
            <text x="50" y="102" fill="#f1f5f9" font-size="12" font-weight="bold">PlatformMonotonicClock</text>
            <text x="50" y="120" fill="#94a3b8" font-size="11">Win QPC / Mach / MONOTONIC_RAW</text>

            <rect x="38" y="148" width="204" height="56" rx="8" fill="#13203a" stroke="#10b981"/>
            <text x="50" y="170" fill="#34d399" font-size="12" font-weight="bold">OverlayClock (Kalman PLL)</text>
            <text x="50" y="188" fill="#94a3b8" font-size="11">State: [offset_ns &theta;, drift_ppb &alpha;]</text>

            <rect x="38" y="218" width="204" height="54" rx="8" fill="#13203a" stroke="#38bdf8"/>
            <text x="50" y="240" fill="#38bdf8" font-size="12" font-weight="bold">SmpteTimecodeEngine</text>
            <text x="50" y="258" fill="#94a3b8" font-size="11">ST 12-1 Drop-Frame + 32-bit BCD</text>

            <!-- Arrow 1 -> 2 -->
            <line x1="260" y1="160" x2="320" y2="160" stroke="#38bdf8" stroke-width="2.5"/>
            <polygon points="320,160 310,154 310,166" fill="#38bdf8"/>

            <!-- Tier 2 Box -->
            <rect x="320" y="30" width="290" height="260" rx="12" fill="#0d1527" stroke="#a855f7" stroke-width="2"/>
            <text x="340" y="60" fill="#c084fc" font-size="15" font-weight="bold">TIER 2: NETWORK &amp; TELEMETRY BUS</text>
            <rect x="338" y="80" width="254" height="52" rx="8" fill="#13203a" stroke="#263554"/>
            <text x="350" y="102" fill="#f1f5f9" font-size="12" font-weight="bold">DualModePtpEngine + ST 2059-2 TLV</text>
            <text x="350" y="120" fill="#94a3b8" font-size="11">PTP (319/320) &bull; gPTP &bull; AbuSync (50319)</text>

            <rect x="338" y="148" width="254" height="56" rx="8" fill="#13203a" stroke="#f59e0b"/>
            <text x="350" y="170" fill="#fbbf24" font-size="12" font-weight="bold">AbuScanCoordinator (+50ms Barrier)</text>
            <text x="350" y="188" fill="#94a3b8" font-size="11">SCHEDULE_CAPTURE_AT + 500&mu;s Spinlock</text>

            <rect x="338" y="218" width="254" height="54" rx="8" fill="#13203a" stroke="#263554"/>
            <text x="350" y="240" fill="#f1f5f9" font-size="12" font-weight="bold">OpenTrackIO (OTrk) &amp; FreeD (0xD1/D2)</text>
            <text x="350" y="258" fill="#94a3b8" font-size="11">6-DoF Camera Pose + Exact Frame TAI</text>

            <!-- Arrow 2 -> 3 -->
            <line x1="610" y1="160" x2="670" y2="160" stroke="#a855f7" stroke-width="2.5"/>
            <polygon points="670,160 660,154 660,166" fill="#a855f7"/>

            <!-- Tier 3 Box -->
            <rect x="670" y="30" width="410" height="260" rx="12" fill="#0d1527" stroke="#10b981" stroke-width="2"/>
            <text x="690" y="60" fill="#34d399" font-size="15" font-weight="bold">TIER 3: UNIFIED APP, ENGINES &amp; APPLIANCE</text>
            <rect x="690" y="80" width="178" height="85" rx="8" fill="#13203a" stroke="#38bdf8"/>
            <text x="702" y="104" fill="#38bdf8" font-size="12" font-weight="bold">Abu Clock PC Studio</text>
            <text x="702" y="124" fill="#94a3b8" font-size="11">&bull; 120Hz Digital Slate</text>
            <text x="702" y="142" fill="#94a3b8" font-size="11">&bull; Live Control Center</text>

            <rect x="882" y="80" width="178" height="85" rx="8" fill="#13203a" stroke="#10b981"/>
            <text x="894" y="104" fill="#34d399" font-size="12" font-weight="bold">UE5 &amp; Omniverse</text>
            <text x="894" y="124" fill="#94a3b8" font-size="11">&bull; UAbuClockTimecodeProvider</text>
            <text x="894" y="142" fill="#94a3b8" font-size="11">&bull; omni.abu.clock C-ABI</text>

            <rect x="690" y="180" width="178" height="90" rx="8" fill="#13203a" stroke="#f59e0b"/>
            <text x="702" y="204" fill="#fbbf24" font-size="12" font-weight="bold">Abu-Track-Scan Rigs</text>
            <text x="702" y="224" fill="#94a3b8" font-size="11">&bull; iOS Swift AbuClockKit</text>
            <text x="702" y="242" fill="#94a3b8" font-size="11">&bull; Win C++ &amp; RoboSense LiDAR</text>

            <rect x="882" y="180" width="178" height="90" rx="8" fill="#13203a" stroke="#a855f7"/>
            <text x="894" y="204" fill="#c084fc" font-size="12" font-weight="bold">Pi CM4 Grandmaster</text>
            <text x="894" y="224" fill="#94a3b8" font-size="11">&bull; DS3231 TCXO (&plusmn;2ppm)</text>
            <text x="894" y="242" fill="#94a3b8" font-size="11">&bull; /dev/ptp0 &amp; /dev/ptp1</text>
          </svg>
        </div>
      </section>

      <div class="grid-2">
        <section class="card">
          <h2>Core Fundamentals: How Abu Clock Works</h2>
          <p>
            <strong>1. Why Standard OS Clocks Fail:</strong> Standard operating system wall clocks (<code>SystemTime</code> / <code>gettimeofday</code>) jump forward or backward whenever background NTP daemons step the clock, ruining multi-camera shutter alignment and 3D Gaussian Splatting (3DGS) reconstruction.
            <strong>Abu Clock</strong> never mutates the OS clock. Instead, it reads the CPU/OS hardware monotonic counter (<code>QueryPerformanceCounter</code> on Windows, <code>mach_continuous_time</code> on iOS/macOS, <code>CLOCK_MONOTONIC_RAW</code> on Linux) and maintains a lock-free user-space <strong>OverlayClock</strong>:
          </p>
          <pre class="code-box">T_TAI(t) = T_anchor + (C_mono(t) - C_mono(t0)) * (1 + alpha_freq) + theta_phase</pre>
          <p>
            <strong>2. 2-State Kalman PLL &amp; 7.0-Second Warm-Up Gate:</strong> Network packets experience queuing jitter. The <code>ClockKalmanFilter</code> estimates both instantaneous phase offset ($\theta_{\text{phase}}$ in ns) and crystal frequency drift ($\alpha_{\text{freq}}$ in ppb). Per <strong>SMPTE ST 2059-2</strong>, the node enforces a mandatory <strong>7.0-second warm-up convergence gate</strong> before transitioning its status badge to <code>LOCKED</code> (&lt; 100 ns wired, &lt; 500 &mu;s wireless).
          </p>
          <p>
            <strong>3. TAI vs. UTC &amp; Rational Drop-Frame SMPTE Math:</strong> Broadcast PTP operates on <strong>International Atomic Time (TAI)</strong>, which is monotonic and has zero leap-second discontinuities (currently <code>TAI = UTC + 37 seconds</code>). For NTSC fractional rates (<code>29.97 DF = 30000/1001</code>), frame numbers <code>00</code> and <code>01</code> are skipped at the start of every minute except every 10th minute (<code>17,982</code> frames per 10-minute block) using exact 128-bit integer arithmetic with zero floating-point drift.
          </p>
          <p>
            <strong>4. Future-Time Barrier Triggering (<code>SCHEDULE_CAPTURE_AT</code>):</strong> Wi-Fi and LAN packets take variable time (`0.3 ms` to `15 ms`) to arrive. Abu Clock eliminates network trigger jitter by broadcasting a future target timestamp ($T_{\text{exec}} = T_{\text{now}} + 50\text{ ms}$). Every device sleeps until $T_{\text{exec}} - 500\ \mu\text{s}$ and then enters a tight hardware register spinlock to fire the shutter at the exact nanosecond.
          </p>
        </section>

        <section class="card">
          <h2>Graphical Step-by-Step Guide: How to Use This App</h2>
          <div style="background:#050811;border:1px solid var(--border);border-radius:10px;padding:12px;margin-bottom:12px;">
            <svg viewBox="0 0 560 210" style="width:100%;height:auto;display:block;">
              <!-- Visual Callout Map of the UI -->
              <rect x="10" y="10" width="540" height="45" rx="7" fill="#0f172a" stroke="#38bdf8"/>
              <circle cx="30" cy="32" r="11" fill="#38bdf8"/><text x="26" y="36" fill="#020617" font-size="12" font-weight="bold">1</text>
              <text x="50" y="36" fill="#f1f5f9" font-size="12" font-weight="bold">Master Digital Slate Banner (Top): Live SMPTE TC, BCD Hex, Phase &amp; Frame-00 Flash</text>

              <rect x="10" y="65" width="262" height="62" rx="7" fill="#0f172a" stroke="#10b981"/>
              <circle cx="30" cy="96" r="11" fill="#10b981"/><text x="26" y="100" fill="#020617" font-size="12" font-weight="bold">2</text>
              <text x="48" y="90" fill="#34d399" font-size="11" font-weight="bold">Tab 1 Left: Parameter Settings</text>
              <text x="48" y="108" fill="#94a3b8" font-size="10">Set Role, Profile, FPS, Domain &amp; Apply</text>

              <rect x="288" y="65" width="262" height="62" rx="7" fill="#0f172a" stroke="#f59e0b"/>
              <circle cx="308" cy="96" r="11" fill="#f59e0b"/><text x="304" y="100" fill="#020617" font-size="12" font-weight="bold">3</text>
              <text x="326" y="90" fill="#fbbf24" font-size="11" font-weight="bold">Tab 1 Right: Kalman PLL Test Bench</text>
              <text x="326" y="108" fill="#94a3b8" font-size="10">Inject Jitter, Fast-Forward 7s &amp; View Scope</text>

              <rect x="10" y="138" width="172" height="62" rx="7" fill="#0f172a" stroke="#a855f7"/>
              <circle cx="28" cy="168" r="11" fill="#a855f7"/><text x="24" y="172" fill="#020617" font-size="12" font-weight="bold">4</text>
              <text x="44" y="164" fill="#c084fc" font-size="11" font-weight="bold">Tab 2: PTP &amp; TLV</text>
              <text x="44" y="180" fill="#94a3b8" font-size="10">BMCA &amp; 20B Hex Dump</text>

              <rect x="194" y="138" width="172" height="62" rx="7" fill="#0f172a" stroke="#38bdf8"/>
              <circle cx="212" cy="168" r="11" fill="#38bdf8"/><text x="208" y="172" fill="#020617" font-size="12" font-weight="bold">5</text>
              <text x="228" y="164" fill="#38bdf8" font-size="11" font-weight="bold">Tab 3: Trigger &amp; Pose</text>
              <text x="228" y="180" fill="#94a3b8" font-size="10">+50ms Barrier &amp; FreeD</text>

              <rect x="378" y="138" width="172" height="62" rx="7" fill="#0f172a" stroke="#10b981"/>
              <circle cx="396" cy="168" r="11" fill="#10b981"/><text x="392" y="172" fill="#020617" font-size="12" font-weight="bold">6</text>
              <text x="412" y="164" fill="#34d399" font-size="11" font-weight="bold">Tab 4: Appliance</text>
              <text x="412" y="180" fill="#94a3b8" font-size="10">Pi CM4 + TCXO Class 6/7</text>
            </svg>
          </div>
          <ol style="padding-left:18px;margin:0;">
            <li>
              <span class="step-badge">1</span><strong>Monitor the Master Digital Slate (Top Ribbon):</strong> Watch the real-time SMPTE timecode, 32-bit BCD word, sub-frame phase bar, and Frame-00 optical flash beacon driven directly by the Rust <code>SmpteTimecodeEngine</code>.
            </li>
            <li>
              <span class="step-badge">2</span><strong>Configure Timing Parameters (Tab 1 Left):</strong> Choose your Operating Role (<code>AutoBmca</code>, <code>Master</code>, <code>Follower</code>), Synchronization Profile, and Frame Rate (e.g. <code>29.97 DF</code>, <code>24 fps</code>, <code>60 fps</code>, or <code>10 Hz LiDAR</code>), then click <strong>Apply Settings to Rust Engine</strong>.
            </li>
            <li>
              <span class="step-badge">3</span><strong>Test Kalman Clock Steering &amp; Fault Recovery (Tab 1 Right):</strong> Click <strong>Inject +50 &mu;s Phase Disturbance</strong> or <strong>Simulate Unlock Fault</strong> to perturb the clock, then click <strong>Fast-Forward 7.0s ST 2059 Warm-Up</strong> to watch the Kalman oscilloscope converge back to <code>LOCKED (&lt;100 ns)</code>.
            </li>
            <li>
              <span class="step-badge">4</span><strong>Inspect PTP &amp; Run BMCA Elections (Tab 2):</strong> Inspect the live 20-byte big-endian <code>SMPTETLV</code> wire bytes and inject a remote Grandmaster <code>Announce</code> packet to verify automatic follower failover and frame-rate adoption.
            </li>
            <li>
              <span class="step-badge">5</span><strong>Fire Multi-Sensor Captures &amp; Camera Telemetry (Tab 3):</strong> Click <strong>Fire SCHEDULE_CAPTURE_AT</strong> to verify sub-50&mu;s multi-rig receipts, adjust 6-DoF camera pose sliders to inspect live <strong>FreeD (0xD1/0xD2)</strong> and <strong>OpenTrackIO</strong> packets, and export <code>.abuscan.json</code> / Colmap <code>images.txt</code> manifests.
            </li>
            <li>
              <span class="step-badge">6</span><strong>Test Grandmaster Appliance Holdover (Tab 4):</strong> Toggle between GNSS/TCXO Locked (<code>Class 6</code>) and Holdover (<code>Class 7</code>) on the Phase 3 hardware Grandmaster monitor.
            </li>
          </ol>
        </section>
      </div>
    </div>
  </div>

  <script>
    const API_BASE = "http://127.0.0.1:7878";
    let formInitialized = false;

    function switchTab(tabId, btnEl) {
      document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
      document.getElementById(tabId).classList.add('active');
      btnEl.classList.add('active');
    }

    function renderSnapshot(snap) {
      if (!snap || !snap.version) return;
      document.getElementById('hdr-version').textContent = snap.version;
      document.getElementById('hdr-backend').textContent = snap.monotonic_backend;

      // Badges
      const lockBadge = document.getElementById('badge-lock');
      lockBadge.textContent = snap.convergence.lock_state.toUpperCase();
      lockBadge.className = 'badge ' + snap.convergence.lock_state;

      document.getElementById('badge-port').textContent = 'PORT: ' + String(snap.ptp_port_state).toUpperCase();
      document.getElementById('badge-transport').textContent = snap.convergence.is_wireless ? 'WIRELESS Wi-Fi' : 'WIRED LAN';
      document.getElementById('badge-wakelock').textContent = snap.keep_screen_on_active ? 'WAKE-LOCK ON' : 'WAKE-LOCK OFF';

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

      // Initialize form inputs once
      if (!formInitialized) {
        document.getElementById('cfg-role').value = snap.role;
        document.getElementById('cfg-profile').value = snap.profile;
        document.getElementById('cfg-domain').value = snap.ptp_domain;
        document.getElementById('cfg-prio1').value = snap.priority1;
        document.getElementById('cfg-prio2').value = snap.priority2;
        document.getElementById('cfg-offset-ns').value = snap.user_offset_ns;
        document.getElementById('cfg-lead-ms').value = snap.trigger_lead_time_ms;
        document.getElementById('cfg-wireless').value = String(snap.convergence.is_wireless);
        document.getElementById('cfg-wakelock').value = String(snap.keep_screen_on_active);
        formInitialized = true;
      }

      // Kalman KPIs
      document.getElementById('kpi-offset').textContent = snap.convergence.offset_ns + ' ns';
      document.getElementById('kpi-drift').textContent = snap.convergence.drift_ppb.toFixed(2) + ' ppb';
      document.getElementById('kpi-delay').textContent = snap.convergence.path_delay_ns + ' ns';
      document.getElementById('kpi-var').textContent = snap.convergence.offset_variance_ns2.toFixed(1) + ' ns²';

      const gateLabel = document.getElementById('kalman-gate-label');
      gateLabel.textContent = '7.0s WARM-UP: ' + (snap.convergence.warmup_progress * 100).toFixed(0) + '%';
      gateLabel.className = 'badge ' + snap.convergence.lock_state;

      // Oscilloscope polyline
      const hist = snap.offset_history_ns || [];
      if (hist.length > 0) {
        document.getElementById('osc-latest-text').textContent = 'Latest: ' + hist[hist.length - 1] + ' ns';
        const maxAbs = Math.max(100, ...hist.map(v => Math.abs(v)));
        const pts = hist.map((val, idx) => {
          const x = (idx / Math.max(1, hist.length - 1)) * 600;
          const y = 65 - (val / maxAbs) * 52;
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
            <td style="color:${r.within_subframe_budget ? '#34d399' : '#fb7185'}">${r.jitter_ns >= 0 ? '+' : ''}${r.jitter_ns} ns</td>
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

    async function postApi(path, payload) {
      try {
        const res = await fetch(API_BASE + path, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        const snap = await res.json();
        renderSnapshot(snap);
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
        keep_screen_on_active: document.getElementById('cfg-wakelock').value === 'true'
      });
    }

    function clockAction(action) {
      postApi('/api/clock', { action });
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
      });
    }

    function fireTrigger(mode, frames) {
      postApi('/api/trigger', { mode, frames });
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
      });
    }

    function addManifestRecord() {
      postApi('/api/manifest', {
        sensor_id: document.getElementById('manifest-sensor').value,
        ext: document.getElementById('manifest-ext').value
      });
    }

    function setApplianceLock(gnss_locked) {
      postApi('/api/appliance', { gnss_locked });
    }

    pollState();
    setInterval(pollState, 50);
  </script>
</body>
</html>
"##;
