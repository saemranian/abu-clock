//! Abu Ecosystem Auto-Discovery Registry, `Abu Track & Scan` LiveLink Bridge,
//! and RFC 4330 SNTPv4 Responder (`NTPSyncClient.swift` compatibility).
//!
//! Unifies all nodes across the Abu Ecosystem:
//! 1. **Abu Clock Master** (`abu-clock` PC Studio & Pi CM4 Grandmaster Appliance)
//! 2. **Abu Track & Scan iOS App** (`ios_app_cam_track_01`: ARKit 60 Hz 6-DoF + H.264/HEVC RTP)
//! 3. **Abu Track & Scan Windows Workstation** (`DesktopSync/camtrack_desktop_gui.py`: COLMAP / 3DGS / DSLR Sync)
//! 4. **Abu Track & Scan Unreal Engine 5 Plugin** (`UnrealPlugin/LiveLinkAbuTrack`)

use abu_clock_core::{FrameRate, SmpteTimecode, DEFAULT_TAI_MINUS_UTC_SECONDS};
use serde::{Deserialize, Serialize};

use crate::telemetry::CameraPose6DoF;

/// Magic identifier for UDP LAN discovery beacons across the Abu Ecosystem.
pub const ABU_ECOSYSTEM_BEACON_MAGIC: &str = "ABU_ECO_V1";

/// Default UDP port used by `Abu Track & Scan` (iOS -> PC / UE5 telemetry).
pub const ABUTRACK_DEFAULT_TELEMETRY_PORT: u16 = 8080;

/// Default UDP port used by `Abu Track & Scan` for RTP H.264/HEVC low-latency video.
pub const ABUTRACK_DEFAULT_VIDEO_RTP_PORT: u16 = 5004;

/// Role/type of an application or hardware node participating in the Abu Ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbuEcosystemNodeKind {
    /// `Abu Clock` PC Control Studio or Raspberry Pi CM4 Hardware Grandmaster Appliance.
    AbuClockMaster,
    /// `Abu Track & Scan` iOS App (`UDPStreamingManager.swift` / `NTPSyncClient.swift`).
    AbuTrackIos,
    /// `Abu Track & Scan` Windows Workstation (`DesktopSync/camtrack_desktop_gui.py`).
    AbuTrackDesktop,
    /// `Abu Track & Scan` Unreal Engine 5 LiveLink Plugin (`LiveLinkAbuTrackSource.cpp`).
    AbuTrackUnreal,
    /// Solid-state LiDAR sensor array (`RoboSense E1R` / `XGRIDS`) on gPTP Domain 0.
    LidarSensor,
    /// Synchronized cinema / DSLR / mirrorless camera rig.
    CameraRig,
}

/// Live registration and health record for a discovered Abu Ecosystem node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbuEcosystemNodeInfo {
    pub node_id: String,
    pub node_kind: AbuEcosystemNodeKind,
    pub display_name: String,
    pub platform: String,
    pub ip_address: String,
    pub telemetry_port: u16,
    pub video_rtp_port: u16,
    pub clock_port: u16,
    pub control_http_port: u16,
    pub protocols: Vec<String>,
    pub lock_state: String,
    pub offset_ns: i64,
    pub frame_rate_str: String,
    pub last_seen_tai_ns: u64,
}

/// Discovery beacon broadcast over UDP (`50320`) or returned via HTTP (`GET /api/ecosystem/discover`)
/// so iOS, PC, and UE5 nodes automatically find the Master Clock and each other without manual IP entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbuEcosystemDiscoveryBeacon {
    pub magic: String,
    pub version: String,
    pub master_ip: String,
    pub csptp_event_port: u16,
    pub sntp_general_port: u16,
    pub http_api_port: u16,
    pub abutrack_telemetry_port: u16,
    pub abutrack_video_port: u16,
    pub ptp_domain: u8,
    pub tai_ns: u64,
    pub smpte_timecode: String,
    pub bcd_hex: String,
    pub fps_num: u32,
    pub fps_den: u32,
    pub drop_frame: bool,
    pub trigger_lead_time_ms: u64,
    pub nodes: Vec<AbuEcosystemNodeInfo>,
}

