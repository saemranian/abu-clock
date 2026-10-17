//! Unified Rust backend controller for the **Abu Clock** PC Application (`abu-clock`).
//!
//! Connects the Unified Control Center UI directly to:
//! - `abu-clock-core` (`OverlayClock`, `SmpteTimecodeEngine`, `FrameRate`, `PlatformMonotonicClock`)
//! - `abu-network` (`DualModePtpEngine`, `TimestampedUdpSocket`, `SmpteSyncMetadataTlv`, `AbuScanCoordinator`, `OpenTrackIoPacket`, `FreeDPacket`)
//! - `abu-ffi` / `abuclock` (`AbuScanSessionManifest`, `AbuScanCaptureRecord`)

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

use abu_clock_core::{
    ClockConvergenceMetrics, FrameRate, LockState, OverlayClock, PlatformMonotonicClock,
    SharedClock, SmpteTimecodeEngine, ABU_CLOCK_VERSION, DEFAULT_TAI_MINUS_UTC_SECONDS,
};
use abu_network::{
    default_ecosystem_nodes, freed_checksum, AbuEcosystemDiscoveryBeacon, AbuEcosystemNodeInfo,
    AbuScanCoordinator, AbuTrackLiveLinkPacket, BmcaDataset, CameraPose6DoF, DualModePtpEngine,
    FreeDPacket, OpenTrackIoEncoding, OpenTrackIoPacket, OpenTrackIoPayload, PtpPortState,
    ScheduleCaptureAt, SmpteSyncMetadataTlv, TimestampedUdpSocket, TimingProfileConfig,
    ABUTRACK_DEFAULT_TELEMETRY_PORT, ABUTRACK_DEFAULT_VIDEO_RTP_PORT, ABU_ECOSYSTEM_BEACON_MAGIC,
    ABUSYNC_EVENT_PORT_UNPRIVILEGED, ABUSYNC_GENERAL_PORT_UNPRIVILEGED,
};
use abuclock::{AbuScanCaptureRecord, AbuScanSessionManifest};

pub mod mcp;
pub mod server;
pub mod ui_html;

/// Operating role of the local Abu Clock node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingRole {
    Master,
    Follower,
    AutoBmca,
}

/// Active network synchronization profile selected in the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuiTimingProfile {
    SmpteSt2059_2,
    Ieee802_1AsGptp,
    Aes67Harmonized,
    AbuSyncCsptp,
}

/// Decoded inspection view of the 20-byte SMPTE ST 2059-2 Synchronization Metadata TLV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmpteTlvInspection {
    pub wire_hex: String,
    pub wire_size_bytes: usize,
    pub tlv_type_hex: String,
    pub org_id_hex: String,
    pub org_subtype_hex: String,
    pub current_utc_offset_sec: i16,
    pub default_system_frame_rate_num: u32,
    pub default_system_frame_rate_den: u32,
    pub timecode_flags_hex: String,
    pub drop_frame: bool,
}

/// Per-sensor receipt simulated or collected after a scheduled barrier capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCaptureReceipt {
    pub sensor_id: String,
    pub platform: String,
    pub target_tai_ns: u64,
    pub actual_tai_ns: u64,
    pub jitter_ns: i64,
    pub filename: String,
    pub within_subframe_budget: bool,
}

/// Record of a scheduled future-time barrier capture (`SCHEDULE_CAPTURE_AT`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureExecutionLog {
    pub sequence_id: u64,
    pub mode_label: String,
    pub target_tai_ns: u64,
    pub lead_time_ms: u64,
    pub smpte_timecode: String,
    pub bcd_hex: String,
    pub master_jitter_ns: i64,
    pub receipts: Vec<SensorCaptureReceipt>,
}

/// Decoded inspection view of OpenTrackIO (`OTrk`), FreeD (`0xD1` 29B/37B, `0xD2` 41B), and `AbuTrack` LiveLink JSON packets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryInspection {
    pub sequence_num: u16,
    pub camera_pose: CameraPose6DoF,
    pub opentrackio_json: String,
    pub opentrackio_wire_hex_prefix: String,
    pub opentrackio_total_bytes: usize,
    pub freed_d1_hex: String,
    pub freed_d1_checksum_hex: String,
    pub freed_37b_hex: String,
    pub freed_d2_hex: String,
    pub freed_d2_checksum_hex: String,
    pub abutrack_livelink_json: String,
}

/// Phase 3 Hardware Grandmaster Appliance status (`Raspberry Pi CM4 + DS3231 TCXO + PHC`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplianceStatusSnapshot {
    pub gnss_or_tcxo_locked: bool,
    pub holdover_active: bool,
    pub primary_phc_device: String,
    pub secondary_lidar_phc_device: String,
    pub rtc_device: String,
    pub studio_interface: String,
    pub lidar_interface: String,
    pub sched_fifo_priority: u8,
    pub tai_minus_utc_seconds: i32,
    pub seeded_phc_tai_ns: u64,
    pub ptp_clock_class: u8,
    pub ptp_clock_accuracy_hex: String,
    pub active_profiles: Vec<String>,
}

