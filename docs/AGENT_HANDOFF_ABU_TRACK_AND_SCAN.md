# AI Developer Agent Handoff Brief: Integrating `Abu Clock` into `Abu Track & Scan`

**Target Project Workspace:** `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01` (`Abu Track & Scan` / `CamTrack-Pro-01`)  
**Master Clock Infrastructure Workspace:** `c:\Users\USer\Documents\antigravity\abu-clock` (`Abu Clock v0.0.0.14`)

---

## 1. Mission Statement for the `Abu Track & Scan` AI Agent

You are developing **`Abu Track & Scan`** (consisting of the **iOS Swift App**, **Windows Python Desktop Sync & Monitor App**, and **Unreal Engine 5 `LiveLinkAbuTrack` C++ Plugin**).

Instead of maintaining isolated, single-shot NTP wall-clock offsets or arrival-time UDP timestamps, **`Abu Track & Scan` must use `Abu Clock` as its unified temporal & synchronization backbone (`AbuClockCore` + `AbuNetwork` + `AbuClock` Daemon/MCP/API)** so that:
1. All Abu Ecosystem apps (**`Abu Clock`**, **`Abu Track & Scan` iOS/PC/UE5**, **`Abu Cam`**, and **Hardware Grandmaster Appliances**) **auto-discover each other on the LAN** with zero manual IP entry.
2. Every camera frame, ARKit 6-DoF pose, LiDAR depth map, and DSLR raw photo is stamped at its **exact optical exposure midpoint** ($T_{\text{mid}} = T_{\text{SOF}} + T_{\text{integration}} / 2$) in **SMPTE ST 12-1 / ST 2059-2 Timecode (`HH:MM:SS:FF.sf`)** and **64-bit TAI nanoseconds**.
3. Multi-rig 3D Gaussian Splatting (3DGS) / photogrammetry shoots use **Future-Time Scheduled Capture Barriers (`SCHEDULE_CAPTURE_AT = T_now + 50 ms`)** alongside direct `.abuscan.json` and COLMAP `images.txt` export.
4. Unreal Engine 5 locks its render frame rate and LiveLink interpolation to `FQualifiedFrameTime` (`bcdTimecode` + `subframePhase`) instead of Wi-Fi packet arrival time (`FPlatformTime::Seconds()`).

---

## 2. How to Access `Abu Clock` Tools & Skills Immediately (MCP & Global Skill)

`Abu Clock` is already installed and registered globally in your Antigravity environment. You do **not** need to build a clock engine from scratch.

### A. Global Antigravity Skill (Read First)
- **Skill File:** `C:\Users\USer\.gemini\config\skills\abu-clock\SKILL.md`