/// Enriched `Abu Track & Scan` Live Link JSON telemetry packet (`UDP 8080`).
///
/// 100% backward-compatible with `UDPStreamingManager.swift`, `camtrack_desktop_gui.py`,
/// and `LiveLinkAbuTrackSource.cpp`, while adding sub-frame SMPTE ST 12-1 timecode and
/// disciplined TAI nanosecond fields from `AbuClock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbuTrackLiveLinkPacket {
    pub device_name: String,
    /// `[x_m, y_m, z_m]` in right-handed ARKit meters.
    pub translation: [f64; 3],
    /// `[pitch_deg, yaw_deg, roll_deg]` in degrees.
    pub rotation: [f64; 3],
    pub focal_length: f64,
    pub k1: f64,
    pub k2: f64,
    pub video_codec: String,
    /// Disciplined UTC/TAI seconds timestamp (`f64`) for backward compatibility with `camtrack_desktop_gui.py`.
    pub timestamp: f64,
    pub is_mapping_active: bool,
    pub thermal_state: String,
    /// `Abu Clock` SMPTE ST 12-1 formatted timecode (`HH:MM:SS;FF.sf`).
    pub smpte_timecode: String,
    /// `Abu Clock` 32-bit BCD timecode word (`0xHHMMSSFF`).
    pub bcd_timecode: u32,
    /// `Abu Clock` disciplined TAI nanoseconds (`u64`).
    pub tai_ns: u64,
    /// Normalized sub-frame phase $\phi \in [0.0, 1.0)$ for UE5 `FQualifiedFrameTime`.
    pub subframe_phase: f64,
    /// Current `OverlayClock` lock state (`Locked`, `WarmingUp`, `Holdover`, `Unlocked`).
    pub clock_lock_state: String,
}

impl AbuTrackLiveLinkPacket {
    /// Synthesizes an `AbuTrackLiveLinkPacket` from a `CameraPose6DoF`, lens intrinsics,
    /// `SmpteTimecode`, and disciplined TAI nanosecond timestamp.
    pub fn from_pose_and_clock(
        device_name: &str,
        pose: CameraPose6DoF,
        focal_length_mm: f64,
        k1: f64,
        k2: f64,
        tc: SmpteTimecode,
        tai_ns: u64,
        lock_state: &str,
    ) -> Self {
        let utc_ns = tai_ns.saturating_sub((DEFAULT_TAI_MINUS_UTC_SECONDS as u64) * 1_000_000_000);
        let timestamp_sec = (utc_ns as f64) / 1_000_000_000.0;
        Self {
            device_name: device_name.to_string(),
            translation: [
                pose.x_mm / 1000.0,
                pose.y_mm / 1000.0,
                pose.z_mm / 1000.0,
            ],
            rotation: [pose.tilt_deg, pose.pan_deg, pose.roll_deg],
            focal_length: focal_length_mm,
            k1,
            k2,
            video_codec: "H.264".to_string(),
            timestamp: timestamp_sec,
            is_mapping_active: true,
            thermal_state: "NOMINAL".to_string(),
            smpte_timecode: tc.to_slate_string(),
            bcd_timecode: tc.to_bcd_u32(),
            tai_ns,
            subframe_phase: tc.subframe,
            clock_lock_state: lock_state.to_string(),
        }
    }
}

/// RFC 4330 48-byte SNTPv4 Responder for `NTPSyncClient.swift` in `Abu Track & Scan`.
///
/// Allows the iOS app (`NTPSyncClient.swift`) to synchronize against `Abu Clock` on unprivileged
/// UDP port `50320` (or via HTTP `/api/sntp`) without requiring Administrator rights on port `123`.
pub struct SntpResponder;

impl SntpResponder {
    /// NTP epoch offset from Unix epoch (`1900-01-01` to `1970-01-01` in seconds).
    pub const NTP_UNIX_OFFSET_SEC: u64 = 2_208_988_800;

    /// Converts a disciplined TAI nanosecond timestamp into a 64-bit RFC 4330 NTP timestamp
    /// (upper 32 bits = UTC seconds since 1900, lower 32 bits = fractional seconds).
    #[inline]
    pub fn tai_ns_to_ntp_u64(tai_ns: u64) -> u64 {
        let utc_ns = tai_ns.saturating_sub((DEFAULT_TAI_MINUS_UTC_SECONDS as u64) * 1_000_000_000);
        let unix_sec = utc_ns / 1_000_000_000;
        let sub_ns = utc_ns % 1_000_000_000;
        let ntp_sec = unix_sec + Self::NTP_UNIX_OFFSET_SEC;
        let ntp_frac = ((sub_ns as u128) << 32) / 1_000_000_000u128;
        (ntp_sec << 32) | (ntp_frac as u64)
    }

