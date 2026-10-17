//! Local HTTP + JSON RPC server (`127.0.0.1:7878`) and UDP Ecosystem Discovery / SNTPv4 responder (`50320`)
//! for the **Abu Clock** PC Application.
//!
//! Serves both the embedded Unified Control Center UI (`GET /`) and real-time JSON endpoints
//! (`GET /api/state`, `GET /api/ecosystem/discover`, `POST /api/ecosystem/register`, `GET /api/sntp`,
//! `POST /api/config`, `POST /api/clock`, `POST /api/bmca`, `POST /api/trigger`,
//! `POST /api/telemetry`, `POST /api/manifest`, `POST /api/appliance`, `POST /api/shutdown`) with CORS headers.

use std::io::{Read, Write};
use std::net::{TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use abu_network::{
    AbuEcosystemNodeInfo, CameraPose6DoF, SntpResponder, ABUSYNC_GENERAL_PORT_UNPRIVILEGED,
    ABU_ECOSYSTEM_BEACON_MAGIC,
};
use serde::Deserialize;

use crate::{ui_html::INDEX_HTML, ConfigUpdateRequest, GuiAppController};

#[derive(Deserialize)]
struct ClockActionRequest {
    action: String,
}

#[derive(Deserialize)]
struct BmcaTestRequest {
    remote_priority1: u8,
    remote_clock_class: u8,
    remote_priority2: u8,
    remote_fps_num: u32,
    remote_fps_den: u32,
    remote_drop_frame: bool,
}

#[derive(Deserialize)]
struct TriggerActionRequest {
    mode: String,
    frames: Option<u32>,
}

#[derive(Deserialize)]
struct ManifestAddRequest {
    sensor_id: String,
    ext: String,
}

#[derive(Deserialize)]
struct ApplianceActionRequest {
    gnss_locked: bool,
}

/// Starts the Abu Clock local HTTP control server on `addr` (`127.0.0.1:7878`), spawns the
/// continuous 8 Hz (`125 ms`) background Kalman servo ticker thread, and spawns the unprivileged
/// UDP `50320` responder for `NTPSyncClient.swift` (48-byte SNTPv4) and `ABU_ECO_V1` LAN discovery.
pub fn run_http_server(
    addr: &str,
    controller: Arc<Mutex<GuiAppController>>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    // Spawn 8 Hz (125 ms) live Kalman servo & loopback UDP telemetry ticker
    let bg_ctrl = controller.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(125));
        if let Ok(mut guard) = bg_ctrl.lock() {
            guard.background_servo_tick();
        }
    });

    // Spawn unprivileged UDP 50320 responder for iOS NTPSyncClient.swift (48B SNTP) & ABU_ECO_V1 discovery
    let udp_ctrl = controller.clone();
    std::thread::spawn(move || {
        let bind_addr = format!("0.0.0.0:{}", ABUSYNC_GENERAL_PORT_UNPRIVILEGED);
        if let Ok(sock) = UdpSocket::bind(&bind_addr) {
            let mut buf = [0u8; 2048];
            while let Ok((n, peer)) = sock.recv_from(&mut buf) {
                if let Ok(guard) = udp_ctrl.lock() {
                    let rx_tai = guard.clock.now_tai_ns();
                    if n >= 48 && (buf[0] == 0x1B || buf[0] == 0x23) {
                        // RFC 4330 48-byte SNTPv3/v4 request from iOS NTPSyncClient.swift
                        let tx_tai = guard.clock.now_tai_ns();
                        if let Ok(resp) = SntpResponder::build_response(&buf[..48], rx_tai, tx_tai) {
                            let _ = sock.send_to(&resp, peer);
                        }
                    } else if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                        if text.starts_with(ABU_ECOSYSTEM_BEACON_MAGIC) || text.starts_with("DISCOVER_ABU") {
                            let beacon = guard.discovery_beacon();
                            if let Ok(json) = serde_json::to_vec(&beacon) {
                                let _ = sock.send_to(&json, peer);
                            }
                        }
                    }
                }
            }
        }
    });

    for stream in listener.incoming().flatten() {
        let ctrl = controller.clone();
        std::thread::spawn(move || {
            let _ = handle_connection(stream, ctrl);
        });
    }
    Ok(())
}