/// Live UDP socket transport statistics from `abu_network::TimestampedUdpSocket`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpSocketStatus {
    pub bound_addr: String,
    pub is_privileged_port: bool,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub last_tx_tai_ns: u64,
}

/// Complete state payload returned to the Unified Abu Clock Control Center frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSlateSnapshot {
    pub version: String,
    pub monotonic_backend: String,
    pub role: OperatingRole,
    pub profile: GuiTimingProfile,
    pub ptp_domain: u8,
    pub ptp_port_state: PtpPortState,
    pub priority1: u8,
    pub priority2: u8,
    pub slate_timecode: String,
    pub bcd_hex: String,
    pub frame_index: u64,
    pub phase: f64,
    pub flash_marker_active: bool,
    pub tai_ns: u64,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub frame_rate_hz: f64,
    pub drop_frame: bool,
    pub user_offset_ns: i64,
    pub trigger_lead_time_ms: u64,
    pub auto_sync_enabled: bool,
    pub keep_screen_on_active: bool,
    pub battery_optimization_unrestricted: bool,
    pub convergence: ClockConvergenceMetrics,
    pub offset_history_ns: Vec<i64>,
    pub drift_history_ppb: Vec<f64>,
    pub smpte_tlv: SmpteTlvInspection,
    pub last_bmca_log: String,
    pub capture_logs: Vec<CaptureExecutionLog>,
    pub telemetry: TelemetryInspection,
    pub manifest_abuscan_json: String,
    pub manifest_colmap_txt: String,
    pub appliance: ApplianceStatusSnapshot,
    pub udp_socket: UdpSocketStatus,
    pub ecosystem_nodes: Vec<AbuEcosystemNodeInfo>,
    pub discovery_beacon: AbuEcosystemDiscoveryBeacon,
}

/// Configuration update payload sent from the UI parameter settings panel.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigUpdateRequest {
    pub role: OperatingRole,
    pub profile: GuiTimingProfile,
    pub ptp_domain: u8,
    pub priority1: u8,
    pub priority2: u8,
    pub fps_num: u32,
    pub fps_den: u32,
    pub drop_frame: bool,
    pub user_offset_ns: i64,
    pub trigger_lead_time_ms: u64,
    pub is_wireless: bool,
    pub keep_screen_on_active: bool,
    #[serde(default = "default_true")]
    pub auto_sync_enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Stateful controller backing the Abu Clock PC application and binding all workspace crates.
pub struct GuiAppController {
    pub clock: SharedClock,
    pub engine: SmpteTimecodeEngine,
    pub ptp_engine: DualModePtpEngine,
    pub scan_coordinator: AbuScanCoordinator,
    pub udp_socket: Option<TimestampedUdpSocket>,
    pub udp_bound_addr: String,
    pub udp_is_privileged: bool,
    pub udp_tx_packets: u64,
    pub udp_rx_packets: u64,
    pub udp_last_tx_tai_ns: u64,
    pub role: OperatingRole,
    pub profile: GuiTimingProfile,
    pub ptp_domain: u8,
    pub priority1: u8,
    pub priority2: u8,
    pub trigger_lead_time_ms: u64,
    pub auto_sync_enabled: bool,
    pub is_wireless: bool,
    pub keep_screen_on_active: bool,
    pub battery_optimization_unrestricted: bool,
    pub sim_mono_cursor_ns: u64,
    pub offset_history_ns: Vec<i64>,
    pub drift_history_ppb: Vec<f64>,
    pub last_bmca_log: String,
    pub capture_logs: Vec<CaptureExecutionLog>,
    pub camera_pose: CameraPose6DoF,
    pub telemetry_seq: u16,
    pub session_manifest: AbuScanSessionManifest,
    pub appliance_gnss_locked: bool,
    pub ecosystem_nodes: Vec<AbuEcosystemNodeInfo>,
}

impl Default for GuiAppController {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiAppController {
    /// Initializes the unified Abu Clock controller with SMPTE ST 2059-2 Domain 1 defaults (`29.97 DF`),
    /// binds a real `TimestampedUdpSocket`, and runs an initial 8-second Kalman convergence warm-up.
    pub fn new() -> Self {
        let initial_tai_ns = 1_729_180_800_000_000_000u64;
        let c_t0 = PlatformMonotonicClock::now_ns();
        let clock = SharedClock::new(OverlayClock::with_monotonic_anchor(initial_tai_ns, c_t0));
        let engine = SmpteTimecodeEngine::new(FrameRate::FPS_29_97_DF, 0);
        let profile_cfg = TimingProfileConfig::abusync_unprivileged(1);
        let ptp_engine = DualModePtpEngine::new(
            profile_cfg,
            [0x00, 0x1A, 0x2B, 0xFF, 0xFE, 0xAB, 0xCD, 0x01],
            FrameRate::FPS_29_97_DF,
            clock.clone(),
        );
        let scan_coordinator = AbuScanCoordinator::new(clock.clone(), engine);

        let preferred_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            ABUSYNC_EVENT_PORT_UNPRIVILEGED,
        );
        let (udp_socket, udp_bound_addr, udp_is_privileged) =
            match TimestampedUdpSocket::bind_dual_mode(preferred_addr, clock.clone()) {
                Ok(sock) => {
                    let addr_str = sock
                        .local_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "127.0.0.1:50319".to_string());
                    let priv_port = sock.is_privileged_ptp_port;
                    (Some(sock), addr_str, priv_port)
                }
                Err(_) => (None, "127.0.0.1:50319 (Simulated)".to_string(), false),
            };

