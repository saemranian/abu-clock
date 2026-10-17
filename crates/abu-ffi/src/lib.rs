//! `abu-ffi` (`libabuclock`): Cross-Language UniFFI & C-ABI SDK Bridge.
//!
//! Exports both high-level Rust/UniFFI structs (`AbuClockSdkEngine`) for Swift/Python/C#
//! and zero-overhead `extern "C"` functions (`abu_clock_*`) matching `include/abu_clock.h`
//! for the Abu Track & Scan Windows C++ client and the Unreal Engine 5 plugin (`AbuClockUE`).

use std::slice;

use abu_clock_core::{
    ClockConvergenceMetrics, FrameRate, OverlayClock, PlatformMonotonicClock, SharedClock,
    SmpteTimecodeEngine, TimestepTicker,
};
use abu_network::{
    AbuScanCoordinator, CameraPose6DoF, FreeDPacket, OpenTrackIoEncoding, OpenTrackIoPacket,
    OpenTrackIoPayload, ScheduleCaptureAt, FREED_EXTENDED_TC_LEN, FREED_STANDARD_LEN,
};
use serde::{Deserialize, Serialize};

pub use abu_clock_core::ABU_CLOCK_VERSION;

/// FFI-friendly timecode and frame phase snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FfiTimecodeSnapshot {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub subframe: f64,
    pub drop_frame: bool,
    pub bcd_u32: u32,
    pub tai_ns: u64,
    pub frame_index: u64,
    pub phase: f64,
    pub sleep_until_next_frame_ns: u64,
    pub slate_text: String,
}

/// High-level SDK engine exported to Swift (`.xcframework`), C++, C#, and Python via UniFFI.
pub struct AbuClockSdkEngine {
    clock: SharedClock,
    engine: SmpteTimecodeEngine,
    ticker: TimestepTicker,
    coordinator: AbuScanCoordinator,
}

impl AbuClockSdkEngine {
    /// Creates a new `AbuClockSdkEngine` initialized at `initial_tai_ns` and the specified rational frame rate.
    pub fn new(initial_tai_ns: u64, fps_num: u32, fps_den: u32, drop_frame: bool) -> Self {
        let rate = FrameRate::new(fps_num, fps_den, drop_frame);
        let clock = OverlayClock::new_shared(initial_tai_ns);
        let engine = SmpteTimecodeEngine::new(rate, 0);
        let ticker = TimestepTicker::new(clock.clone(), engine);
        let coordinator = AbuScanCoordinator::new(clock.clone(), engine);
        Self {
            clock,
            engine,
            ticker,
            coordinator,
        }
    }

    /// Reads the current disciplined TAI nanosecond from `OverlayClock`.
    #[inline]
    pub fn now_tai_ns(&self) -> u64 {
        self.clock.now_tai_ns()
    }

    /// Translates a host OS monotonic timestamp (such as iOS `CMSampleBuffer` / `ARFrame.timestamp`
    /// or Windows `QueryPerformanceCounter`) into disciplined TAI nanoseconds.
    #[inline]
    pub fn translate_monotonic_to_tai_ns(&self, host_mono_ns: u64) -> u64 {
        self.clock.translate_monotonic_to_tai(host_mono_ns)
    }

    /// Synthesizes the SMPTE ST 12-1 timecode and subframe phase at `tai_ns`.
    pub fn synthesize_timecode_at(&self, tai_ns: u64) -> FfiTimecodeSnapshot {
        let snap = self.engine.synthesize_at_tai_ns(tai_ns);
        FfiTimecodeSnapshot {
            hours: snap.timecode.hours,
            minutes: snap.timecode.minutes,
            seconds: snap.timecode.seconds,
            frames: snap.timecode.frames,
            subframe: snap.timecode.subframe,
            drop_frame: snap.timecode.drop_frame,
            bcd_u32: snap.timecode.to_bcd_u32(),
            tai_ns: snap.tai_ns,
            frame_index: snap.frame_index,
            phase: snap.phase,
            sleep_until_next_frame_ns: snap.sleep_until_next_frame_ns,
            slate_text: snap.timecode.to_slate_string(),
        }
    }

    /// Blocks until the exact next $\phi = 0.0$ frame boundary using the hybrid sleep + spinlock ticker.
    pub fn wait_next_timestep_boundary(&self) -> FfiTimecodeSnapshot {
        let ev = self.ticker.wait_next_frame_boundary();
        self.synthesize_timecode_at(ev.actual_dispatch_tai_ns)
    }

    /// Schedules a future-time multi-sensor capture at $T_{\text{now}} + 50\text{ ms}$.
    pub fn schedule_capture_50ms(&mut self) -> ScheduleCaptureAt {
        self.coordinator.schedule_single_capture()
    }

