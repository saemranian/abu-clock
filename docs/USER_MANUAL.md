# Abu Clock — Official User Manual & Studio Operator Guide (`v0.0.0.13`)

Welcome to the **Abu Clock** User Manual. This document provides a complete, step-by-step operational guide for Virtual Production supervisors, Broadcast engineers, 3D Gaussian Splatting (3DGS) / Photogrammetry capture operators, and LiDAR technicians using the **Abu Clock** PC application.

---

## 1. Quick Start: Launching Abu Clock

### Method A: One-Click Windows Launcher (Recommended)
Double-click **[`Start-AbuClock.bat`](../Start-AbuClock.bat)** in the root folder of the repository.

### Method B: Command Line (PowerShell)
```powershell
cd c:\Users\USer\Documents\antigravity\abu-clock
cargo run --release --bin abu-clock
```

When launched, **Abu Clock**:
1. Starts the native Rust timing, PTP, trigger, and telemetry engine on `http://127.0.0.1:7878` (automatically replacing any older running instance).
2. Binds a real `TimestampedUdpSocket` (`127.0.0.1:50319`) for unprivileged CSPTP and telemetry packets.
3. Opens the chromeless native desktop window titled **`Abu Clock v0.0.0.13 — Precision Master Clock, Timecode & Sync Studio`**.

---

## 2. Understanding the Top Status Bar & Master Digital Slate

The top portion of the **Abu Clock** window remains visible across all 5 studio tabs so you never lose sight of your master timecode or synchronization health.

```
+------------------------------------------------------------------------------------------------------+
| [Clock Icon] ABU CLOCK [v0.0.0.13]                     [LOCKED] [PORT: MASTER] [WIRED LAN] [WAKE ON] |
| Oscillator: Windows::QueryPerformanceCounter • Socket: 127.0.0.1:50319 (TX: 142 / RX: 142)           |
+------------------------------------------------------------------------------------------------------+
|                                                                                                      |
|   16:28:56;06.94                         Sub-Frame Phase (φ): 0.943  [==================  ]          |
|   Frame #51823600610 | BCD 0x16285606 | Rate 29.970 Hz (DF) | Domain 1 | TAI 1729180807... [FLASH]   |
|                                                                                                      |
+------------------------------------------------------------------------------------------------------+
```

### Status Badges (Top Right)
| Badge | Color | Meaning |
| :--- | :--- | :--- |
| **`LOCKED`** | Emerald Green | The 7.0-second SMPTE ST 2059-2 convergence gate is 100% complete and phase error is `< 100 ns` (Wired) or `< 500 µs` (Wi-Fi). Safe for production shooting. |
| **`WARMING UP`** | Amber | The Kalman servo is actively slewing the clock during the mandatory 7.0-second ST 2059-2 warm-up window. Wait for `LOCKED` before rolling cameras. |
| **`HOLDOVER`** | Violet | Network packets from the Grandmaster were interrupted; the local clock is coasting smoothly on its last learned crystal drift ratio ($\alpha_{\text{freq}}$ in ppb). |
| **`UNLOCKED`** | Rose Red | Residual phase offset exceeds synchronization tolerances (`> 100 ns` wired or `> 500 µs` wireless). |

### Master Digital Slate Readout Fields
- **Primary Timecode (`HH:MM:SS;FF.sf` or `HH:MM:SS:FF.sf`):**
  - Uses a **semicolon (`;`)** when **Drop-Frame (`29.97 DF` / `59.94 DF`)** is active.
  - Uses a **colon (`:`)** when **Non-Drop-Frame (`NDF`)** is active.
  - `.sf` displays hundredths of a frame (`00`..`99`) for high-speed camera slate verification.
- **BCD (32-bit):** Hexadecimal representation (`0xHHMMSSFF`) embedded into SMPTE ST 12-1, OpenTrackIO, and Extended FreeD-TC (`0xD2`) packets.
- **Sub-Frame Phase ($\phi$):** Continuous normalized progress (`0.000` $\rightarrow$ `1.000`) within the current frame period.
- **FRAME 00 FLASH Beacon:** Flashes pure white (`#FFFFFF`) during the first quarter ($\phi < 0.25$) of Frame `00` every second so high-speed optical cameras can visually verify zero-latency slate alignment.

---

## 3. Step-by-Step Guide to the 5 Studio Tabs

### Tab 1: `1. Clock & Kalman PLL` (Settings, Presets & Servo Oscilloscope)

#### Using 1-Click Studio Scenario Presets
At the top of Tab 1, click any preset chip to configure the entire engine in one click:
1. **Virtual Production (`24 fps + ST 2059-2`):** Sets Forced Grandmaster (`Priority1 = 64`), `SMPTE ST 2059-2`, Domain `1`, and `24.000 fps NDF`.
2. **Broadcast NTSC (`29.97 DF`):** Sets `Auto BMCA`, `SMPTE ST 2059-2`, Domain `1`, and `29.97 fps Drop-Frame (30000/1001)`.
3. **RoboSense / XGRIDS LiDAR (`10 Hz gPTP`):** Sets Forced Grandmaster, `IEEE 802.1AS gPTP`, Domain `0`, and `10.000 Hz` LiDAR sweep cadence.
4. **UE5 & Omniverse (`60 fps`):** Sets `Auto BMCA`, `AbuSync CSPTP (UDP 50319)`, Domain `1`, and `60.000 fps NDF`.
5. **Mobile Wi-Fi Follower Rig:** Sets `Follower Only`, `AbuSync CSPTP`, and enables the adaptive Wireless Wi-Fi Kalman EMA variance filter.

