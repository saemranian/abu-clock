# Abu Clock: MCP Server, REST API & UDP Discovery Guide

**Version:** `v0.0.0.14`  
**MCP Server Binary:** `target\release\abu-clock-mcp.exe` (or `target\release\abu-clock.exe --mcp`)  
**HTTP Control API:** `http://127.0.0.1:7878`  
**UDP CSPTP & Discovery Ports:** `50319` (CSPTP Event) / `50320` (SNTPv4 Stratum-1 Responder & `ABU_ECO_V1` Discovery)

---

## 1. How Your AI Agent in `Abu Track & Scan` Uses `Abu Clock`

We have registered **`abu-clock`** globally in your Antigravity configuration:
1. **Global MCP Server Config:** `C:\Users\USer\.gemini\config\mcp_config.json` (`"abu-clock"` server pointing to `C:\Users\USer\Documents\antigravity\abu-clock\target\release\abu-clock-mcp.exe`)
2. **Global Antigravity Skill:** `C:\Users\USer\.gemini\config\skills\abu-clock\SKILL.md`
3. **Lazy MCP Tool Schemas:** `C:\Users\USer\.gemini\antigravity\mcp\abu-clock\*.json`

When you open your **`Abu Track & Scan`** workspace (`C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01`) and ask your AI agent:
> *"Use the `abu-clock` MCP server and skill to integrate Abu Clock timecode, auto-discovery, and sub-microsecond sync into our iOS app, Windows DesktopSync app, and UE5 LiveLink plugin"*

Your AI agent can immediately call `abu-clock` MCP tools without leaving the `Abu Track & Scan` project!

---

## 2. Available MCP Tools (`ServerName="abu-clock"`)

| MCP Tool Name | Description | Example Arguments |
| :--- | :--- | :--- |
| **`abu_clock_get_status`** | Returns live TAI nanoseconds, SMPTE ST 12-1 timecode (`HH:MM:SS;FF.sf`), 32-bit BCD word, 2-state Kalman PLL offset/drift, 20-byte ST 2059-2 TLV hex, and discovered ecosystem nodes. | `{}` |
| **`abu_clock_configure`** | Updates Operating Role (`AutoBmca`/`Master`/`Follower`), Profile (`AbuSyncCsptp`/`SmpteSt2059_2`/`Ieee802_1AsGptp`), Domain, Frame Rate (`60/1`, `30000/1001`), Phase Trim, and Lead Time. | `{"role": "AutoBmca", "profile": "AbuSyncCsptp", "fps_num": 60, "fps_den": 1, "drop_frame": false}` |
| **`abu_clock_synthesize_timecode`** | Converts any TAI nanosecond timestamp into SMPTE ST 12-1 timecode, 32-bit BCD word, frame index, sub-frame phase $\phi \in [0.0, 1.0)$, and 20B ST 2059-2 TLV. | `{"tai_ns": 1729180800000000000, "fps_num": 60, "fps_den": 1, "drop_frame": false}` |
| **`abu_clock_schedule_capture`** | Fires a `SCHEDULE_CAPTURE_AT` future-time barrier trigger ($T_{\text{now}} + \Delta T_{\text{lead}}$) and returns deterministic filenames, multi-rig jitter receipts, and `.abuscan.json` + COLMAP `images.txt`. | `{"mode": "single", "frames": 1}` |
| **`abu_clock_pack_telemetry`** | Packs a 6-DoF camera pose + lens intrinsics (`focalLength`, `k1`, `k2`) into **29B FreeD (`0xD1`)**, **37B AbuTrack FreeD (`0xD1 + int64_ms`)**, **41B FreeD-TC (`0xD2`)**, **AbuTrack LiveLink JSON**, and **OpenTrackIO (`OTrk`)**. | `{"camera_id": 1, "pan_deg": 15.0, "tilt_deg": -5.0, "roll_deg": 0.2, "x_mm": 1200.0, "y_mm": -300.0, "z_mm": 1700.0, "focal_length_mm": 24.0}` |
| **`abu_clock_discover_nodes`** | Discovers or registers active **Abu Ecosystem** nodes (`AbuClockMaster`, `AbuTrackIos`, `AbuTrackDesktop`, `AbuTrackUnreal`, `LidarSensor`, `CameraRig`). | `{"register_node_id": "abutrack-win-01", "register_kind": "AbuTrackDesktop", "register_ip": "192.168.1.10"}` |
| **`abu_clock_get_integration_guide`** | Returns complete architectural recommendations and drop-in Swift, Python, and UE5 C++ integration code for `Abu Track & Scan`. | `{"target": "all"}` (or `"ios_swift"`, `"windows_python"`, `"unreal_cpp"`) |

---

## 3. HTTP REST JSON API (`http://127.0.0.1:7878`)

When the **Abu Clock** PC Studio app (`target\release\abu-clock.exe`) is running, any local or LAN application can call its CORS-enabled HTTP endpoints:

### `GET /api/state`
Returns the complete `GuiSlateSnapshot` JSON containing live timecode, BCD, TAI ns, Kalman metrics, 20B ST 2059-2 TLV, barrier receipts, 29B/37B/41B FreeD & OpenTrackIO packets, `.abuscan.json` manifest, and ecosystem nodes.

### `GET /api/ecosystem/discover`
Returns the `AbuEcosystemDiscoveryBeacon` JSON:
```json
{
  "magic": "ABU_ECO_V1",
  "version": "v0.0.0.14",
  "master_ip": "127.0.0.1",
  "csptp_event_port": 50319,
  "sntp_general_port": 50320,
  "http_api_port": 7878,
  "abutrack_telemetry_port": 8080,
  "abutrack_video_port": 5004,
  "ptp_domain": 1,
  "tai_ns": 1729180808000000000,
  "smpte_timecode": "00:00:08;00.00",
  "bcd_hex": "0x00000800",
  "fps_num": 30000,
  "fps_den": 1001,
  "drop_frame": true,
  "trigger_lead_time_ms": 50,
  "nodes": [...]
}
```

### `POST /api/ecosystem/register`
Registers or refreshes an `AbuEcosystemNodeInfo` entry so all other nodes (`iOS`, `PC`, `UE5`) can discover its IP and ports.

### `GET /api/sntp` (and UDP Port `50320`)
Returns a Stratum-1 RFC 4330 48-byte SNTPv4 response (`ref_id = "ABUC"`) plus `rx_tai_ns`, `tx_tai_ns`, `smpte_timecode`, and `bcd_hex`.
- On **UDP Port `50320`**, sending a 48-byte SNTP client request (`packet[0] = 0x1B`, matching `NTPSyncClient.swift`) returns a binary 48-byte Stratum-1 SNTP server packet immediately!
- Sending `"ABU_ECO_V1"` or `"DISCOVER_ABU"` on **UDP Port `50320`** returns the JSON `AbuEcosystemDiscoveryBeacon`!

### `POST /api/config`, `POST /api/clock`, `POST /api/bmca`, `POST /api/trigger`, `POST /api/telemetry`, `POST /api/manifest`
Control the live Rust engine parameters, Kalman fault injection, BMCA election, `SCHEDULE_CAPTURE_AT` barrier triggers, 6-DoF telemetry synthesis, and session manifest generation.
