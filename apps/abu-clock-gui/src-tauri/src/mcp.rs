//! Native Model Context Protocol (MCP) Server for **Abu Clock** (`abu-clock-mcp`).
//!
//! Implements JSON-RPC 2.0 over `stdio` so AI agents (such as Antigravity agents working on
//! `Abu Track & Scan` in `C:\Users\USer\.gemini\antigravity\scratch\ios_app_cam_track_01`)
//! can query clock state, configure SMPTE/PTP profiles, synthesize timecodes, pack 29B/37B/41B
//! FreeD & OpenTrackIO telemetry, discover Abu Ecosystem nodes, and retrieve drop-in Swift,
//! Python (`camtrack_desktop_gui.py`), and UE5 C++ (`LiveLinkAbuTrackSource.cpp`) integration code.

use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::time::Duration;

use abu_clock_core::{FrameRate, SmpteTimecodeEngine, ABU_CLOCK_VERSION};
use abu_network::{
    AbuEcosystemNodeInfo, AbuEcosystemNodeKind, AbuTrackLiveLinkPacket, CameraPose6DoF,
    FreeDPacket, OpenTrackIoEncoding, OpenTrackIoPacket, OpenTrackIoPayload,
    SmpteSyncMetadataTlv,
};
use serde_json::{json, Value};

use crate::{ConfigUpdateRequest, GuiAppController, GuiTimingProfile, OperatingRole};

/// Runs the MCP JSON-RPC 2.0 server loop over `stdin` / `stdout`.
pub fn run_mcp_stdio_server() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut ctrl = GuiAppController::new();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(resp) = handle_mcp_request(&mut ctrl, &req) {
            let out = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
            writeln!(stdout, "{}", out)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// Attempts an HTTP request against a live running `Abu Clock` Studio instance on `127.0.0.1:7878`.
fn try_live_studio_http(method: &str, path: &str, body: Option<&str>) -> Option<Value> {
    let mut stream =
        TcpStream::connect_timeout(&"127.0.0.1:7878".parse().ok()?, Duration::from_millis(120))
            .ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));

    let payload = body.unwrap_or("");
    let req = if method == "POST" {
        format!(
            "POST {} HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            path,
            payload.len(),
            payload
        )
    } else {
        format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nConnection: close\r\n\r\n",
            path
        )
    };
    stream.write_all(req.as_bytes()).ok()?;
    let mut resp_buf = String::new();
    use std::io::Read;
    stream.read_to_string(&mut resp_buf).ok()?;
    let json_part = resp_buf
        .split("\r\n\r\n")
        .nth(1)
        .or_else(|| resp_buf.split("\n\n").nth(1))?;
    serde_json::from_str(json_part).ok()
}

/// Handles a single JSON-RPC 2.0 MCP message and returns `Some(response)` for requests
/// or `None` for notifications.
pub fn handle_mcp_request(ctrl: &mut GuiAppController, req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));

    // Notifications (no id) do not return a response
    if id.is_none() || method.starts_with("notifications/") {
        return None;
    }
    let req_id = id.unwrap_or(Value::Null);

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "abu-clock-mcp",
                    "version": ABU_CLOCK_VERSION
                }
            }
        })),
        "ping" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {}
        })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": mcp_tool_definitions()
            }
        })),
        "tools/call" => {
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let tool_output = execute_mcp_tool(ctrl, tool_name, &args);
            let pretty =
                serde_json::to_string_pretty(&tool_output).unwrap_or_else(|_| "{}".to_string());
            Some(json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": pretty
                        }
                    ],
                    "isError": false
                }
            }))
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        })),
    }
}