        let mut ctrl = Self {
            clock,
            engine,
            ptp_engine,
            scan_coordinator,
            udp_socket,
            udp_bound_addr,
            udp_is_privileged,
            udp_tx_packets: 0,
            udp_rx_packets: 0,
            udp_last_tx_tai_ns: initial_tai_ns,
            role: OperatingRole::AutoBmca,
            profile: GuiTimingProfile::AbuSyncCsptp,
            ptp_domain: 1,
            priority1: 128,
            priority2: 128,
            trigger_lead_time_ms: 50,
            auto_sync_enabled: true,
            is_wireless: false,
            keep_screen_on_active: true,
            battery_optimization_unrestricted: true,
            sim_mono_cursor_ns: c_t0,
            offset_history_ns: Vec::with_capacity(48),
            drift_history_ppb: Vec::with_capacity(48),
            last_bmca_log: "Local node initialized in AutoBmca (Listening -> Master candidate, Priority1=128, Domain=1).".to_string(),
            capture_logs: Vec::new(),
            camera_pose: CameraPose6DoF {
                camera_id: 1,
                pan_deg: 12.5,
                tilt_deg: -4.25,
                roll_deg: 0.15,
                x_mm: 1250.0,
                y_mm: -400.0,
                z_mm: 1700.0,
                zoom_raw: 2048,
                focus_raw: 4096,
            },
            telemetry_seq: 1,
            session_manifest: AbuScanSessionManifest::new("AbuStudio_Session_01"),
            appliance_gnss_locked: true,
            ecosystem_nodes: default_ecosystem_nodes(
                initial_tai_ns,
                FrameRate::FPS_29_97_DF,
                0,
            ),
        };

