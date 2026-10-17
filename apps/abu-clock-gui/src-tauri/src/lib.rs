//! Tauri v2 Rust backend bridge for the Abu Clock cross-platform GUI.
//!
//! Connects the Tauri v2 GUI directly to `abu-clock-core` (`OverlayClock`, `SmpteTimecodeEngine`)
//! and `abu-network` (`DualModePtpEngine`, `AbuScanCoordinator`, `TimingProfileConfig`).

use serde::{Deserialize, Serialize};

use abu_clock_core::{
    ClockConvergenceMetrics, FrameRate, LockState, OverlayClock, PlatformMonotonicClock,
    SharedClock, SmpteTimecodeEngine, ABU_CLOCK_VERSION,
};
use abu_network::{
    AbuScanCoordinator, DualModePtpEngine, PtpPortState, ScheduleCaptureAt, TimingProfileConfig,
};

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

/// Complete state payload returned to the React/TypeScript Digital Slate frontend.
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
    pub frame_rate_hz: f64,
    pub drop_frame: bool,
    pub user_offset_ns: i64,
    pub keep_screen_on_active: bool,
    pub battery_optimization_unrestricted: bool,
    pub convergence: ClockConvergenceMetrics,
}

/// Stateful controller backing the Tauri v2 GUI commands and binding `abu-clock-core` + `abu-network`.
pub struct GuiAppController {
    pub clock: SharedClock,
    pub engine: SmpteTimecodeEngine,
    pub ptp_engine: DualModePtpEngine,
    pub scan_coordinator: AbuScanCoordinator,
    pub role: OperatingRole,
    pub profile: GuiTimingProfile,
    pub ptp_domain: u8,
    pub priority1: u8,
    pub priority2: u8,
    pub keep_screen_on_active: bool,
    pub battery_optimization_unrestricted: bool,
}

impl Default for GuiAppController {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiAppController {
    /// Initializes the GUI controller with SMPTE ST 2059-2 Domain 1 defaults (`29.97 DF`).
    pub fn new() -> Self {
        let initial_tai_ns = 1_729_180_800_000_000_000u64;
        let clock = OverlayClock::new_shared(initial_tai_ns);
        let engine = SmpteTimecodeEngine::new(FrameRate::FPS_29_97_DF, 0);
        let profile_cfg = TimingProfileConfig::abusync_unprivileged(1);
        let ptp_engine = DualModePtpEngine::new(
            profile_cfg,
            [0x00, 0x1A, 0x2B, 0xFF, 0xFE, 0xAB, 0xCD, 0x01],
            FrameRate::FPS_29_97_DF,
            clock.clone(),
        );
        let scan_coordinator = AbuScanCoordinator::new(clock.clone(), engine);

        Self {
            clock,
            engine,
            ptp_engine,
            scan_coordinator,
            role: OperatingRole::AutoBmca,
            profile: GuiTimingProfile::AbuSyncCsptp,
            ptp_domain: 1,
            priority1: 128,
            priority2: 128,
            keep_screen_on_active: true,
            battery_optimization_unrestricted: true,
        }
    }

    /// Generates an instantaneous `GuiSlateSnapshot` for the 60/120 Hz frontend slate renderer.
    pub fn snapshot(&self) -> GuiSlateSnapshot {
        let tai_ns = self.clock.now_tai_ns();
        let synth = self.engine.synthesize_at_tai_ns(tai_ns);
        let bcd = synth.timecode.to_bcd_u32();
        let convergence = self.clock.metrics();

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
            frame_rate_hz: self.engine.frame_rate.as_hz(),
            drop_frame: self.engine.frame_rate.drop_frame,
            user_offset_ns: self.engine.user_offset_ns,
            keep_screen_on_active: self.keep_screen_on_active,
            battery_optimization_unrestricted: self.battery_optimization_unrestricted,
            convergence,
        }
    }

    /// Schedules a future-time ($T_{\text{now}} + 50\text{ ms}$) multi-sensor capture from the GUI.
    pub fn trigger_scheduled_capture(&mut self) -> ScheduleCaptureAt {
        self.scan_coordinator.schedule_single_capture()
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
        self.role = role;
        self.profile = profile;
        self.ptp_domain = ptp_domain.min(127);
        self.engine = SmpteTimecodeEngine::new(frame_rate, user_offset_ns);
        self.ptp_engine.active_frame_rate = frame_rate;
        self.scan_coordinator = AbuScanCoordinator::new(self.clock.clone(), self.engine);
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

        let cap = ctrl.trigger_scheduled_capture();
        assert_eq!(cap.sequence_id, 1);
    }
}