/// Returns the JSON Schema definitions for all 7 `abu-clock` MCP tools.
pub fn mcp_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "abu_clock_get_status",
            "description": "Query live Abu Clock Master status, including TAI nanoseconds, SMPTE ST 12-1 timecode (HH:MM:SS;FF.sf), 32-bit BCD word, 2-state Kalman PLL offset/drift, 20-byte SMPTE ST 2059-2 TLV, and discovered Abu Ecosystem nodes.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "abu_clock_configure",
            "description": "Configure the Abu Clock engine parameters (Operating Role, Sync Profile, PTP Domain, Rational Frame Rate, Drop-Frame, User Phase Trim Offset in ns, Barrier Trigger Lead Time in ms, and Wired/Wi-Fi medium).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "role": { "type": "string", "enum": ["AutoBmca", "Master", "Follower"], "description": "Operating role (AutoBmca, Master, or Follower)" },
                    "profile": { "type": "string", "enum": ["AbuSyncCsptp", "SmpteSt2059_2", "Ieee802_1AsGptp", "Aes67Harmonized"], "description": "Synchronization profile" },
                    "ptp_domain": { "type": "integer", "description": "IEEE 1588 PTP domain number (0-127, default 1, or 0 for gPTP LiDAR)" },
                    "fps_num": { "type": "integer", "description": "Frame rate numerator (e.g. 60, 30000, 24000, 24, 25, 10)" },
                    "fps_den": { "type": "integer", "description": "Frame rate denominator (e.g. 1 or 1001)" },
                    "drop_frame": { "type": "boolean", "description": "Enable SMPTE ST 12-1 Drop-Frame compensation (for 29.97 / 59.94)" },
                    "user_offset_ns": { "type": "integer", "description": "User phase trim offset in nanoseconds" },
                    "trigger_lead_time_ms": { "type": "integer", "description": "Future-time barrier trigger lead time in milliseconds (default 50)" },
                    "is_wireless": { "type": "boolean", "description": "True for Wi-Fi (<500us target), false for Wired LAN (<100ns target)" }
                }
            }
        }),
        json!({
            "name": "abu_clock_synthesize_timecode",
            "description": "Convert any TAI nanosecond timestamp (or current disciplined clock time) into SMPTE ST 12-1 timecode (HH:MM:SS;FF.sf), 32-bit packed BCD word, monotonic frame index, sub-frame phase [0.0, 1.0), and 20-byte SMPTE ST 2059-2 TLV hex.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tai_ns": { "type": "integer", "description": "Optional TAI timestamp in nanoseconds (uses live clock if omitted)" },
                    "fps_num": { "type": "integer", "description": "Rational frame rate numerator (default 30000)" },
                    "fps_den": { "type": "integer", "description": "Rational frame rate denominator (default 1001)" },
                    "drop_frame": { "type": "boolean", "description": "SMPTE Drop-Frame flag (default true)" },
                    "user_offset_ns": { "type": "integer", "description": "Phase trim offset in nanoseconds (default 0)" }
                }
            }
        }),
        json!({
            "name": "abu_clock_schedule_capture",
            "description": "Fire a future-time SCHEDULE_CAPTURE_AT multi-sensor barrier trigger (+LeadTime ms) across iOS, Windows, LiDAR, and UE5 rigs, returning deterministic filenames, per-sensor jitter receipts, and .abuscan.json + COLMAP images.txt manifest records.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mode": { "type": "string", "enum": ["single", "burst"], "description": "Single barrier capture or multi-frame interval burst" },
                    "frames": { "type": "integer", "description": "Number of frames when mode is 'burst' (default 5)" }
                }
            }
        }),
        json!({
            "name": "abu_clock_pack_telemetry",
            "description": "Pack a 6-DoF camera pose + lens intrinsics + live SMPTE timecode into all 4 studio telemetry wire formats: (1) 29-byte Standard FreeD (0xD1), (2) 37-byte Abu Track & Scan Extended FreeD (0xD1 + int64_ms), (3) 41-byte AbuClock FreeD-TC (0xD2 + BCD + TAI ns), and (4) AbuTrack LiveLink JSON + SMPTE RIS OpenTrackIO (OTrk).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "camera_id": { "type": "integer", "description": "Camera ID (0-255, default 1)" },
                    "pan_deg": { "type": "number", "description": "Pan/Yaw angle in degrees" },
                    "tilt_deg": { "type": "number", "description": "Tilt/Pitch angle in degrees" },
                    "roll_deg": { "type": "number", "description": "Roll angle in degrees" },
                    "x_mm": { "type": "number", "description": "Position X in millimeters" },
                    "y_mm": { "type": "number", "description": "Position Y in millimeters" },
                    "z_mm": { "type": "number", "description": "Position Z in millimeters" },
                    "focal_length_mm": { "type": "number", "description": "Camera focal length in mm (default 24.0)" },
                    "k1": { "type": "number", "description": "Radial distortion k1 (default 0.0)" },
                    "k2": { "type": "number", "description": "Radial distortion k2 (default 0.0)" }
                }
            }
        }),
        json!({
            "name": "abu_clock_discover_nodes",
            "description": "Discover or register active Abu Ecosystem nodes (AbuClockMaster, AbuTrackIos, AbuTrackDesktop, AbuTrackUnreal, LidarSensor, CameraRig) and return the live AbuEcosystemDiscoveryBeacon with all UDP/RTP/CSPTP/SNTP/HTTP ports.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "register_node_id": { "type": "string", "description": "Optional node_id to register/heartbeat into the Abu Ecosystem mesh" },
                    "register_display_name": { "type": "string", "description": "Optional human-readable node name" },
                    "register_ip": { "type": "string", "description": "Optional IPv4 address of the node" },
                    "register_kind": { "type": "string", "enum": ["AbuClockMaster", "AbuTrackIos", "AbuTrackDesktop", "AbuTrackUnreal", "LidarSensor", "CameraRig"] }
                }
            }
        }),
        json!({
            "name": "abu_clock_get_integration_guide",
            "description": "Retrieve complete architectural recommendations and drop-in integration code for connecting 'Abu Track & Scan' (iOS Swift app, Windows Python DesktopSync workstation, UE5 LiveLinkAbuTrack C++ plugin, and DSLR Hot-Shoe sync_photos.py) to the Abu Clock infrastructure.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "enum": ["all", "ios_swift", "windows_python", "unreal_cpp", "dslr_sync", "c_ffi"],
                        "description": "Which platform integration blueprint to return (default: 'all')"
                    }
                }
            }
        }),
    ]
}