### B. Native `abu-clock` MCP Server (Call via `call_mcp_tool` with `ServerName: "abu-clock"`)
- **MCP Server Config:** `C:\Users\USer\.gemini\config\mcp_config.json`
- **Executable Binary:** `c:\Users\USer\Documents\antigravity\abu-clock\target\release\abu-clock-mcp.exe`
- **Lazy Tool Schemas Directory:** `C:\Users\USer\.gemini\antigravity\mcp\abu-clock\`
  1. `abu_clock_get_status` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_get_status.json`) — Inspect live TAI nanoseconds, SMPTE timecode, Kalman PLL lock state, 37B/41B FreeD hex, OpenTrackIO, and active ecosystem nodes.
  2. `abu_clock_configure` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_configure.json`) — Configure frame rate (`23.976`–`120 fps` + LiDAR `10/15/20 Hz`), Drop-Frame, PTP Domain, Role, Sync Profile, and lead time.
  3. `abu_clock_synthesize_timecode` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_synthesize_timecode.json`) — Convert any `tai_ns` timestamp into SMPTE `HH:MM:SS:FF.sf` and 32-bit BCD.
  4. `abu_clock_schedule_capture` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_schedule_capture.json`) — Trigger a future-time multi-sensor capture barrier ($T_{\text{now}} + 50\text{ ms}$) and return `.abuscan.json` and COLMAP `images.txt`.
  5. `abu_clock_pack_telemetry` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_pack_telemetry.json`) — Pack 6-DoF pose into 29B FreeD (`0xD1`), **37B `Abu Track & Scan` Extended FreeD**, **41B Abu Extended FreeD (`0xD2`)**, OpenTrackIO (`OTrk`), and **Enriched `AbuTrack` LiveLink JSON**.
  6. `abu_clock_discover_nodes` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_discover_nodes.json`) — Query or register active `Abu Track & Scan` iOS, Windows Desktop, and UE5 nodes on the LAN.
  7. `abu_clock_get_integration_guide` (`C:\Users\USer\.gemini\antigravity\mcp\abu-clock\abu_clock_get_integration_guide.json`) — Fetch ready-to-use code snippets (`topic`: `"overview"`, `"ios_swift"`, `"python_pc"`, `"unreal_cpp"`, `"freed_37b"`).

---

## 3. Essential Documentation & Code Paths to Read

Before modifying `Abu Track & Scan`, read these reference guides and drop-in SDK adapters in the `abu-clock` repository:

### Core Integration & API Guides
1. **Master `Abu Track & Scan` Integration Blueprint:**  
   `c:\Users\USer\Documents\antigravity\abu-clock\docs\ABU_TRACK_AND_SCAN_INTEGRATION_GUIDE.md`
2. **MCP Server & Local HTTP REST API Guide (`127.0.0.1:7878` & UDP `50320`):**  
   `c:\Users\USer\Documents\antigravity\abu-clock\docs\MCP_AND_API_GUIDE.md`
3. **Developer & Systems Integration Guide (FFI, C/C++, Swift, Python, UE5, Omniverse):**  
   `c:\Users\USer\Documents\antigravity\abu-clock\docs\DEVELOPER_GUIDE.md`
4. **User Manual & Studio Operator Guide:**  
   `c:\Users\USer\Documents\antigravity\abu-clock\docs\USER_MANUAL.md`
5. **`Abu Track & Scan` Existing Architecture Reference:**  
   `c:\Users\USer\.gemini\antigravity\brain\06f20761-50da-4ab4-b9c3-caf0e1bf6e0d\abu_track_and_scan_documentation.md`

### Ready-Made SDK & Adapter Files in `abu-clock`
- **iOS Swift SDK (`AbuClockKit`):**  
  `c:\Users\USer\Documents\antigravity\abu-clock\integrations\abu-track-scan\ios-swift\AbuClockKit.swift`
- **Windows C++ Aggregator Client (`AbuClockWinClient`):**  
  `c:\Users\USer\Documents\antigravity\abu-clock\integrations\abu-track-scan\windows-cpp\AbuClockWinClient.hpp`
- **C/C++ Native ABI Header (`abu_clock.h`):**  
  `c:\Users\USer\Documents\antigravity\abu-clock\crates\abu-ffi\include\abu_clock.h`
- **Unreal Engine 5 `AbuClockUE` Plugin (`UTimecodeProvider` + `UCustomTimeStep` + LiveLink):**  
  - `c:\Users\USer\Documents\antigravity\abu-clock\integrations\unreal-plugin\AbuClockUE\Source\AbuClockUE\Public\AbuClockTimecodeProvider.h`
  - `c:\Users\USer\Documents\antigravity\abu-clock\integrations\unreal-plugin\AbuClockUE\Source\AbuClockUE\Public\AbuClockCustomTimeStep.h`
  - `c:\Users\USer\Documents\antigravity\abu-clock\integrations\unreal-plugin\AbuClockUE\Source\AbuClockUE\Public\AbuTrackLiveLinkSource.h`
- **Rust Ecosystem & 37B/41B FreeD Wire Reference:**  
  - `c:\Users\USer\Documents\antigravity\abu-clock\crates\abu-network\src\ecosystem.rs`
  - `c:\Users\USer\Documents\antigravity\abu-clock\crates\abu-network\src\telemetry\freed.rs`

---

## 4. Concrete Implementation Checklist in `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01`

### Step 1 — iOS App (`CamTrack/`)
- **Target Files:**
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\CamTrack\Network\NTPSyncClient.swift`
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\CamTrack\Network\UDPStreamingManager.swift`
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\CamTrack\Core\TrackingSessionManager.swift`
- **Upgrades to Implement:**
  1. **Unprivileged Stratum-1 Sync & Auto-Discovery (`UDP 50320`):** Update `NTPSyncClient.swift` to support `Abu Clock`'s unprivileged Stratum-1 SNTPv4 responder on UDP port `50320` (avoiding Windows port `123` privilege blocks) AND broadcast `"DISCOVER_ABU"` on UDP port `50320` to receive the `AbuEcosystemDiscoveryBeacon` (`"ABU_ECO_V1"`) containing the PC workstation and UE5 IP addresses automatically.
  2. **2-State Kalman PLL & Exposure Midpoint Timestamping (`AbuClockKit.swift`):** Embed `AbuClockNode` from `AbuClockKit.swift` into `TrackingSessionManager.swift` to compute exact optical exposure midpoint timestamps ($T_{\text{mid}} = T_{\text{SOF}} + T_{\text{integration}} / 2$) for every `ARFrame`.
  3. **Enriched LiveLink JSON & 37B/41B FreeD (`UDPStreamingManager.swift`):** Include `smpteTimecode`, `bcdTimecode`, `taiNs`, `subframePhase`, and `clockLockState` in `LiveLinkPacket` (matching `AbuTrackLiveLinkPacket` in `ecosystem.rs`), and add an option to emit 41-byte `0xD2` Extended FreeD alongside the existing 37-byte `0xD1` Extended FreeD.

