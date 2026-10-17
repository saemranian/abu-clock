//! Scheduled Future-Time Multi-Sensor Triggering (`AbuScanCoordinator`).
//!
//! Eliminates variable wired/wireless packet transit jitter (`0.3 ms` LAN to `12 ms` Wi-Fi)
//! by never issuing immediate unscheduled capture commands. Instead, broadcasts
//! `ScheduleCaptureAt(target_tai_ns, smpte_tc)` scheduled `50 ms` (`50_000_000 ns`)
//! in the future so every camera and LiDAR node counts down locally against its
//! disciplined `OverlayClock` and triggers at the exact same physical nanosecond.

use std::thread;
use std::time::Duration;

use abu_clock_core::{
    FrameRate, SharedClock, SmpteTimecode, SmpteTimecodeEngine, SPINLOCK_GUARD_WINDOW_NS,
};
use serde::{Deserialize, Serialize};

/// Default future lead time for scheduled multi-sensor captures (`50 ms` = `50_000_000 ns`).
pub const DEFAULT_TRIGGER_LEAD_TIME_NS: u64 = 50_000_000;

/// Trigger mode for an `AbuScan` capture session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureTriggerMode {
    /// Single simultaneous shutter / LiDAR slice capture at `target_tai_ns`.
    SingleCapture,
    /// Continuous interval scan starting at `target_tai_ns` and repeating every `interval_ns`.
    IntervalScan { interval_ns: u64, total_frames: u32 },
}

/// Network command broadcast by the `AbuScanCoordinator` master to all connected camera/LiDAR nodes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScheduleCaptureAt {
    /// Monotonic trigger sequence identifier.
    pub sequence_id: u64,
    /// Exact future TAI nanosecond ($T_{\text{exec}} = T_{\text{now}} + 50\text{ ms}$) when all sensors fire.
    pub target_tai_ns: u64,
    /// Synthesized SMPTE ST 12-1 timecode at `target_tai_ns`.
    pub smpte_tc: SmpteTimecode,
    /// Active frame rate / sweep cadence.
    pub frame_rate: FrameRate,
    /// Single capture or continuous interval scan.
    pub mode: CaptureTriggerMode,
}

impl ScheduleCaptureAt {
    /// Generates the deterministic Abu Scan capture filename `<SensorID>_<TC_HHMMSSFF>_<TAI_ns>.<ext>`.
    pub fn format_capture_filename(&self, sensor_id: &str, ext: &str) -> String {
        let clean_ext = ext.trim_start_matches('.');
        format!(
            "{}_{}_{}.{}",
            sensor_id,
            self.smpte_tc.to_compact_filename_token(),
            self.target_tai_ns,
            clean_ext
        )
    }
}

/// Coordinator that schedules future-time triggers and executes deterministic follower countdowns.
pub struct AbuScanCoordinator {
    clock: SharedClock,
    engine: SmpteTimecodeEngine,
    lead_time_ns: u64,
    next_seq: u64,
}

impl AbuScanCoordinator {
    /// Creates a new `AbuScanCoordinator` with the standard `50 ms` future trigger lead time.
    pub fn new(clock: SharedClock, engine: SmpteTimecodeEngine) -> Self {
        Self {
            clock,
            engine,
            lead_time_ns: DEFAULT_TRIGGER_LEAD_TIME_NS,
            next_seq: 1,
        }
    }

    /// Overrides the future trigger lead time (in nanoseconds).
    pub fn with_lead_time_ns(mut self, lead_time_ns: u64) -> Self {
        self.lead_time_ns = lead_time_ns;
        self
    }

    /// Schedules a single multi-sensor capture at $T_{\text{exec}} = T_{\text{now}} + \text{lead\_time\_ns}$.
    pub fn schedule_single_capture(&mut self) -> ScheduleCaptureAt {
        let now_tai = self.clock.now_tai_ns();
        self.schedule_single_capture_from(now_tai)
    }