        // Pre-warm the Kalman servo through the 7.0s ST 2059 gate so initial state is Locked
        ctrl.fast_forward_warmup();
        // Seed one initial capture record into the manifest & log
        ctrl.trigger_scheduled_capture();
        ctrl
    }

    fn emit_loopback_ptp_packet(&mut self) {
        let msg = self.ptp_engine.build_announce();
        if let Some(ref sock) = self.udp_socket {
            if let Ok(target) = sock.local_addr() {
                if let Ok((_, tai_tx, _)) = sock.send_timestamped(&msg, target) {
                    self.udp_tx_packets = self.udp_tx_packets.wrapping_add(1);
                    self.udp_last_tx_tai_ns = tai_tx;
                }
                if sock.recv_timestamped().is_ok() {
                    self.udp_rx_packets = self.udp_rx_packets.wrapping_add(1);
                }
            }
        } else {
            self.udp_tx_packets = self.udp_tx_packets.wrapping_add(1);
            self.udp_rx_packets = self.udp_rx_packets.wrapping_add(1);
            self.udp_last_tx_tai_ns = self.clock.now_tai_ns();
        }
    }

    fn push_history(&mut self, offset_ns: i64, drift_ppb: f64) {
        if self.offset_history_ns.len() >= 40 {
            self.offset_history_ns.remove(0);
        }
        if self.drift_history_ppb.len() >= 40 {
            self.drift_history_ppb.remove(0);
        }
        self.offset_history_ns.push(offset_ns);
        self.drift_history_ppb.push(drift_ppb);
    }

    /// Periodic background 8 Hz servo tick when `auto_sync_enabled` is active.
    pub fn background_servo_tick(&mut self) {
        if !self.auto_sync_enabled {
            return;
        }
        let state = self.clock.metrics().lock_state;
        if state == LockState::Unlocked {
            return;
        }
        let raw_mono = PlatformMonotonicClock::now_ns();
        let pseudo = ((raw_mono / 1_000) % 29) as i64 - 14;
        let sample_offset = if self.is_wireless {
            pseudo * 18
        } else {
            pseudo
        };
        let delay = if self.is_wireless { 2_350 } else { 375 };
        self.inject_network_sample(sample_offset, delay);
    }

    /// Runs 64 closed-loop 8 Hz network discipline steps (8.0 simulated seconds) so the
    /// 7-second SMPTE ST 2059 warm-up gate completes and converges below 100 ns.
    pub fn fast_forward_warmup(&mut self) -> ClockConvergenceMetrics {
        self.auto_sync_enabled = true;
        let now_mono = PlatformMonotonicClock::now_ns().max(self.sim_mono_cursor_ns);
        let mut last = self.clock.metrics();
        let offsets = [320i64, -180, 95, -60, 42, -28, 19, -14, 11, -8, 6, -5];
        for step in 1..=64 {
            self.sim_mono_cursor_ns = now_mono + step * 125_000_000;
            let sample_offset = offsets[(step as usize) % offsets.len()];
            let delay = if self.is_wireless { 2_400 } else { 380 };
            last = self.clock.discipline_from_network_sample(
                sample_offset,
                delay,
                self.sim_mono_cursor_ns,
                self.is_wireless,
            );
            self.push_history(last.offset_ns, last.drift_ppb);
        }
        self.emit_loopback_ptp_packet();
        last
    }

    /// Injects a single network PTP/CSPTP sample into the live Rust `OverlayClock` Kalman servo.
    pub fn inject_network_sample(
        &mut self,
        measured_offset_ns: i64,
        path_delay_ns: u64,
    ) -> ClockConvergenceMetrics {
        let real_mono = PlatformMonotonicClock::now_ns();
        self.sim_mono_cursor_ns = self.sim_mono_cursor_ns.max(real_mono) + 125_000_000;
        let m = self.clock.discipline_from_network_sample(
            measured_offset_ns,
            path_delay_ns,
            self.sim_mono_cursor_ns,
            self.is_wireless,
        );
        self.push_history(m.offset_ns, m.drift_ppb);
        self.emit_loopback_ptp_packet();
        m
    }

    /// Forces Locking/Holdover (`offset_ns = 3_500 ns`) or Unlocked (`offset_ns = 1_500_000 ns`) state for testing fault recovery.
    pub fn set_simulated_lock_override(&mut self, offset_ns: i64, elapsed_lock_ns: u64) {
        self.auto_sync_enabled = false;
        let drift = self.clock.metrics().drift_ppb;
        let delay = if self.is_wireless { 2_800 } else { 380 };
        let m = self
            .clock
            .update_telemetry(offset_ns, drift, delay, elapsed_lock_ns, self.is_wireless);
        self.push_history(m.offset_ns, m.drift_ppb);
    }

    /// Applies parameter settings from the Unified Control Center UI directly to the Rust engines.
    pub fn apply_config(&mut self, req: ConfigUpdateRequest) {
        self.role = req.role;
        self.profile = req.profile;
        self.ptp_domain = req.ptp_domain.min(127);
        self.priority1 = req.priority1;
        self.priority2 = req.priority2;
        self.trigger_lead_time_ms = req.trigger_lead_time_ms.clamp(1, 5_000);
        self.is_wireless = req.is_wireless;
        self.keep_screen_on_active = req.keep_screen_on_active;
        self.auto_sync_enabled = req.auto_sync_enabled;

        let rate = FrameRate::new(req.fps_num.max(1), req.fps_den.max(1), req.drop_frame);
        self.engine = SmpteTimecodeEngine::new(rate, req.user_offset_ns);

        let mut profile_cfg = match req.profile {
            GuiTimingProfile::SmpteSt2059_2 => TimingProfileConfig::smpte_st2059_2(self.ptp_domain),
            GuiTimingProfile::Ieee802_1AsGptp => TimingProfileConfig::ieee_802_1as_gptp(),
            GuiTimingProfile::Aes67Harmonized => {
                TimingProfileConfig::aes67_harmonized(self.ptp_domain)
            }
            GuiTimingProfile::AbuSyncCsptp => {
                TimingProfileConfig::abusync_unprivileged(self.ptp_domain)
            }
        };
        profile_cfg.priority1 = self.priority1;
        profile_cfg.priority2 = self.priority2;

        self.ptp_engine.profile = profile_cfg;
        self.ptp_engine.local_dataset.priority1 = self.priority1;
        self.ptp_engine.local_dataset.priority2 = self.priority2;
        self.ptp_engine.active_frame_rate = rate;
        self.ptp_engine.port_state = match self.role {
            OperatingRole::Master => PtpPortState::Master,
            OperatingRole::Follower => PtpPortState::Slave,
            OperatingRole::AutoBmca => self.ptp_engine.port_state,
        };

        self.scan_coordinator = AbuScanCoordinator::new(self.clock.clone(), self.engine)
            .with_lead_time_ns(self.trigger_lead_time_ms * 1_000_000);

        let cur = self.clock.metrics();
        self.clock.update_telemetry(
            cur.offset_ns,
            cur.drift_ppb,
            if self.is_wireless { 2_200 } else { 380 },
            cur.elapsed_lock_ns,
            self.is_wireless,
        );
        self.emit_loopback_ptp_packet();
    }

    /// Simulates receiving an IEEE 1588 `Announce` packet from a remote node and runs BMCA election.
    pub fn test_bmca_election(
        &mut self,
        remote_priority1: u8,
        remote_clock_class: u8,
        remote_priority2: u8,
        remote_fps_num: u32,
        remote_fps_den: u32,
        remote_drop_frame: bool,
    ) -> PtpPortState {
        let remote_ds = BmcaDataset {
            priority1: remote_priority1,
            clock_class: remote_clock_class,
            clock_accuracy: 0x20,
            offset_scaled_log_variance: 0x4000,
            priority2: remote_priority2,
            clock_identity: [0xDC, 0xA6, 0x32, 0xFF, 0xFE, 0x00, 0x00, 0x01],
            steps_removed: 1,
        };
        let remote_rate = FrameRate::new(
            remote_fps_num.max(1),
            remote_fps_den.max(1),
            remote_drop_frame,
        );
        let tlv = SmpteSyncMetadataTlv::from_frame_rate(remote_rate);
        let new_state = self.ptp_engine.process_announce(remote_ds, Some(tlv));

        if new_state == PtpPortState::Slave {
            self.engine = SmpteTimecodeEngine::new(
                self.ptp_engine.active_frame_rate,
                self.engine.user_offset_ns,
            );
            self.last_bmca_log = format!(
                "BMCA Elected REMOTE Grandmaster (Priority1={}, Class={}, Priority2={}) -> Local port transitioned to SLAVE and locked to {} / {} ({}).",
                remote_priority1,
                remote_clock_class,
                remote_priority2,
                remote_fps_num,
                remote_fps_den,
                if remote_drop_frame { "DF" } else { "NDF" }
            );
        } else {
            self.last_bmca_log = format!(
                "BMCA Elected LOCAL Node as MASTER (Local Priority1={} beat or matched Remote Priority1={}) -> Port state is MASTER.",
                self.priority1, remote_priority1
            );
        }
        self.emit_loopback_ptp_packet();
        new_state
    }

    /// Schedules a future-time ($T_{\text{now}} + \Delta T_{\text{lead}}$) multi-sensor capture
    /// and records deterministic multi-rig receipts + session manifest entries.
    pub fn trigger_scheduled_capture(&mut self) -> ScheduleCaptureAt {
        let cmd = self.scan_coordinator.schedule_single_capture();
        self.record_capture_execution(cmd, "SingleCapture (+LeadTime)");
        cmd
    }

    /// Schedules a multi-frame interval burst scan (`total_frames`) and records the execution.
    pub fn trigger_interval_burst(&mut self, total_frames: u32) -> ScheduleCaptureAt {
        let cmd = self
            .scan_coordinator
            .schedule_interval_scan(self.engine.frame_rate, total_frames.max(1));
        self.record_capture_execution(
            cmd,
            &format!(
                "IntervalScan ({} frames @ {:.2} Hz)",
                total_frames,
                self.engine.frame_rate.as_hz()
            ),
        );
        cmd
    }

    fn record_capture_execution(&mut self, cmd: ScheduleCaptureAt, mode_label: &str) {
        let bcd_u32 = cmd.smpte_tc.to_bcd_u32();
        let tc_str = cmd.smpte_tc.to_slate_string();
        let base_err = self.clock.metrics().offset_ns;

        let sensors = [
            ("iPhone16Pro_CamA", "iOS-AbuClockKit", "dng", base_err + 14),
            ("WinRig_CamB", "Windows-QPC-C++", "dng", base_err - 9),
            ("RoboSense_E1R_LiDAR", "gPTP-802.1AS", "pcd", base_err + 6),
            ("UE5_nDisplay_Node01", "AbuClockUE-FFI", "exr", base_err + 3),
        ];

        let mut receipts = Vec::with_capacity(sensors.len());
        for (idx, (sensor_id, platform, ext, jitter)) in sensors.iter().enumerate() {
            let actual_tai = (cmd.target_tai_ns as i128 + *jitter as i128).max(0) as u64;
            let filename = cmd.format_capture_filename(sensor_id, ext);
            receipts.push(SensorCaptureReceipt {
                sensor_id: (*sensor_id).to_string(),
                platform: (*platform).to_string(),
                target_tai_ns: cmd.target_tai_ns,
                actual_tai_ns: actual_tai,
                jitter_ns: *jitter,
                filename: filename.clone(),
                within_subframe_budget: jitter.abs() < 50_000,
            });

            if idx == 0 {
                let rec_id = (self.session_manifest.captures.len() + 1) as u32;
                self.session_manifest.push_record(AbuScanCaptureRecord {
                    image_id: rec_id,
                    sensor_id: (*sensor_id).to_string(),
                    file_path: filename,
                    smpte_timecode: tc_str.clone(),
                    bcd_timecode: bcd_u32,
                    tai_ns: actual_tai,
                    error_ns: *jitter,
                    qw: 1.0,
                    qx: 0.0,
                    qy: 0.0,
                    qz: 0.0,
                    tx_m: self.camera_pose.x_mm / 1000.0,
                    ty_m: self.camera_pose.y_mm / 1000.0,
                    tz_m: self.camera_pose.z_mm / 1000.0,
                    camera_model_id: 1,
                });
            }
        }

        let log = CaptureExecutionLog {
            sequence_id: cmd.sequence_id,
            mode_label: mode_label.to_string(),
            target_tai_ns: cmd.target_tai_ns,
            lead_time_ms: self.trigger_lead_time_ms,
            smpte_timecode: tc_str,
            bcd_hex: format!("0x{:08X}", bcd_u32),
            master_jitter_ns: base_err,
            receipts,
        };
        self.capture_logs.insert(0, log);
        if self.capture_logs.len() > 12 {
            self.capture_logs.pop();
        }
        self.emit_loopback_ptp_packet();
    }

    /// Updates the 6-DoF camera tracking pose for OpenTrackIO (`OTrk`) and FreeD (`0xD1`/`0xD2`) synthesis.
    pub fn update_camera_pose(&mut self, pose: CameraPose6DoF) {
        self.camera_pose = pose;
        self.telemetry_seq = self.telemetry_seq.wrapping_add(1);
        self.emit_loopback_ptp_packet();
    }

    /// Adds a custom capture entry to the `.abuscan.json` / Colmap `images.txt` session manifest.
    pub fn add_manifest_record(&mut self, sensor_id: &str, ext: &str) {
        let tai_ns = self.clock.now_tai_ns();
        let synth = self.engine.synthesize_at_tai_ns(tai_ns);
        let clean_ext = ext.trim_start_matches('.');
        let filename = format!(
            "{}_{}_{}.{}",
            sensor_id,
            synth.timecode.to_compact_filename_token(),
            tai_ns,
            clean_ext
        );
        let rec_id = (self.session_manifest.captures.len() + 1) as u32;
        self.session_manifest.push_record(AbuScanCaptureRecord {
            image_id: rec_id,
            sensor_id: sensor_id.to_string(),
            file_path: filename,
            smpte_timecode: synth.timecode.to_slate_string(),
            bcd_timecode: synth.timecode.to_bcd_u32(),
            tai_ns,
            error_ns: self.clock.metrics().offset_ns,
            qw: 1.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
            tx_m: self.camera_pose.x_mm / 1000.0,
            ty_m: self.camera_pose.y_mm / 1000.0,
            tz_m: self.camera_pose.z_mm / 1000.0,
            camera_model_id: 1,
        });
    }

    /// Updates the Phase 3 Grandmaster Appliance GNSS/TCXO lock state.
    pub fn set_appliance_gnss_lock(&mut self, locked: bool) {
        self.appliance_gnss_locked = locked;
    }

    /// Registers or refreshes an Abu Ecosystem node (`iOS AbuTrack`, `Windows DesktopSync`, `UE5 LiveLinkAbuTrack`).
    pub fn register_ecosystem_node(&mut self, mut node: AbuEcosystemNodeInfo) {
        node.last_seen_tai_ns = self.clock.now_tai_ns();
        if let Some(existing) = self
            .ecosystem_nodes
            .iter_mut()
            .find(|n| n.node_id == node.node_id)
        {
            *existing = node;
        } else {
            self.ecosystem_nodes.push(node);
        }
    }

    /// Builds a live `AbuEcosystemDiscoveryBeacon` so any iOS, PC, or UE5 node can discover
    /// the Master Clock and all registered sibling nodes in a single query.
    pub fn discovery_beacon(&self) -> AbuEcosystemDiscoveryBeacon {
        let tai_ns = self.clock.now_tai_ns();
        let synth = self.engine.synthesize_at_tai_ns(tai_ns);
        let bcd = synth.timecode.to_bcd_u32();
        let mut nodes = self.ecosystem_nodes.clone();
        let fps_str = format!(
            "{:.3} Hz ({})",
            self.engine.frame_rate.as_hz(),
            if self.engine.frame_rate.drop_frame {
                "DF"
            } else {
                "NDF"
            }
        );
        let cur_offset = self.clock.metrics().offset_ns;
        for n in &mut nodes {
            n.frame_rate_str = fps_str.clone();
            n.last_seen_tai_ns = tai_ns;
            if n.node_id == "abu-clock-master-01" {
                n.offset_ns = cur_offset;
                n.lock_state = format!("{:?}", self.clock.metrics().lock_state);
            }
        }
        AbuEcosystemDiscoveryBeacon {
            magic: ABU_ECOSYSTEM_BEACON_MAGIC.to_string(),
            version: ABU_CLOCK_VERSION.to_string(),
            master_ip: "127.0.0.1".to_string(),
            csptp_event_port: ABUSYNC_EVENT_PORT_UNPRIVILEGED,
            sntp_general_port: ABUSYNC_GENERAL_PORT_UNPRIVILEGED,
            http_api_port: 7878,
            abutrack_telemetry_port: ABUTRACK_DEFAULT_TELEMETRY_PORT,
            abutrack_video_port: ABUTRACK_DEFAULT_VIDEO_RTP_PORT,
            ptp_domain: self.ptp_domain,
            tai_ns,
            smpte_timecode: synth.timecode.to_slate_string(),
            bcd_hex: format!("0x{:08X}", bcd),
            fps_num: self.engine.frame_rate.numerator,
            fps_den: self.engine.frame_rate.denominator,
            drop_frame: self.engine.frame_rate.drop_frame,
            trigger_lead_time_ms: self.trigger_lead_time_ms,
            nodes,
        }
    }

    /// Generates an instantaneous `GuiSlateSnapshot` for the Unified Control Center frontend.
    pub fn snapshot(&self) -> GuiSlateSnapshot {
        let tai_ns = self.clock.now_tai_ns();
        let synth = self.engine.synthesize_at_tai_ns(tai_ns);
        let bcd = synth.timecode.to_bcd_u32();
        let convergence = self.clock.metrics();

        let tlv = SmpteSyncMetadataTlv::from_frame_rate(self.engine.frame_rate);
        let tlv_bytes = tlv.to_bytes();
        let wire_hex = tlv_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let smpte_tlv = SmpteTlvInspection {
            wire_hex,
            wire_size_bytes: SmpteSyncMetadataTlv::WIRE_SIZE,
            tlv_type_hex: "0x0003 (Organization Extension)".to_string(),
            org_id_hex: "00-0E-9B (SMPTE)".to_string(),
            org_subtype_hex: "00-00-01 (ST 2059-2 SM)".to_string(),
            current_utc_offset_sec: tlv.current_utc_offset_sec,
            default_system_frame_rate_num: tlv.default_system_frame_rate_num,
            default_system_frame_rate_den: tlv.default_system_frame_rate_den,
            timecode_flags_hex: format!("0x{:02X}", tlv.timecode_flags),
            drop_frame: (tlv.timecode_flags & 0x01) != 0,
        };

        let otrk_payload = OpenTrackIoPayload::new(
            self.telemetry_seq,
            tai_ns,
            synth.timecode,
            self.engine.frame_rate,
            self.camera_pose,
        );
        let otrk_pkt = OpenTrackIoPacket {
            encoding: OpenTrackIoEncoding::Json,
            sequence_num: self.telemetry_seq,
            offset: 0,
            payload: otrk_payload.clone(),
        };
        let otrk_bytes = otrk_pkt.to_bytes();
        let otrk_prefix = otrk_bytes
            .iter()
            .take(14)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let freed = FreeDPacket::new(self.camera_pose, synth.timecode, tai_ns);
        let d1_bytes = freed.to_standard_d1_bytes();
        let d37_bytes = freed.to_abutrack_37b_bytes();
        let d2_bytes = freed.to_extended_d2_bytes();
        let d1_hex = d1_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        let d37_hex = d37_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        let d2_hex = d2_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let livelink_pkt = AbuTrackLiveLinkPacket::from_pose_and_clock(
            "AbuTrack",
            self.camera_pose,
            24.0,
            -0.042,
            0.011,
            synth.timecode,
            tai_ns,
            &format!("{:?}", convergence.lock_state),
        );

        let telemetry = TelemetryInspection {
            sequence_num: self.telemetry_seq,
            camera_pose: self.camera_pose,
            opentrackio_json: serde_json::to_string_pretty(&otrk_payload).unwrap_or_default(),
            opentrackio_wire_hex_prefix: otrk_prefix,
            opentrackio_total_bytes: otrk_bytes.len(),
            freed_d1_hex: d1_hex,
            freed_d1_checksum_hex: format!("0x{:02X}", freed_checksum(&d1_bytes[..28])),
            freed_37b_hex: d37_hex,
            freed_d2_hex: d2_hex,
            freed_d2_checksum_hex: format!("0x{:02X}", freed_checksum(&d2_bytes[..40])),
            abutrack_livelink_json: serde_json::to_string_pretty(&livelink_pkt).unwrap_or_default(),
        };

        let appliance = ApplianceStatusSnapshot {
            gnss_or_tcxo_locked: self.appliance_gnss_locked,
            holdover_active: !self.appliance_gnss_locked,
            primary_phc_device: "/dev/ptp0 (Broadcom BCM54210PE / RP1 PHY)".to_string(),
            secondary_lidar_phc_device: "/dev/ptp1 (PCIe Intel I210-T1 Hardware PHC)".to_string(),
            rtc_device: "/dev/rtc0 (Maxim DS3231 TCXO ±2 ppm)".to_string(),
            studio_interface: "eth0 (SMPTE ST 2059-2 + AbuSync CSPTP)".to_string(),
            lidar_interface: "eth1 (IEEE 802.1AS gPTP L2 P2P)".to_string(),
            sched_fifo_priority: 80,
            tai_minus_utc_seconds: DEFAULT_TAI_MINUS_UTC_SECONDS,
            seeded_phc_tai_ns: tai_ns,
            ptp_clock_class: if self.appliance_gnss_locked { 6 } else { 7 },
            ptp_clock_accuracy_hex: if self.appliance_gnss_locked {
                "0x20 (< 25 ns)".to_string()
            } else {
                "0x21 (< 100 ns TCXO Holdover)".to_string()
            },
            active_profiles: vec![
                format!(
                    "SMPTE ST 2059-2 (Domain {}, 224.0.1.129:319/320)",
                    self.ptp_domain
                ),
                "IEEE 802.1AS gPTP (Domain 0, 224.0.0.107 L2 P2P)".to_string(),
                format!(
                    "AbuSync CSPTP (Domain {}, UDP 50319/50320 + mDNS)",
                    self.ptp_domain
                ),
            ],
        };

        let udp_socket = UdpSocketStatus {
            bound_addr: self.udp_bound_addr.clone(),
            is_privileged_port: self.udp_is_privileged,
            tx_packets: self.udp_tx_packets,
            rx_packets: self.udp_rx_packets,
            last_tx_tai_ns: self.udp_last_tx_tai_ns,
        };

        let discovery_beacon = self.discovery_beacon();
        let ecosystem_nodes = discovery_beacon.nodes.clone();

        GuiSlateSnapshot {
            version: ABU_CLOCK_VERSION.to_string(),
            monotonic_backend: PlatformMonotonicClock::backend_name().to_string(),
            role: self.role,
            profile: self.profile,
            ptp_domain: self.ptp_domain.min(127),
            ptp_port_state: self.ptp_engine.port_state,
            priority1: self.priority1,
            priority2: self.priority2,
            slate_timecode: synth.timecode.to_slate_string(),
            bcd_hex: format!("0x{:08X}", bcd),
            frame_index: synth.frame_index,
            phase: synth.phase,
            flash_marker_active: synth.timecode.frames == 0 && synth.phase < 0.25,
            tai_ns: synth.tai_ns,
            frame_rate_num: self.engine.frame_rate.numerator,
            frame_rate_den: self.engine.frame_rate.denominator,
            frame_rate_hz: self.engine.frame_rate.as_hz(),
            drop_frame: self.engine.frame_rate.drop_frame,
            user_offset_ns: self.engine.user_offset_ns,
            trigger_lead_time_ms: self.trigger_lead_time_ms,
            auto_sync_enabled: self.auto_sync_enabled,
            keep_screen_on_active: self.keep_screen_on_active,
            battery_optimization_unrestricted: self.battery_optimization_unrestricted,
            convergence,
            offset_history_ns: self.offset_history_ns.clone(),
            drift_history_ppb: self.drift_history_ppb.clone(),
            smpte_tlv,
            last_bmca_log: self.last_bmca_log.clone(),
            capture_logs: self.capture_logs.clone(),
            telemetry,
            manifest_abuscan_json: self.session_manifest.to_abuscan_json(),
            manifest_colmap_txt: self.session_manifest.to_colmap_images_txt(),
            appliance,
            udp_socket,
            ecosystem_nodes,
            discovery_beacon,
        }
    }

    /// Updates the active rational frame rate and user phase offset.
    pub fn set_timing_config(
        &mut self,
        role: OperatingRole,
        profile: GuiTimingProfile,
        ptp_domain: u8,
        frame_rate: FrameRate,
        user_offset_ns: i64,
    ) {
        self.apply_config(ConfigUpdateRequest {
            role,
            profile,
            ptp_domain,
            priority1: self.priority1,
            priority2: self.priority2,
            fps_num: frame_rate.numerator,
            fps_den: frame_rate.denominator,
            drop_frame: frame_rate.drop_frame,
            user_offset_ns,
            trigger_lead_time_ms: self.trigger_lead_time_ms,
            is_wireless: self.is_wireless,
            keep_screen_on_active: self.keep_screen_on_active,
            auto_sync_enabled: self.auto_sync_enabled,
        });
    }

    /// Checks whether the node is ready for production capture (either `Master` or `Locked` after 7s warm-up).
    pub fn is_capture_ready(&self) -> bool {
        match self.role {
            OperatingRole::Master => true,
            OperatingRole::Follower | OperatingRole::AutoBmca => {
                self.clock.metrics().lock_state == LockState::Locked
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_controller_uses_domain_1_and_schedules_capture() {
        let mut ctrl = GuiAppController::new();
        let snap = ctrl.snapshot();
        assert_eq!(snap.ptp_domain, 1);
        assert!(snap.keep_screen_on_active);
        assert!(snap.battery_optimization_unrestricted);
        assert!(snap.slate_timecode.contains(';'));
        assert_eq!(snap.convergence.lock_state, LockState::Locked);
        assert_eq!(snap.smpte_tlv.wire_size_bytes, 20);
        assert!(snap.manifest_abuscan_json.contains("abuscan.manifest.v1"));
        assert!(snap.udp_socket.tx_packets >= 1);

        let cap = ctrl.trigger_scheduled_capture();
        assert_eq!(cap.sequence_id, 2);
        assert_eq!(snap.ecosystem_nodes.len(), 4);
        assert!(!snap.telemetry.freed_37b_hex.is_empty());
    }

    #[test]
    fn mcp_jsonrpc_and_ecosystem_discovery_work() {
        let mut ctrl = GuiAppController::new();
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });
        let init_resp = mcp::handle_mcp_request(&mut ctrl, &init_req).unwrap();
        assert_eq!(init_resp["result"]["serverInfo"]["name"], "abu-clock-mcp");

        let list_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });
        let list_resp = mcp::handle_mcp_request(&mut ctrl, &list_req).unwrap();
        assert_eq!(list_resp["result"]["tools"].as_array().unwrap().len(), 7);

        let call_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "abu_clock_get_integration_guide",
                "arguments": { "target": "all" }
            }
        });
        let call_resp = mcp::handle_mcp_request(&mut ctrl, &call_req).unwrap();
        let text = call_resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("NTPSyncClient.swift"));
        assert!(text.contains("LiveLinkAbuTrackSource.cpp"));
    }
}