/// Executes one of the 7 `abu-clock` MCP tools.
pub fn execute_mcp_tool(ctrl: &mut GuiAppController, tool_name: &str, args: &Value) -> Value {
    match tool_name {
        "abu_clock_get_status" => {
            if let Some(live) = try_live_studio_http("GET", "/api/state", None) {
                return json!({
                    "source": "live_abu_clock_studio_http_7878",
                    "state": live
                });
            }
            ctrl.background_servo_tick();
            json!({
                "source": "embedded_abu_clock_engine",
                "state": ctrl.snapshot()
            })
        }
        "abu_clock_configure" => {
            let cur = ctrl.snapshot();
            let role = match args.get("role").and_then(|v| v.as_str()) {
                Some("Master") => OperatingRole::Master,
                Some("Follower") => OperatingRole::Follower,
                Some("AutoBmca") => OperatingRole::AutoBmca,
                _ => cur.role,
            };
            let profile = match args.get("profile").and_then(|v| v.as_str()) {
                Some("SmpteSt2059_2") => GuiTimingProfile::SmpteSt2059_2,
                Some("Ieee802_1AsGptp") => GuiTimingProfile::Ieee802_1AsGptp,
                Some("Aes67Harmonized") => GuiTimingProfile::Aes67Harmonized,
                Some("AbuSyncCsptp") => GuiTimingProfile::AbuSyncCsptp,
                _ => cur.profile,
            };
            let req = ConfigUpdateRequest {
                role,
                profile,
                ptp_domain: args
                    .get("ptp_domain")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u8)
                    .unwrap_or(cur.ptp_domain),
                priority1: cur.priority1,
                priority2: cur.priority2,
                fps_num: args
                    .get("fps_num")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .unwrap_or(cur.frame_rate_num),
                fps_den: args
                    .get("fps_den")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .unwrap_or(cur.frame_rate_den),
                drop_frame: args
                    .get("drop_frame")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(cur.drop_frame),
                user_offset_ns: args
                    .get("user_offset_ns")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(cur.user_offset_ns),
                trigger_lead_time_ms: args
                    .get("trigger_lead_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(cur.trigger_lead_time_ms),
                is_wireless: args
                    .get("is_wireless")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(cur.convergence.is_wireless),
                keep_screen_on_active: true,
                auto_sync_enabled: true,
            };
            ctrl.apply_config(req);
            let snap = ctrl.snapshot();
            // Also forward to live running studio if active
            if let Ok(body_str) = serde_json::to_string(&json!({
                "role": snap.role,
                "profile": snap.profile,
                "ptp_domain": snap.ptp_domain,
                "priority1": snap.priority1,
                "priority2": snap.priority2,
                "fps_num": snap.frame_rate_num,
                "fps_den": snap.frame_rate_den,
                "drop_frame": snap.drop_frame,
                "user_offset_ns": snap.user_offset_ns,
                "trigger_lead_time_ms": snap.trigger_lead_time_ms,
                "is_wireless": snap.convergence.is_wireless,
                "keep_screen_on_active": true,
                "auto_sync_enabled": true
            })) {
                let _ = try_live_studio_http("POST", "/api/config", Some(&body_str));
            }
            json!({
                "status": "configured",
                "slate_timecode": snap.slate_timecode,
                "bcd_hex": snap.bcd_hex,
                "frame_rate_hz": snap.frame_rate_hz,
                "drop_frame": snap.drop_frame,
                "ptp_domain": snap.ptp_domain
            })
        }
        "abu_clock_synthesize_timecode" => {
            let tai_ns = args
                .get("tai_ns")
                .and_then(|v| v.as_u64())
                .unwrap_or_else(|| ctrl.clock.now_tai_ns());
            let fps_num = args
                .get("fps_num")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(ctrl.engine.frame_rate.numerator);
            let fps_den = args
                .get("fps_den")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(ctrl.engine.frame_rate.denominator);
            let drop_frame = args
                .get("drop_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(ctrl.engine.frame_rate.drop_frame);
            let user_offset_ns = args
                .get("user_offset_ns")
                .and_then(|v| v.as_i64())
                .unwrap_or(ctrl.engine.user_offset_ns);

            let rate = FrameRate::new(fps_num.max(1), fps_den.max(1), drop_frame);
            let eng = SmpteTimecodeEngine::new(rate, user_offset_ns);
            let synth = eng.synthesize_at_tai_ns(tai_ns);
            let tlv = SmpteSyncMetadataTlv::from_frame_rate(rate);
            let tlv_hex = tlv
                .to_bytes()
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            json!({
                "tai_ns": synth.tai_ns,
                "smpte_timecode": synth.timecode.to_slate_string(),
                "compact_filename_token": synth.timecode.to_compact_filename_token(),
                "bcd_u32": synth.timecode.to_bcd_u32(),
                "bcd_hex": format!("0x{:08X}", synth.timecode.to_bcd_u32()),
                "frame_index": synth.frame_index,
                "subframe_phase": synth.phase,
                "flash_marker_frame_00": synth.timecode.frames == 0 && synth.phase < 0.25,
                "smpte_st2059_2_tlv_20b_hex": tlv_hex
            })
        }
        "abu_clock_schedule_capture" => {
            let mode = args
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("single");
            let frames = args
                .get("frames")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(5);
            let cmd = if mode == "burst" {
                ctrl.trigger_interval_burst(frames)
            } else {
                ctrl.trigger_scheduled_capture()
            };
            if let Ok(body_str) = serde_json::to_string(&json!({ "mode": mode, "frames": frames }))
            {
                let _ = try_live_studio_http("POST", "/api/trigger", Some(&body_str));
            }
            let snap = ctrl.snapshot();
            json!({
                "sequence_id": cmd.sequence_id,
                "target_tai_ns": cmd.target_tai_ns,
                "smpte_timecode": cmd.smpte_tc.to_slate_string(),
                "bcd_hex": format!("0x{:08X}", cmd.smpte_tc.to_bcd_u32()),
                "latest_receipts": snap.capture_logs.first(),
                "manifest_abuscan_json": snap.manifest_abuscan_json,
                "manifest_colmap_txt": snap.manifest_colmap_txt
            })
        }
        "abu_clock_pack_telemetry" => {
            let pose = CameraPose6DoF {
                camera_id: args
                    .get("camera_id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u8)
                    .unwrap_or(1),
                pan_deg: args.get("pan_deg").and_then(|v| v.as_f64()).unwrap_or(12.5),
                tilt_deg: args
                    .get("tilt_deg")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(-4.25),
                roll_deg: args
                    .get("roll_deg")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.15),
                x_mm: args.get("x_mm").and_then(|v| v.as_f64()).unwrap_or(1250.0),
                y_mm: args.get("y_mm").and_then(|v| v.as_f64()).unwrap_or(-400.0),
                z_mm: args.get("z_mm").and_then(|v| v.as_f64()).unwrap_or(1700.0),
                zoom_raw: 2048,
                focus_raw: 4096,
            };
            let focal_mm = args
                .get("focal_length_mm")
                .and_then(|v| v.as_f64())
                .unwrap_or(24.0);
            let k1 = args.get("k1").and_then(|v| v.as_f64()).unwrap_or(-0.042);
            let k2 = args.get("k2").and_then(|v| v.as_f64()).unwrap_or(0.011);

            ctrl.update_camera_pose(pose);
            let tai_ns = ctrl.clock.now_tai_ns();
            let synth = ctrl.engine.synthesize_at_tai_ns(tai_ns);
            let freed = FreeDPacket::new(pose, synth.timecode, tai_ns);
            let d1_hex = freed
                .to_standard_d1_bytes()
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let d37_hex = freed
                .to_abutrack_37b_bytes()
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let d2_hex = freed
                .to_extended_d2_bytes()
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let livelink = AbuTrackLiveLinkPacket::from_pose_and_clock(
                "AbuTrack",
                pose,
                focal_mm,
                k1,
                k2,
                synth.timecode,
                tai_ns,
                "Locked",
            );
            let otrk_payload = OpenTrackIoPayload::new(
                ctrl.telemetry_seq,
                tai_ns,
                synth.timecode,
                ctrl.engine.frame_rate,
                pose,
            );
            let otrk_pkt = OpenTrackIoPacket {
                encoding: OpenTrackIoEncoding::Json,
                sequence_num: ctrl.telemetry_seq,
                offset: 0,
                payload: otrk_payload.clone(),
            };

            json!({
                "freed_29b_standard_d1_hex": d1_hex,
                "freed_37b_abutrack_extended_hex": d37_hex,
                "freed_41b_abuclock_tc_d2_hex": d2_hex,
                "abutrack_livelink_json_udp_8080": livelink,
                "opentrackio_payload": otrk_payload,
                "opentrackio_total_wire_bytes": otrk_pkt.to_bytes().len()
            })
        }
        "abu_clock_discover_nodes" => {
            if let Some(node_id) = args.get("register_node_id").and_then(|v| v.as_str()) {
                let kind = match args.get("register_kind").and_then(|v| v.as_str()) {
                    Some("AbuTrackIos") => AbuEcosystemNodeKind::AbuTrackIos,
                    Some("AbuTrackDesktop") => AbuEcosystemNodeKind::AbuTrackDesktop,
                    Some("AbuTrackUnreal") => AbuEcosystemNodeKind::AbuTrackUnreal,
                    Some("LidarSensor") => AbuEcosystemNodeKind::LidarSensor,
                    Some("CameraRig") => AbuEcosystemNodeKind::CameraRig,
                    _ => AbuEcosystemNodeKind::AbuClockMaster,
                };
                let info = AbuEcosystemNodeInfo {
                    node_id: node_id.to_string(),
                    node_kind: kind,
                    display_name: args
                        .get("register_display_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(node_id)
                        .to_string(),
                    platform: "Registered via Abu Clock MCP".to_string(),
                    ip_address: args
                        .get("register_ip")
                        .and_then(|v| v.as_str())
                        .unwrap_or("127.0.0.1")
                        .to_string(),
                    telemetry_port: 8080,
                    video_rtp_port: 5004,
                    clock_port: 50319,
                    control_http_port: 7878,
                    protocols: vec![
                        "AbuSync-CSPTP (50319)".to_string(),
                        "SNTPv4 & Discovery (50320)".to_string(),
                        "UDP 8080 Telemetry".to_string(),
                    ],
                    lock_state: "Locked".to_string(),
                    offset_ns: ctrl.clock.metrics().offset_ns,
                    frame_rate_str: "60.000 Hz (NDF)".to_string(),
                    last_seen_tai_ns: ctrl.clock.now_tai_ns(),
                };
                ctrl.register_ecosystem_node(info.clone());
                if let Ok(body_str) = serde_json::to_string(&info) {
                    let _ =
                        try_live_studio_http("POST", "/api/ecosystem/register", Some(&body_str));
                }
            }
            if let Some(live_beacon) =
                try_live_studio_http("GET", "/api/ecosystem/discover", None)
            {
                return live_beacon;
            }
            serde_json::to_value(ctrl.discovery_beacon()).unwrap_or_else(|_| json!({}))
        }
        "abu_clock_get_integration_guide" => {
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("all");
            build_abutrack_integration_guide(target)
        }
        _ => json!({
            "error": format!("Unknown tool: {}", tool_name)
        }),
    }
}

fn build_abutrack_integration_guide(target: &str) -> Value {
    let ios_guide = json!({
        "component": "Abu Track & Scan iOS App (Sources/Managers/NTPSyncClient.swift & UDPStreamingManager.swift)",
        "current_bottlenecks": [
            "1. NTPSyncClient.swift currently sends a single one-shot 48-byte SNTP packet to privileged UDP port 123 (blocked on Windows without admin) and uses Date().timeIntervalSince1970 (which jumps on wall-clock steps and ignores crystal frequency drift alpha_freq).",
            "2. UDPStreamingManager.swift stamps frames with local seconds without embedding SMPTE ST 12-1 timecode (HH:MM:SS;FF.sf) or sub-frame phase."
        ],
        "recommended_upgrades": [
            "1. Point NTPSyncClient.swift to unprivileged UDP port 50320 on the Abu Clock PC (or send 'ABU_ECO_V1' to port 50320 to auto-discover the PC IP & active frame rate without typing IPs!).",
            "2. Anchor ARFrame.timestamp (mach_continuous_time) to AbuClock's 2-state Kalman PLL (integrations/abu-track-scan/ios/AbuClockKit.swift) so 60 Hz poses have <100us jitter and zero crystal drift.",
            "3. Listen for SCHEDULE_CAPTURE_AT barrier triggers so multi-iPhone / DSLR rigs fire within <50us."
        ],
        "swift_drop_in_snippet": r#"// In Sources/Managers/NTPSyncClient.swift:
// 1. Auto-discover Abu Clock Master & PC Workstation IP on UDP port 50320 (Zero-Config!)
func discoverAbuClockMaster(completion: @escaping (String?, Int?) -> Void) {
    // Send "ABU_ECO_V1" to 255.255.255.255:50320 or query http://<host>:7878/api/ecosystem/discover
    // Response JSON contains master_ip, csptp_event_port (50319), sntp_general_port (50320),
    // abutrack_telemetry_port (8080), fps_num, fps_den, drop_frame!
}
// 2. Use port 50320 instead of privileged port 123 for 48-byte SNTPv4 sync:
let port = NWEndpoint.Port(rawValue: 50320)! // Handled by AbuClock SntpResponder (Stratum 1, 'ABUC')"#
    });

    let win_guide = json!({
        "component": "Abu Track & Scan Windows Workstation (DesktopSync/camtrack_desktop_gui.py & sync_photos.py)",
        "current_bottlenecks": [
            "1. camtrack_desktop_gui.py receives UDP 8080 packets and records wall-clock arrival or uncalibrated float timestamps.",
            "2. sync_photos.py matches DSLR EXIF timestamps against 60 Hz poses using 1-second EXIF granularity, which drifts if the DSLR internal RTC is off by even 50ms."
        ],
        "recommended_upgrades": [
            "1. On startup of camtrack_desktop_gui.py, POST to http://127.0.0.1:7878/api/ecosystem/register so the iOS app and UE5 automatically discover the Windows Workstation IP and ports.",
            "2. Load target/release/abuclock.dll via integrations/python/abu_clock.py (or query http://127.0.0.1:7878/api/state) to stamp every incoming 37-byte FreeD / LiveLink JSON frame with exact TAI ns + SMPTE BCD timecode and export .abuscan.json + COLMAP images.txt directly.",
            "3. In sync_photos.py, photograph the Abu Clock 'FRAME 00 FLASH' optical beacon once at the start of the shoot to lock the DSLR's sub-frame shutter offset to <1ms!"
        ],
        "python_drop_in_snippet": r#"# In DesktopSync/camtrack_desktop_gui.py:
import urllib.request, json

def register_with_abu_clock():
    node = {
        "node_id": "abutrack-win-workstation",
        "node_kind": "AbuTrackDesktop",
        "display_name": "Abu Track & Scan PC Workstation",
        "platform": "Windows Python/CUDA",
        "ip_address": "127.0.0.1",
        "telemetry_port": 8080,
        "video_rtp_port": 5004,
        "clock_port": 50319,
        "control_http_port": 7878,
        "protocols": ["UDP 8080 Receiver", "COLMAP/3DGS", "DSLR Sync"],
        "lock_state": "Locked",
        "offset_ns": 0,
        "frame_rate_str": "60.000 Hz (NDF)",
        "last_seen_tai_ns": 0
    }
    req = urllib.request.Request(
        "http://127.0.0.1:7878/api/ecosystem/register",
        data=json.dumps(node).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=0.5) as resp:
        return json.loads(resp.read().decode("utf-8"))"#
    });

    let ue5_guide = json!({
        "component": "Abu Track & Scan Unreal Engine 5 Plugin (UnrealPlugin/LiveLinkAbuTrack/Source/LiveLinkAbuTrack/Private/LiveLinkAbuTrackSource.cpp)",
        "current_bottlenecks": [
            "1. LiveLinkAbuTrackSource.cpp line 131 sets CameraFrameData.WorldTime = FLiveLinkWorldTime(FPlatformTime::Seconds()) on UDP packet arrival, which bakes variable Wi-Fi packet jitter directly into Unreal Engine's camera interpolation!",
            "2. FrameData.MetaData.SceneTime is not populated with SMPTE FQualifiedFrameTime, preventing Take Recorder and nDisplay genlock frame-matching."
        ],
        "recommended_upgrades": [
            "1. Read 'smpteTimecode' / 'bcdTimecode' / 'subframePhase' from the enriched AbuTrack LiveLink JSON (or 37B/41B FreeD packet) or call abu_clock_synthesize_at_tai_ns() from abuclock.dll.",
            "2. Populate CameraFrameData.MetaData.SceneTime = FQualifiedFrameTime(FTimecode(Hours, Minutes, Seconds, Frames, bDropFrame), FFrameRate(FpsNum, FpsDen)) so UE5 Take Recorder & Composure lock to exact capture frame & sub-frame phase!"
        ],
        "cpp_drop_in_snippet": r#"// In LiveLinkAbuTrackSource.cpp (ParseAndPushJSON / ParseAndPushFreeD):
#include "Misc/Timecode.h"
#include "Misc/QualifiedFrameTime.h"

// Populate exact SMPTE ST 12-1 SceneTime from AbuClock instead of jittery arrival time:
int32 Hours = 0, Minutes = 0, Seconds = 0, Frames = 0;
double Subframe = 0.0;
if (JsonObject->TryGetNumberField(TEXT("bcdTimecode"), BcdVal)) {
    Hours   = ((BcdVal >> 28) & 0xF) * 10 + ((BcdVal >> 24) & 0xF);
    Minutes = ((BcdVal >> 20) & 0xF) * 10 + ((BcdVal >> 16) & 0xF);
    Seconds = ((BcdVal >> 12) & 0xF) * 10 + ((BcdVal >> 8)  & 0xF);
    Frames  = ((BcdVal >> 4)  & 0xF) * 10 + (BcdVal & 0xF);
}
JsonObject->TryGetNumberField(TEXT("subframePhase"), Subframe);
const FTimecode Tc(Hours, Minutes, Seconds, Frames, false);
const FFrameTime FrameTime(Tc.ToFrameNumber(FFrameRate(60, 1)), static_cast<float>(Subframe));
CameraFrameData.MetaData.SceneTime = FQualifiedFrameTime(FrameTime, FFrameRate(60, 1));"#
    });

    match target {
        "ios_swift" => ios_guide,
        "windows_python" | "dslr_sync" => win_guide,
        "unreal_cpp" => ue5_guide,
        _ => json!({
            "ecosystem_overview": "Abu Clock provides the unified sub-microsecond timing backbone, UDP 50320 Auto-Discovery + Stratum-1 SNTPv4 responder, 29B/37B/41B FreeD + OpenTrackIO + LiveLink JSON telemetry synthesizer, and .abuscan.json / COLMAP manifest generator for all Abu Track & Scan apps.",
            "documentation_files": [
                "c:/Users/USer/Documents/antigravity/abu-clock/docs/ABU_TRACK_AND_SCAN_INTEGRATION_GUIDE.md",
                "c:/Users/USer/Documents/antigravity/abu-clock/docs/MCP_AND_API_GUIDE.md",
                "c:/Users/USer/Documents/antigravity/abu-clock/docs/DEVELOPER_GUIDE.md"
            ],
            "ios_swift": ios_guide,
            "windows_python": win_guide,
            "unreal_cpp": ue5_guide
        }),
    }
}
