import React, { useEffect, useState } from "react";
import { FRAME_RATE_PRESETS, FrameRatePreset, GuiSlateSnapshot, GuiTimingProfile, OperatingRole } from "./types";

const API_BASE = "http://127.0.0.1:7878";

const INITIAL_SNAPSHOT: GuiSlateSnapshot = {
  version: "v0.0.0.13",
  monotonic_backend: "QueryPerformanceCounter (Windows QPC)",
  role: "AutoBmca",
  profile: "AbuSyncCsptp",
  ptp_domain: 1,
  priority1: 128,
  priority2: 128,
  slate_timecode: "00:00:00;00.00",
  bcd_hex: "0x00000000",
  frame_index: 0,
  phase: 0.0,
  flash_marker_active: false,
  tai_ns: 1729180800000000000,
  frame_rate_hz: 29.97002997,
  drop_frame: true,
  user_offset_ns: 0,
  keep_screen_on_active: true,
  battery_optimization_unrestricted: true,
  convergence: {
    offset_ns: 6,
    drift_ppb: 14.8,
    path_delay_ns: 380,
    offset_variance_ns2: 24.0,
    elapsed_lock_ns: 8_000_000_000,
    warmup_progress: 1.0,
    is_wireless: false,
    lock_state: "Locked",
  },
};