    /// Schedules a single capture relative to an explicit `now_tai_ns` timestamp.
    pub fn schedule_single_capture_from(&mut self, now_tai_ns: u64) -> ScheduleCaptureAt {
        let target_tai_ns = now_tai_ns.saturating_add(self.lead_time_ns);
        let snap = self.engine.synthesize_at_tai_ns(target_tai_ns);
        let seq = self.next_seq;
        self.next_seq += 1;

        ScheduleCaptureAt {
            sequence_id: seq,
            target_tai_ns,
            smpte_tc: snap.timecode,
            frame_rate: self.engine.frame_rate,
            mode: CaptureTriggerMode::SingleCapture,
        }
    }

    /// Schedules a continuous interval scan (e.g. `10 Hz` synchronized LiDAR + photo array)
    /// starting at $T_{\text{now}} + 50\text{ ms}$.
    pub fn schedule_interval_scan(
        &mut self,
        scan_rate: FrameRate,
        total_frames: u32,
    ) -> ScheduleCaptureAt {
        let now_tai = self.clock.now_tai_ns();
        let target_tai_ns = now_tai.saturating_add(self.lead_time_ns);
        let snap = self.engine.synthesize_at_tai_ns(target_tai_ns);
        let seq = self.next_seq;
        self.next_seq += 1;

        ScheduleCaptureAt {
            sequence_id: seq,
            target_tai_ns,
            smpte_tc: snap.timecode,
            frame_rate: scan_rate,
            mode: CaptureTriggerMode::IntervalScan {
                interval_ns: scan_rate.frame_duration_ns(),
                total_frames,
            },
        }
    }

    /// Follower-side deterministic countdown: waits against the disciplined `OverlayClock`
    /// using coarse sleep + final `500 us` hardware spinlock until `cmd.target_tai_ns` is reached,
    /// returning the actual execution TAI nanosecond and trigger jitter (`actual - target`).
    pub fn await_scheduled_trigger(&self, cmd: &ScheduleCaptureAt) -> (u64, i64) {
        let now_tai = self.clock.now_tai_ns();
        if cmd.target_tai_ns > now_tai {
            let remaining_ns = cmd.target_tai_ns - now_tai;
            if remaining_ns > SPINLOCK_GUARD_WINDOW_NS {
                thread::sleep(Duration::from_nanos(remaining_ns - SPINLOCK_GUARD_WINDOW_NS));
            }
            while self.clock.now_tai_ns() < cmd.target_tai_ns {
                core::hint::spin_loop();
            }
        }
        let actual_tai = self.clock.now_tai_ns();
        let jitter_ns = (actual_tai as i128 - cmd.target_tai_ns as i128) as i64;
        (actual_tai, jitter_ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abu_clock_core::OverlayClock;

    #[test]
    fn scheduled_capture_adds_50ms_and_formats_filename() {
        let clock = OverlayClock::new_shared(1_729_180_800_000_000_000);
        let engine = SmpteTimecodeEngine::new(FrameRate::FPS_25, 0);
        let mut coord = AbuScanCoordinator::new(clock, engine);

        let cmd = coord.schedule_single_capture_from(1_729_180_800_000_000_000);
        assert_eq!(
            cmd.target_tai_ns,
            1_729_180_800_000_000_000 + DEFAULT_TRIGGER_LEAD_TIME_NS
        );

        let filename = cmd.format_capture_filename("iPhone16Pro_CamA", "dng");
        assert!(filename.starts_with("iPhone16Pro_CamA_"));
        assert!(filename.ends_with("_1729180800050000000.dng"));
    }

    #[test]
    fn follower_countdown_executes_at_scheduled_tai_ns() {
        let clock = OverlayClock::new_shared(1_729_180_800_000_000_000);
        let engine = SmpteTimecodeEngine::new(FrameRate::LIDAR_10HZ, 0);
        // Use 2 ms lead time in unit test for fast execution
        let mut coord = AbuScanCoordinator::new(clock, engine).with_lead_time_ns(2_000_000);
        let cmd = coord.schedule_single_capture();
        let (actual_tai, jitter_ns) = coord.await_scheduled_trigger(&cmd);
        assert!(actual_tai >= cmd.target_tai_ns);
        assert!(
            jitter_ns.abs() < 500_000,
            "Trigger jitter exceeded bound: {} ns",
            jitter_ns
        );
    }
}
