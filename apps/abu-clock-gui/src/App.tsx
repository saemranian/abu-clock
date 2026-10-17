import React, { useEffect, useState } from "react";
import {
  FRAME_RATE_PRESETS,
  FrameRatePreset,
  GuiSlateSnapshot,
  GuiTimingProfile,
  OperatingRole,
} from "./types";

const INITIAL_SNAPSHOT: GuiSlateSnapshot = {
  version: "v0.0.0.4",
  monotonic_backend: "PlatformMonotonicClock",
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
    offset_ns: 42,
    drift_ppb: 14.8,
    path_delay_ns: 380,
    offset_variance_ns2: 64.0,
    elapsed_lock_ns: 7_500_000_000,
    warmup_progress: 1.0,
    is_wireless: false,
    lock_state: "Locked",
  },
};

export const App: React.FC = () => {
  const [snapshot, setSnapshot] = useState<GuiSlateSnapshot>(INITIAL_SNAPSHOT);
  const [selectedPreset, setSelectedPreset] = useState<FrameRatePreset>(
    FRAME_RATE_PRESETS[3] // 29.97 DF default
  );
  const [offsetHistory, setOffsetHistory] = useState<number[]>([
    820, 410, 195, 88, 64, 51, 42,
  ]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setSnapshot((prev) => {
        const nextFrame = prev.frame_index + 1;
        const fps = Math.round(selectedPreset.numerator / selectedPreset.denominator);
        const f = nextFrame % fps;
        const totalSec = Math.floor(nextFrame / fps);
        const s = totalSec % 60;
        const m = Math.floor(totalSec / 60) % 60;
        const h = Math.floor(totalSec / 3600) % 24;
        const sep = selectedPreset.drop_frame ? ";" : ":";
        const pad = (n: number) => n.toString().padStart(2, "0");
        const tc = `${pad(h)}:${pad(m)}:${pad(s)}${sep}${pad(f)}.00`;
        const jitter = Math.round((Math.random() - 0.5) * 20);
        setOffsetHistory((hist) => [...hist.slice(-31), 40 + jitter]);
        return {
          ...prev,
          frame_index: nextFrame,
          slate_timecode: tc,
          flash_marker_active: f === 0,
          frame_rate_hz: selectedPreset.numerator / selectedPreset.denominator,
          drop_frame: selectedPreset.drop_frame,
          convergence: {
            ...prev.convergence,
            offset_ns: 40 + jitter,
          },
        };
      });
    }, 33);
    return () => window.clearInterval(timer);
  }, [selectedPreset]);

  return (
    <div className="slate-container">
      <header className="top-header">
        <div>
          <h1>ABU CLOCK</h1>
          <small>
            {snapshot.version} &bull; {snapshot.monotonic_backend} &bull; Domain{" "}
            {snapshot.ptp_domain}
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
            {snapshot.frame_rate_hz.toFixed(3)} Hz{" "}
            {snapshot.drop_frame ? "(Drop-Frame)" : "(Non-Drop-Frame)"}
          </div>
        </div>
        <div
          title="Optical Sync Flash Marker (Fires on Frame 00)"
          className={`optical-flash-beacon ${
            snapshot.flash_marker_active ? "active" : ""
          }`}
        />
      </section>

      <div className="grid-panels">
        <section className="card">
          <h2>Master / Follower &amp; Profile Configuration</h2>
          <label>
            Operating Mode (BMCA):
            <select
              value={snapshot.role}
              onChange={(e) =>
                setSnapshot({ ...snapshot, role: e.target.value as OperatingRole })
              }
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
              onChange={(e) =>
                setSnapshot({
                  ...snapshot,
                  profile: e.target.value as GuiTimingProfile,
                })
              }
            >
              <option value="SmpteSt2059_2">
                SMPTE ST 2059-2 Broadcast (224.0.1.129 : 319/320)
              </option>
              <option value="Ieee802_1AsGptp">
                IEEE 802.1AS gPTP LiDAR (224.0.0.107 / L2 P2P)
              </option>
              <option value="Aes67Harmonized">
                AES67 + SMPTE ST 2059-2 Harmonized (AES-R16)
              </option>
              <option value="AbuSyncCsptp">
                AbuSync Unprivileged CSPTP (UDP 50319/50320 + mDNS)
              </option>
            </select>
          </label>

          <label>
            Frame Rate / LiDAR Sweep Cadence (Hz):
            <select
              value={selectedPreset.label}
              onChange={(e) => {
                const found = FRAME_RATE_PRESETS.find(
                  (p) => p.label === e.target.value
                );
                if (found) setSelectedPreset(found);
              }}
            >
              {FRAME_RATE_PRESETS.map((p) => (
                <option key={p.label} value={p.label}>
                  {p.label}
                </option>
              ))}
            </select>
          </label>

          <label>
            PTP Domain (0–127, Default 1):
            <input
              type="number"
              min={0}
              max={127}
              value={snapshot.ptp_domain}
              onChange={(e) =>
                setSnapshot({
                  ...snapshot,
                  ptp_domain: Math.min(127, Math.max(0, Number(e.target.value))),
                })
              }
            />
          </label>
        </section>

        <section className="card">
          <h2>Kalman PLL &amp; 7-Second Warm-Up Gate Telemetry</h2>
          <div>
            Phase Offset (&theta;):{" "}
            <strong>{snapshot.convergence.offset_ns} ns</strong>
          </div>
          <div>
            Crystal Drift (&alpha;):{" "}
            <strong>{snapshot.convergence.drift_ppb.toFixed(2)} ppb</strong>
          </div>
          <div>
            One-Way Path Delay:{" "}
            <strong>{snapshot.convergence.path_delay_ns} ns</strong>
          </div>
          <div>
            7.0s ST 2059 Warm-Up Gate:{" "}
            <strong>
              {(snapshot.convergence.warmup_progress * 100).toFixed(0)}%
            </strong>
          </div>
          <div style={{ fontFamily: "monospace", fontSize: "0.82rem" }}>
            Offset History (ns): [{offsetHistory.slice(-8).join(", ")}]
          </div>
        </section>
      </div>
    </div>
  );
};
export default App;