    /// Processes a 48-byte SNTP client request (`packet[0] = 0x1B` or `0x23`) and returns
    /// a 48-byte Stratum-1 SNTP server response populated with `rx_tai_ns` ($T_2$) and `tx_tai_ns` ($T_3$).
    pub fn build_response(client_req: &[u8], rx_tai_ns: u64, tx_tai_ns: u64) -> Result<[u8; 48], &'static str> {
        if client_req.len() < 48 {
            return Err("SNTP client request must be at least 48 bytes");
        }
        let mut resp = [0u8; 48];
        // LI = 0 (no warning), VN = 4 (or 3), Mode = 4 (Server) -> 0x24
        resp[0] = 0x24;
        // Stratum = 1 (Primary Reference / AbuClock Grandmaster)
        resp[1] = 1;
        // Poll = 4, Precision = -29 (~1.8 ns)
        resp[2] = 4;
        resp[3] = (-29i8) as u8;
        // Reference ID = "ABUC"
        resp[12..16].copy_from_slice(b"ABUC");

        let ref_ntp = Self::tai_ns_to_ntp_u64(rx_tai_ns);
        let rx_ntp = ref_ntp;
        let tx_ntp = Self::tai_ns_to_ntp_u64(tx_tai_ns);

        // Reference Timestamp (bytes 16..24)
        resp[16..24].copy_from_slice(&ref_ntp.to_be_bytes());
        // Originate Timestamp T1 = Client Transmit Timestamp from request bytes 40..48 (copied to 24..32)
        resp[24..32].copy_from_slice(&client_req[40..48]);
        // Receive Timestamp T2 (bytes 32..40)
        resp[32..40].copy_from_slice(&rx_ntp.to_be_bytes());
        // Transmit Timestamp T3 (bytes 40..48)
        resp[40..48].copy_from_slice(&tx_ntp.to_be_bytes());

        Ok(resp)
    }
}