fn handle_connection(
    mut stream: std::net::TcpStream,
    controller: Arc<Mutex<GuiAppController>>,
) -> std::io::Result<()> {
    let mut buf = [0u8; 32768];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let raw = String::from_utf8_lossy(&buf[..n]);
    let mut lines = raw.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "POST" && path == "/api/shutdown" {
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
        let _ = stream.write_all(resp.as_bytes());
        let _ = stream.flush();
        std::process::exit(0);
    }

    let body = raw
        .split("\r\n\r\n")
        .nth(1)
        .or_else(|| raw.split("\n\n").nth(1))
        .unwrap_or("");

    if method == "GET" && (path == "/" || path.starts_with("/index.html")) {
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-cache\r\n\r\n{}",
            INDEX_HTML.len(),
            INDEX_HTML
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    let mut guard = match controller.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    if method == "GET" && path == "/api/ecosystem/discover" {
        let beacon = guard.discovery_beacon();
        drop(guard);
        let json = serde_json::to_string(&beacon).unwrap_or_else(|_| "{}".to_string());
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nContent-Length: {}\r\n\r\n{}",
            json.len(),
            json
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if (method == "GET" || method == "POST") && path == "/api/sntp" {
        let rx_tai = guard.clock.now_tai_ns();
        let synth = guard.engine.synthesize_at_tai_ns(rx_tai);
        let mut req48 = [0u8; 48];
        req48[0] = 0x1B;
        let tx_tai = guard.clock.now_tai_ns();
        let sntp_bytes = SntpResponder::build_response(&req48, rx_tai, tx_tai).unwrap_or([0u8; 48]);
        let sntp_hex = sntp_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        let payload = serde_json::json!({
            "stratum": 1,
            "ref_id": "ABUC",
            "rx_tai_ns": rx_tai,
            "tx_tai_ns": tx_tai,
            "smpte_timecode": synth.timecode.to_slate_string(),
            "bcd_hex": format!("0x{:08X}", synth.timecode.to_bcd_u32()),
            "subframe_phase": synth.phase,
            "sntp_48b_hex": sntp_hex,
        });
        drop(guard);
        let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nContent-Length: {}\r\n\r\n{}",
            json.len(),
            json
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    match (method, path) {
        ("GET", "/api/state") => {}
        ("POST", "/api/ecosystem/register") => {
            if let Ok(node) = serde_json::from_str::<AbuEcosystemNodeInfo>(body) {
                guard.register_ecosystem_node(node);
            }
        }
        ("POST", "/api/config") => {
            if let Ok(req) = serde_json::from_str::<ConfigUpdateRequest>(body) {
                guard.apply_config(req);
            }
        }
        ("POST", "/api/clock") => {
            if let Ok(req) = serde_json::from_str::<ClockActionRequest>(body) {
                match req.action.as_str() {
                    "tick_clean" => {
                        let jitter = ((guard.clock.now_tai_ns() % 19) as i64) - 9;
                        guard.inject_network_sample(jitter, 360);
                    }
                    "tick_wifi" => {
                        guard.is_wireless = true;
                        let spike = 1_850i64;
                        guard.inject_network_sample(spike, 4_200);
                    }
                    "step_50us" => {
                        guard.inject_network_sample(50_000, 410);
                    }
                    "fast_forward_7s" => {
                        guard.fast_forward_warmup();
                    }
                    "holdover" => {
                        guard.set_simulated_lock_override(3_500, 3_000_000_000);
                    }
                    "unlock" => {
                        guard.set_simulated_lock_override(1_500_000, 1_000_000_000);
                    }
                    "toggle_autosync" => {
                        guard.auto_sync_enabled = !guard.auto_sync_enabled;
                    }
                    _ => {}
                }
            }
        }
        ("POST", "/api/bmca") => {
            if let Ok(req) = serde_json::from_str::<BmcaTestRequest>(body) {
                guard.test_bmca_election(
                    req.remote_priority1,
                    req.remote_clock_class,
                    req.remote_priority2,
                    req.remote_fps_num,
                    req.remote_fps_den,
                    req.remote_drop_frame,
                );
            }
        }
        ("POST", "/api/trigger") => {
            if let Ok(req) = serde_json::from_str::<TriggerActionRequest>(body) {
                if req.mode == "burst" {
                    guard.trigger_interval_burst(req.frames.unwrap_or(5));
                } else {
                    guard.trigger_scheduled_capture();
                }
            }
        }
        ("POST", "/api/telemetry") => {
            if let Ok(pose) = serde_json::from_str::<CameraPose6DoF>(body) {
                guard.update_camera_pose(pose);
            }
        }
        ("POST", "/api/manifest") => {
            if let Ok(req) = serde_json::from_str::<ManifestAddRequest>(body) {
                guard.add_manifest_record(&req.sensor_id, &req.ext);
            }
        }
        ("POST", "/api/appliance") => {
            if let Ok(req) = serde_json::from_str::<ApplianceActionRequest>(body) {
                guard.set_appliance_gnss_lock(req.gnss_locked);
            }
        }
        _ => {
            let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
            stream.write_all(not_found.as_bytes())?;
            return Ok(());
        }
    }

    let snap = guard.snapshot();
    drop(guard);

    let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string());
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    );
    stream.write_all(resp.as_bytes())?;
    Ok(())
}