export const App: React.FC = () => {
  const [snapshot, setSnapshot] = useState<GuiSlateSnapshot>(INITIAL_SNAPSHOT);
  const [selectedPreset, setSelectedPreset] = useState<FrameRatePreset>(FRAME_RATE_PRESETS[3]);
  const [liveRustConnected, setLiveRustConnected] = useState<boolean>(false);
  const [activeTab, setActiveTab] = useState<"dashboard" | "ptp" | "trigger" | "appliance" | "help">("dashboard");

  useEffect(() => {
    document.title = `Abu Clock ${snapshot.version} — Precision Master Clock, Timecode & Sync Studio`;
  }, [snapshot.version]);

  useEffect(() => {
    const timer = window.setInterval(async () => {
      try {
        const res = await fetch(`${API_BASE}/api/state`);
        if (res.ok) {
          const data = await res.json();
          setSnapshot(data);
          setLiveRustConnected(true);
          return;
        }
      } catch {
        setLiveRustConnected(false);
      }
      // Fallback local tick when Vite is run without cargo run --bin abu-clock
      setSnapshot((prev) => {
        const nextFrame = prev.frame_index + 1;
        const fps = Math.max(1, Math.round(selectedPreset.numerator / selectedPreset.denominator));
        const f = nextFrame % fps;
        const totalSec = Math.floor(nextFrame / fps);
        const s = totalSec % 60;
        const m = Math.floor(totalSec / 60) % 60;
        const h = Math.floor(totalSec / 3600) % 24;
        const sep = selectedPreset.drop_frame ? ";" : ":";
        const pad = (n: number) => n.toString().padStart(2, "0");
        const tc = `${pad(h)}:${pad(m)}:${pad(s)}${sep}${pad(f)}.00`;
        return {
          ...prev,
          frame_index: nextFrame,
          slate_timecode: tc,
          flash_marker_active: f === 0,
          frame_rate_hz: selectedPreset.numerator / selectedPreset.denominator,
          drop_frame: selectedPreset.drop_frame,
        };
      });
    }, 50);
    return () => window.clearInterval(timer);
  }, [selectedPreset]);

  const postAction = async (path: string, payload: unknown) => {
    try {
      const res = await fetch(`${API_BASE}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (res.ok) {
        setSnapshot(await res.json());
      }
    } catch {}
  };

  return (
    <div className="slate-container">
      <header className="top-header">
        <div>
          <h1>ABU CLOCK</h1>
          <small>
            {snapshot.version} &bull; {snapshot.monotonic_backend} &bull; Domain {snapshot.ptp_domain} &bull;{" "}
            {liveRustConnected ? "LIVE RUST ENGINE CONNECTED" : "BROWSER SIMULATOR (Run `cargo run --bin abu-clock` for Live Engine)"}
          </small>
        </div>
        <div style={{ display: "flex", gap: "10px", alignItems: "center" }}>
          <span className={`badge ${snapshot.convergence.lock_state}`}>
            {snapshot.convergence.lock_state.toUpperCase()}
          </span>
          <span className="badge Locked">
            {snapshot.keep_screen_on_active ? "WAKE-LOCK ACTIVE" : "WAKE-LOCK OFF"}
          </span>
        </div>
      </header>

      <section className="digital-slate" aria-label="SMPTE ST 12-1 Digital Timecode Slate">
        <div>
          <div className="timecode-readout">{snapshot.slate_timecode}</div>
          <div style={{ color: "#94a3b8", marginTop: "8px" }}>
            Frame #{snapshot.frame_index} &bull; BCD {snapshot.bcd_hex} &bull;{" "}
            {snapshot.frame_rate_hz.toFixed(3)} Hz {snapshot.drop_frame ? "(Drop-Frame)" : "(Non-Drop-Frame)"} &bull; TAI {snapshot.tai_ns}
          </div>
        </div>
        <div
          title="Optical Sync Flash Marker (Fires on Frame 00)"
          className={`optical-flash-beacon ${snapshot.flash_marker_active ? "active" : ""}`}
        />
      </section>

      <div style={{ display: "flex", gap: "8px", marginBottom: "16px", flexWrap: "wrap" }}>
        <button onClick={() => setActiveTab("dashboard")}>1. Clock, Settings & Kalman PLL</button>
        <button onClick={() => setActiveTab("ptp")}>2. PTP Engine & ST 2059-2 TLV</button>
        <button onClick={() => setActiveTab("trigger")}>3. Barrier Trigger & Telemetry</button>
        <button onClick={() => setActiveTab("appliance")}>4. Grandmaster Appliance</button>
        <button onClick={() => setActiveTab("help")}>5. Help & Visual Guide</button>
      </div>

      {activeTab === "dashboard" && (
        <div className="grid-panels">
          <section className="card">
            <h2>Master / Follower &amp; Profile Configuration</h2>
            <label>
              Operating Mode (BMCA):
              <select
                value={snapshot.role}
                onChange={(e) => setSnapshot({ ...snapshot, role: e.target.value as OperatingRole })}
              >
                <option value="AutoBmca">Auto (IEEE 1588 BMCA Election)</option>
                <option value="Master">Forced Grandmaster</option>
                <option value="Follower">Follower Only</option>
              </select>
            </label>
            <label>
              Synchronization Profile:
              <select
                value={snapshot.profile}
                onChange={(e) => setSnapshot({ ...snapshot, profile: e.target.value as GuiTimingProfile })}
              >
                <option value="AbuSyncCsptp">AbuSync Unprivileged CSPTP (UDP 50319/50320)</option>
                <option value="SmpteSt2059_2">SMPTE ST 2059-2 Broadcast (224.0.1.129:319/320)</option>
                <option value="Ieee802_1AsGptp">IEEE 802.1AS gPTP LiDAR (224.0.0.107)</option>
                <option value="Aes67Harmonized">AES67 + SMPTE ST 2059-2 Harmonized</option>
              </select>
            </label>
            <label>
              Frame Rate / LiDAR Cadence:
              <select
                value={selectedPreset.label}
                onChange={(e) => {
                  const found = FRAME_RATE_PRESETS.find((p) => p.label === e.target.value);
                  if (found) setSelectedPreset(found);
                }}
              >
                {FRAME_RATE_PRESETS.map((p) => (
                  <option key={p.label} value={p.label}>{p.label}</option>
                ))}
              </select>
            </label>
            <label>
              PTP Domain (0–127):
              <input
                type="number"
                min={0}
                max={127}
                value={snapshot.ptp_domain}
                onChange={(e) =>
                  setSnapshot({ ...snapshot, ptp_domain: Math.min(127, Math.max(0, Number(e.target.value))) })
                }
              />
            </label>
            <button
              onClick={() =>
                postAction("/api/config", {
                  role: snapshot.role,
                  profile: snapshot.profile,
                  ptp_domain: snapshot.ptp_domain,
                  priority1: snapshot.priority1,
                  priority2: snapshot.priority2,
                  fps_num: selectedPreset.numerator,
                  fps_den: selectedPreset.denominator,
                  drop_frame: selectedPreset.drop_frame,
                  user_offset_ns: snapshot.user_offset_ns,
                  trigger_lead_time_ms: 50,
                  is_wireless: snapshot.convergence.is_wireless,
                  keep_screen_on_active: snapshot.keep_screen_on_active,
                })
              }
            >
              Apply Settings to Rust Engine
            </button>
          </section>

          <section className="card">
            <h2>Kalman PLL &amp; 7-Second Warm-Up Gate Telemetry</h2>
            <div>Phase Offset (&theta;): <strong>{snapshot.convergence.offset_ns} ns</strong></div>
            <div>Crystal Drift (&alpha;): <strong>{snapshot.convergence.drift_ppb.toFixed(2)} ppb</strong></div>
            <div>One-Way Path Delay: <strong>{snapshot.convergence.path_delay_ns} ns</strong></div>
            <div>7.0s ST 2059 Warm-Up Gate: <strong>{(snapshot.convergence.warmup_progress * 100).toFixed(0)}%</strong></div>
            <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", marginTop: "10px" }}>
              <button onClick={() => postAction("/api/clock", { action: "tick_clean" })}>+ Clean PTP Tick</button>
              <button onClick={() => postAction("/api/clock", { action: "tick_wifi" })}>+ Wi-Fi Spike</button>
              <button onClick={() => postAction("/api/clock", { action: "fast_forward_7s" })}>Fast-Forward 7s Lock</button>
              <button onClick={() => postAction("/api/clock", { action: "holdover" })}>Simulate Holdover</button>
            </div>
          </section>
        </div>
      )}

      {activeTab !== "dashboard" && (
        <section className="card">
          <h2>Full Studio Mode Active on Native Port 7878</h2>
          <p>
            Run <code>cargo run --bin abu-clock</code> or double-click <code>Start-AbuClock.bat</code> in the repository root to open the native standalone <strong>Abu Clock</strong> desktop window with all 5 interactive tabs connected directly to the Rust engine.
          </p>
        </section>
      )}
    </div>
  );
};
export default App;