    /// Returns the current `OverlayClock` Kalman convergence metrics.
    pub fn get_metrics(&self) -> ClockConvergenceMetrics {
        self.clock.metrics()
    }

    /// Packs a complete OpenTrackIO (`OTrk`) UDP datagram (14-byte header + JSON payload + Fletcher-16).
    pub fn pack_opentrackio_json(&self, sequence_id: u16, pose: CameraPose6DoF) -> Vec<u8> {
        let tai_ns = self.clock.now_tai_ns();
        let snap = self.engine.synthesize_at_tai_ns(tai_ns);
        let payload = OpenTrackIoPayload::new(
            sequence_id,
            tai_ns,
            snap.timecode,
            self.engine.frame_rate,
            pose,
        );
        let pkt = OpenTrackIoPacket {
            encoding: OpenTrackIoEncoding::Json,
            sequence_num: sequence_id,
            offset: 0,
            payload,
        };
        pkt.to_bytes()
    }

    /// Packs a Standard 29-byte FreeD (`0xD1`) UDP packet.
    pub fn pack_freed_d1(&self, pose: CameraPose6DoF) -> [u8; FREED_STANDARD_LEN] {
        let tai_ns = self.clock.now_tai_ns();
        let snap = self.engine.synthesize_at_tai_ns(tai_ns);
        FreeDPacket::new(pose, snap.timecode, tai_ns).to_standard_d1_bytes()
    }

    /// Packs an Extended 41-byte AbuTrack FreeD-TC (`0xD2`) UDP packet.
    pub fn pack_freed_d2(&self, pose: CameraPose6DoF) -> [u8; FREED_EXTENDED_TC_LEN] {
        let tai_ns = self.clock.now_tai_ns();
        let snap = self.engine.synthesize_at_tai_ns(tai_ns);
        FreeDPacket::new(pose, snap.timecode, tai_ns).to_extended_d2_bytes()
    }
}

// ============================================================================
// C-ABI Exports (matching crates/abu-ffi/include/abu_clock.h)
// ============================================================================

/// Opaque C handle wrapping `AbuClockSdkEngine`.
pub struct AbuClockOpaqueHandle(AbuClockSdkEngine);

/// C-ABI frame synthesis struct matching `AbuClockCFrameSnapshot` in `abu_clock.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AbuClockCFrameSnapshot {
    pub tai_ns: u64,
    pub frame_index: u64,
    pub phase: f64,
    pub sleep_until_next_frame_ns: u64,
    pub bcd_timecode: u32,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub drop_frame: u8,
}

/// C-ABI 6DoF camera pose struct matching `AbuClockCPose6DoF` in `abu_clock.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AbuClockCPose6DoF {
    pub camera_id: u8,
    pub pan_deg: f64,
    pub tilt_deg: f64,
    pub roll_deg: f64,
    pub x_mm: f64,
    pub y_mm: f64,
    pub z_mm: f64,
    pub zoom_raw: u32,
    pub focus_raw: u32,
}

impl From<AbuClockCPose6DoF> for CameraPose6DoF {
    fn from(c: AbuClockCPose6DoF) -> Self {
        Self {
            camera_id: c.camera_id,
            pan_deg: c.pan_deg,
            tilt_deg: c.tilt_deg,
            roll_deg: c.roll_deg,
            x_mm: c.x_mm,
            y_mm: c.y_mm,
            z_mm: c.z_mm,
            zoom_raw: c.zoom_raw,
            focus_raw: c.focus_raw,
        }
    }
}

#[no_mangle]
pub extern "C" fn abu_clock_create(
    initial_tai_ns: u64,
    fps_num: u32,
    fps_den: u32,
    drop_frame: bool,
) -> *mut AbuClockOpaqueHandle {
    let sdk = AbuClockSdkEngine::new(initial_tai_ns, fps_num, fps_den, drop_frame);
    Box::into_raw(Box::new(AbuClockOpaqueHandle(sdk)))
}

/// # Safety
/// `handle` must be a valid pointer returned by `abu_clock_create` or null.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_destroy(handle: *mut AbuClockOpaqueHandle) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// # Safety
/// `handle` must be a valid non-null pointer to `AbuClockOpaqueHandle`.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_now_tai_ns(handle: *const AbuClockOpaqueHandle) -> u64 {
    if handle.is_null() {
        return PlatformMonotonicClock::now_ns();
    }
    unsafe { (*handle).0.now_tai_ns() }
}

/// # Safety
/// `handle` must be a valid non-null pointer to `AbuClockOpaqueHandle`.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_translate_monotonic_to_tai_ns(
    handle: *const AbuClockOpaqueHandle,
    host_mono_ns: u64,
) -> u64 {
    if handle.is_null() {
        return host_mono_ns;
    }
    unsafe { (*handle).0.translate_monotonic_to_tai_ns(host_mono_ns) }
}