#### Customizing Parameters Manually
1. Choose your **Operating Role**, **Synchronization Profile**, **PTP Domain (`0–127`)**, and **BMCA Priority 1 / Priority 2**.
2. Select your **Rational Frame Rate**, **User Phase Trim Offset (`ns`)** (useful for compensating LED wall scanout delay), and **Barrier Trigger Lead Time (`ms`)** (default `50 ms`).
3. Click **Apply Settings to Rust Engine**. A confirmation toast appears in the bottom-right corner and the Master Digital Slate updates immediately.

#### Using the 2-State Kalman PLL & Fault Injection Test Bench
- **8 Hz Auto-Sync Stream Button:** Keeps the background 8 Hz servo ticker running so the 40-sample SVG oscilloscope updates continuously in real time. Click to pause/resume when inspecting static fault states.
- **Inject Clean PTP Tick (`±9 ns`):** Feeds a low-jitter wired Ethernet sample into `OverlayClock`.
- **Inject Wi-Fi Spike (`+1,850 ns`):** Simulates a wireless access point queuing burst.
- **Inject `+50 µs` Disturbance:** Injects a sudden $50,000\text{ ns}$ phase disturbance so you can watch the oscilloscope spike and the Kalman filter slew back to zero.
- **Fast-Forward 7.0s Warm-Up (Converge & Lock):** Runs 64 closed-loop 8 Hz servo steps (`8.0 s` simulated duration) to immediately complete the mandatory 7.0-second ST 2059-2 warm-up gate and lock the clock `< 100 ns`.

---

### Tab 2: `2. PTP & ST 2059-2 TLV` (BMCA Failover & Wire Hex Inspector)

1. **Test IEEE 1588-2019 BMCA Election (Left Panel):**
   - Enter a simulated remote node's `Priority 1` (e.g. `64`), `Clock Class` (`6`), `Priority 2` (`128`), and advertised frame rate (e.g. `24.000 fps`).
   - Click **Inject Remote Announce & Run BMCA Election**.
   - Because `Priority 1 = 64` is lower (better) than the default local `128`, the local port automatically transitions to `PORT: SLAVE` and adopts the remote Grandmaster's `24.000 fps` frame rate.
   - To make your local PC win election again, set `Remote Priority 1` to `200` and click the button again — your local node reclaims `PORT: MASTER`.
2. **Inspect the Live SMPTE ST 2059-2 TLV (Right Panel):**
   - View the exact 20-byte big-endian hexadecimal payload (`00 03 00 10 00 0E 9B 00 00 01 ...`) generated by `SmpteSyncMetadataTlv::to_bytes()`.
   - Click **Copy Hex** to copy the wire bytes to your clipboard for Wireshark or packet-generator testing.

---

### Tab 3: `3. Trigger & Telemetry` (+50ms Barrier, FreeD/OpenTrackIO & Manifests)

1. **Fire Multi-Sensor Barrier Captures (Top-Left Panel):**
   - Click **Fire SCHEDULE_CAPTURE_AT (+Lead Time)** to broadcast a future-time single shutter command ($T_{\text{exec}} = T_{\text{now}} + 50\text{ ms}$), or click **Schedule 5-Frame Synchronized Burst Scan**.
   - Review the **Latest Multi-Rig Barrier Receipts** table showing each rig (`iPhone16Pro_CamA`, `WinRig_CamB`, `RoboSense_E1R_LiDAR`, `UE5_nDisplay_Node01`), its nanosecond execution jitter, and its deterministic filename (`<SensorID>_<TC>_<TAI>.<ext>`).
2. **Synthesize 6-DoF Camera Tracking Packets (Top-Right Panel):**
   - Adjust `Camera ID`, `Pan (°)`, `Tilt (°)`, `Roll (°)`, `Pos X/Y/Z (mm)`, and `Lens Zoom`.
   - Click **Synthesize OpenTrackIO & FreeD Packets**.
   - Inspect and copy the live **29-byte Standard FreeD (`0xD1`)** hex dump, **41-byte Extended AbuTrack FreeD-TC (`0xD2`)** hex dump, and **SMPTE RIS OpenTrackIO (`OTrk`)** JSON payload.
3. **Export Synchronized Session Manifests (Bottom Panel):**
   - Enter a sensor name (e.g. `SonyA1_Rig03`) and file extension (`arw` or `dng`) and click **Stamp & Append Sensor Capture to Manifest**.
   - Click **Copy JSON** to copy the `.abuscan.json` manifest or **Copy Colmap** to copy the Colmap / Jawset Postshot `images.txt` pose table for 3D Gaussian Splatting (3DGS).

---

### Tab 4: `4. Grandmaster Appliance` (Pi CM4 + DS3231 TCXO + PHC)

- Monitors the Phase 3 dedicated hardware Grandmaster configuration (`Raspberry Pi CM4 / Pi 5 + Maxim DS3231 TCXO ±2 ppm + Intel I210-T1 PCIe PHC`).
- Click **Lock Primary Reference (Class 6)** or **Simulate Antenna Loss / Holdover Mode (Class 7)** to verify seamless PTP clock-class degradation during GNSS/antenna holdover.

---

### Tab 5: `5. Visual Guide & SDK` (Interactive Built-In Help)

Provides four interactive sub-tabs inside the application:
1. **Interactive Visual Studio Tour:** Annotated SVG map of the interface + 6 step cards with live **"Try It Now"** action buttons.
2. **Core Engineering Fundamentals & Diagrams:** Visual SVG comparison of Naive UDP triggers vs. `SCHEDULE_CAPTURE_AT` barrier triggers and the Kalman PLL state machine.
3. **Developer SDK & Engine Integration Guide:** Copyable C/C++17, Python/Omniverse, and HTTP REST API reference.
4. **Studio Operator Checklist & Troubleshooting:** Pre-shoot verification table and common fixes.