### Step 2 — Windows Desktop Sync & Monitor App (`DesktopSync/`)
- **Target Files:**
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\DesktopSync\camtrack_desktop_gui.py`
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\DesktopSync\sync_photos.py`
- **Upgrades to Implement:**
  1. **Register with `Abu Clock` Ecosystem Registry:** On startup in `camtrack_desktop_gui.py`, call `POST http://127.0.0.1:7878/api/ecosystem/register` (with `node_kind: "AbuTrackDesktop"`, `telemetry_port: 8080`, `video_rtp_port: 5004`) and poll `GET http://127.0.0.1:7878/api/ecosystem/discover` so the iOS device and UE5 node appear automatically in the UI.
  2. **Display Live SMPTE Timecode & PLL Lock State:** Read `smpteTimecode`, `taiNs`, and `clockLockState` from incoming packets (or `GET http://127.0.0.1:7878/api/status`) and display the master SMPTE timecode on the `DesktopSync` dashboard.
  3. **Future-Time Capture Barrier (`POST /api/trigger`) & COLMAP Export:** Add a button in `camtrack_desktop_gui.py` and logic in `sync_photos.py` to trigger `POST http://127.0.0.1:7878/api/trigger` ($T_{\text{now}} + 50\text{ ms}$) and export `.abuscan.json` + COLMAP `images.txt` with exact TAI nanosecond exposure alignment.

### Step 3 — Unreal Engine 5 Plugin (`UnrealPlugin/LiveLinkAbuTrack/`)
- **Target Files:**
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\UnrealPlugin\LiveLinkAbuTrack\Source\LiveLinkAbuTrack\Private\LiveLinkAbuTrackSource.cpp`
  - `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01\UnrealPlugin\LiveLinkAbuTrack\Source\LiveLinkAbuTrack\Public\LiveLinkAbuTrackSource.h`
- **Upgrades to Implement:**
  1. **Parse Enriched SMPTE Fields in `ParseAndPushJSON`:** Extract `"bcdTimecode"`, `"subframePhase"`, `"taiNs"`, and `"smpteTimecode"` from the JSON object when present (and also parse 41-byte `0xD2` FreeD packets alongside 29-byte and 37-byte `0xD1` packets in `ParseAndPushFreeD`).
  2. **Populate `CameraFrameData.MetaData.SceneTime`:** Convert `bcdTimecode` + `subframePhase` into `FQualifiedFrameTime(FFrameTime(FrameNumber, SubframePhase), FrameRate)` so UE5's LiveLink buffer interpolates camera poses using the exact iOS sensor exposure timestamp rather than `FPlatformTime::Seconds()` UDP arrival time.
  3. **Link `UAbuClockTimecodeProvider` & `UAbuClockCustomTimeStep`:** Include or bridge the classes from `c:\Users\USer\Documents\antigravity\abu-clock\integrations\unreal-plugin\AbuClockUE\` so UE5's engine tick and master timecode lock to `Abu Clock`.