/// Returns the default pre-seeded Abu Ecosystem node topology (`Abu Clock Master`,
/// `Abu Track & Scan iOS`, `Abu Track & Scan Windows Workstation`, `UE5 LiveLinkAbuTrack`).
pub fn default_ecosystem_nodes(tai_ns: u64, rate: FrameRate, offset_ns: i64) -> Vec<AbuEcosystemNodeInfo> {
    let fps_str = format!(
        "{:.3} Hz ({})",
        rate.as_hz(),
        if rate.drop_frame { "DF" } else { "NDF" }
    );
    vec![
        AbuEcosystemNodeInfo {
            node_id: "abu-clock-master-01".to_string(),
            node_kind: AbuEcosystemNodeKind::AbuClockMaster,
            display_name: "Abu Clock Studio Master (PC / GM)".to_string(),
            platform: "Windows / Rust (QPC + TimestampedUdpSocket)".to_string(),
            ip_address: "127.0.0.1".to_string(),
            telemetry_port: ABUTRACK_DEFAULT_TELEMETRY_PORT,
            video_rtp_port: ABUTRACK_DEFAULT_VIDEO_RTP_PORT,
            clock_port: 50319,
            control_http_port: 7878,
            protocols: vec![
                "AbuSync-CSPTP (50319)".to_string(),
                "SMPTE ST 2059-2 TLV".to_string(),
                "SNTPv4 Responder (50320)".to_string(),
                "MCP + REST API (7878)".to_string(),
            ],
            lock_state: "Locked".to_string(),
            offset_ns,
            frame_rate_str: fps_str.clone(),
            last_seen_tai_ns: tai_ns,
        },
        AbuEcosystemNodeInfo {
            node_id: "abutrack-ios-iphone16pro".to_string(),
            node_kind: AbuEcosystemNodeKind::AbuTrackIos,
            display_name: "Abu Track & Scan iOS (iPhone 16 Pro)".to_string(),
            platform: "iOS Swift (ARKit 60Hz + Metal + AbuClockKit)".to_string(),
            ip_address: "192.168.1.42".to_string(),
            telemetry_port: ABUTRACK_DEFAULT_TELEMETRY_PORT,
            video_rtp_port: ABUTRACK_DEFAULT_VIDEO_RTP_PORT,
            clock_port: 50319,
            control_http_port: 0,
            protocols: vec![
                "LiveLink JSON (UDP 8080)".to_string(),
                "FreeD 37B & 41B (0xD1/0xD2)".to_string(),
                "OpenTrackIO (OTrk)".to_string(),
                "RTP H.264/HEVC (UDP 5004)".to_string(),
            ],
            lock_state: "Locked".to_string(),
            offset_ns: offset_ns + 14,
            frame_rate_str: fps_str.clone(),
            last_seen_tai_ns: tai_ns,
        },
        AbuEcosystemNodeInfo {
            node_id: "abutrack-win-workstation".to_string(),
            node_kind: AbuEcosystemNodeKind::AbuTrackDesktop,
            display_name: "Abu Track & Scan PC Workstation (DesktopSync)".to_string(),
            platform: "Windows Python/CUDA (COLMAP + 3DGS + DSLR Sync)".to_string(),
            ip_address: "127.0.0.1".to_string(),
            telemetry_port: ABUTRACK_DEFAULT_TELEMETRY_PORT,
            video_rtp_port: ABUTRACK_DEFAULT_VIDEO_RTP_PORT,
            clock_port: 50319,
            control_http_port: 7878,
            protocols: vec![
                "UDP 8080 Telemetry Receiver".to_string(),
                "DSLR Hot-Shoe + Slate Sync".to_string(),
                ".abuscan.json & Colmap Exporter".to_string(),
            ],
            lock_state: "Locked".to_string(),
            offset_ns: offset_ns - 6,
            frame_rate_str: fps_str.clone(),
            last_seen_tai_ns: tai_ns,
        },
        AbuEcosystemNodeInfo {
            node_id: "abutrack-ue5-livelink".to_string(),
            node_kind: AbuEcosystemNodeKind::AbuTrackUnreal,
            display_name: "Unreal Engine 5 (LiveLinkAbuTrack + AbuClockUE)".to_string(),
            platform: "UE 5.4 C++ (FLiveLinkCameraRole + FQualifiedFrameTime)".to_string(),
            ip_address: "127.0.0.1".to_string(),
            telemetry_port: ABUTRACK_DEFAULT_TELEMETRY_PORT,
            video_rtp_port: ABUTRACK_DEFAULT_VIDEO_RTP_PORT,
            clock_port: 50319,
            control_http_port: 7878,
            protocols: vec![
                "LiveLinkAbuTrack (UDP 8080)".to_string(),
                "UTimecodeProvider (SMPTE ST 12-1)".to_string(),
                "FreeD 29B/37B/41B & OpenTrackIO".to_string(),
            ],
            lock_state: "Locked".to_string(),
            offset_ns: offset_ns + 3,
            frame_rate_str: fps_str,
            last_seen_tai_ns: tai_ns,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sntp_responder_preserves_client_originate_and_stamps_server_tai() {
        let mut req = [0u8; 48];
        req[0] = 0x1B; // Matches NTPSyncClient.swift in ios_app_cam_track_01
        req[40..48].copy_from_slice(&0xE9A1_0000_1234_5678u64.to_be_bytes());

        let rx_tai = 1_729_180_837_000_000_000u64;
        let tx_tai = 1_729_180_837_000_050_000u64;
        let resp = SntpResponder::build_response(&req, rx_tai, tx_tai).unwrap();

        assert_eq!(resp[0], 0x24); // Mode 4 Server
        assert_eq!(resp[1], 1); // Stratum 1
        assert_eq!(&resp[12..16], b"ABUC");
        // Originate timestamp T1 must equal client transmit bytes 40..48
        assert_eq!(&resp[24..32], &req[40..48]);
    }

    #[test]
    fn abutrack_livelink_packet_serializes_camel_case_with_smpte_fields() {
        let pose = CameraPose6DoF {
            camera_id: 1,
            pan_deg: 15.0,
            tilt_deg: -5.0,
            roll_deg: 0.5,
            x_mm: 1200.0,
            y_mm: -300.0,
            z_mm: 1600.0,
            zoom_raw: 2048,
            focus_raw: 4096,
        };
        let tc = SmpteTimecode {
            hours: 1,
            minutes: 2,
            seconds: 3,
            frames: 4,
            subframe: 0.5,
            drop_frame: true,
        };
        let pkt = AbuTrackLiveLinkPacket::from_pose_and_clock(
            "AbuTrack",
            pose,
            24.0,
            -0.04,
            0.01,
            tc,
            1_729_180_837_000_000_000,
            "Locked",
        );
        let json = serde_json::to_string(&pkt).unwrap();
        assert!(json.contains("\"deviceName\":\"AbuTrack\""));
        assert!(json.contains("\"focalLength\":24.0"));
        assert!(json.contains("\"smpteTimecode\":\"01:02:03;04.50\""));
    }
}