/// # Safety
/// `handle` and `out_snapshot` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_synthesize_now(
    handle: *const AbuClockOpaqueHandle,
    out_snapshot: *mut AbuClockCFrameSnapshot,
) -> bool {
    if handle.is_null() || out_snapshot.is_null() {
        return false;
    }
    let sdk = unsafe { &(*handle).0 };
    let snap = sdk.synthesize_timecode_at(sdk.now_tai_ns());
    unsafe {
        *out_snapshot = AbuClockCFrameSnapshot {
            tai_ns: snap.tai_ns,
            frame_index: snap.frame_index,
            phase: snap.phase,
            sleep_until_next_frame_ns: snap.sleep_until_next_frame_ns,
            bcd_timecode: snap.bcd_u32,
            hours: snap.hours,
            minutes: snap.minutes,
            seconds: snap.seconds,
            frames: snap.frames,
            drop_frame: u8::from(snap.drop_frame),
        };
    }
    true
}

/// # Safety
/// `handle` and `out_snapshot` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_wait_next_timestep(
    handle: *const AbuClockOpaqueHandle,
    out_snapshot: *mut AbuClockCFrameSnapshot,
) -> bool {
    if handle.is_null() || out_snapshot.is_null() {
        return false;
    }
    let sdk = unsafe { &(*handle).0 };
    let snap = sdk.wait_next_timestep_boundary();
    unsafe {
        *out_snapshot = AbuClockCFrameSnapshot {
            tai_ns: snap.tai_ns,
            frame_index: snap.frame_index,
            phase: snap.phase,
            sleep_until_next_frame_ns: snap.sleep_until_next_frame_ns,
            bcd_timecode: snap.bcd_u32,
            hours: snap.hours,
            minutes: snap.minutes,
            seconds: snap.seconds,
            frames: snap.frames,
            drop_frame: u8::from(snap.drop_frame),
        };
    }
    true
}

/// # Safety
/// `handle`, `pose`, and `out_buf_29` (at least 29 bytes) must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_pack_freed_d1(
    handle: *const AbuClockOpaqueHandle,
    pose: *const AbuClockCPose6DoF,
    out_buf_29: *mut u8,
) -> usize {
    if handle.is_null() || pose.is_null() || out_buf_29.is_null() {
        return 0;
    }
    let sdk = unsafe { &(*handle).0 };
    let rust_pose: CameraPose6DoF = unsafe { (*pose).into() };
    let bytes = sdk.pack_freed_d1(rust_pose);
    let dst = unsafe { slice::from_raw_parts_mut(out_buf_29, FREED_STANDARD_LEN) };
    dst.copy_from_slice(&bytes);
    FREED_STANDARD_LEN
}

/// # Safety
/// `handle`, `pose`, and `out_buf_41` (at least 41 bytes) must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn abu_clock_pack_freed_d2(
    handle: *const AbuClockOpaqueHandle,
    pose: *const AbuClockCPose6DoF,
    out_buf_41: *mut u8,
) -> usize {
    if handle.is_null() || pose.is_null() || out_buf_41.is_null() {
        return 0;
    }
    let sdk = unsafe { &(*handle).0 };
    let rust_pose: CameraPose6DoF = unsafe { (*pose).into() };
    let bytes = sdk.pack_freed_d2(rust_pose);
    let dst = unsafe { slice::from_raw_parts_mut(out_buf_41, FREED_EXTENDED_TC_LEN) };
    dst.copy_from_slice(&bytes);
    FREED_EXTENDED_TC_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_version_matches_core() {
        assert!(ABU_CLOCK_VERSION.starts_with("v0.0.0."));
    }

    #[test]
    fn c_abi_lifecycle_and_freed_packing_works() {
        unsafe {
            let handle = abu_clock_create(1_729_180_800_000_000_000, 30000, 1001, true);
            assert!(!handle.is_null());

            let mut snap = std::mem::zeroed::<AbuClockCFrameSnapshot>();
            assert!(abu_clock_synthesize_now(handle, &mut snap));
            assert_eq!(snap.drop_frame, 1);

            let pose = AbuClockCPose6DoF {
                camera_id: 3,
                pan_deg: 15.0,
                tilt_deg: -5.0,
                roll_deg: 0.5,
                x_mm: 100.0,
                y_mm: 200.0,
                z_mm: 1500.0,
                zoom_raw: 1024,
                focus_raw: 2048,
            };
            let mut d1 = [0u8; 29];
            let mut d2 = [0u8; 41];
            assert_eq!(abu_clock_pack_freed_d1(handle, &pose, d1.as_mut_ptr()), 29);
            assert_eq!(abu_clock_pack_freed_d2(handle, &pose, d2.as_mut_ptr()), 41);
            assert_eq!(d1[0], 0xD1);
            assert_eq!(d2[0], 0xD2);

            abu_clock_destroy(handle);
        }
    }
}
