# Cross-Boundary Public API Contracts (`INTERFACES.md`)

This document defines the canonical cross-crate interfaces shared between Tier 1 (`crates/abu-clock-core`), Tier 2 (`crates/abu-network`), and Tier 3 (`crates/abu-ffi`, `apps/abu-clock-gui`, and `/integrations/*`).

---

## 1. `crates/abu-clock-core` Public API

### `clock` Module
- `PlatformMonotonicClock`:
  - `PlatformMonotonicClock::now_ns() -> u64`
  - `PlatformMonotonicClock::backend_name() -> &'static str`
- `OverlayClock` & `SharedClock`:
  - `OverlayClock::new(initial_tai_ns: u64) -> Self`
  - `OverlayClock::now_tai_ns(&self) -> u64`
  - `OverlayClock::translate_monotonic_to_tai(&self, mono_ns: u64) -> u64`
  - `OverlayClock::steer_frequency_ppb(&self, drift_ppb: f64)`
  - `OverlayClock::adjust_phase_ns(&self, offset_ns: i64)`
  - `OverlayClock::update_telemetry(&self, offset_ns: i64, drift_ppb: f64, path_delay_ns: u64, is_wireless: bool)`
  - `OverlayClock::metrics(&self) -> ClockConvergenceMetrics`
- `LockState`:
  - `Unlocked` ($|\text{offset}| > 1\text{ ms}$)
  - `Locking` ($1\text{ }\mu\text{s} \le |\text{offset}| \le 1\text{ ms}$ or warm-up $< 7.0\text{ s}$)
  - `Locked` ($|\text{offset}| < 1\text{ }\mu\text{s}$ wired / $< 500\text{ }\mu\text{s}$ wireless after $7.0\text{ s}$ warm-up)

### `smpte` Module
- `FrameRate`:
  - Rational `numerator: u32`, `denominator: u32`, `drop_frame: bool`
  - Presets: `FPS_23_976`, `FPS_24`, `FPS_25`, `FPS_29_97_NDF`, `FPS_29_97_DF`, `FPS_30`, `FPS_48`, `FPS_50`, `FPS_59_94_NDF`, `FPS_59_94_DF`, `FPS_60`, `FPS_120`, `LIDAR_10HZ`, `LIDAR_15HZ`, `LIDAR_20HZ`
- `SmpteTimecode`:
  - `hours: u8`, `minutes: u8`, `seconds: u8`, `frames: u8`, `subframe: f64`, `drop_frame: bool`
  - `to_bcd_u32(&self) -> u32`, `from_bcd_u32(bcd: u32, drop_frame: bool) -> Self`
- `SmpteTimecodeEngine`:
  - `synthesize_at_tai_ns(&self, tai_ns: u64) -> FrameSynthesisSnapshot`
- `TimestepTicker`:
  - Hybrid sleep ($\Delta t_{\text{sleep}} - 500\text{ }\mu\text{s}$) + `core::hint::spin_loop()` to $\phi = 0.0$

---

## 2. `crates/abu-network` Public API

### `ptp` Module
- `TimingProfile`:
  - `SmpteSt2059_2` (Multicast `224.0.1.129`, Ports `319`/`320`, Default Domain `1`)
  - `Ieee802_1AsGptp` (Multicast `224.0.0.107` / L2 P2P, `8 Hz` for RoboSense E1R LiDAR)
  - `Aes67Harmonized` (AES-R16-2016)
  - `AbuSyncCsptp` (Unprivileged UDP Ports `50319`/`50320` + mDNS `_abuclock._udp.local.`)
- `SmpteSyncMetadataTlv`:
  - SMPTE ST 2059-2 Organization Extension TLV (`0x000E9B`) attached to PTP `Announce` frames.

### `trigger` & `telemetry` Modules
- `AbuScanCoordinator`:
  - `schedule_single_capture(now_tai_ns: u64, rate: FrameRate) -> ScheduledCaptureCommand` ($T_{\text{now}} + 50\text{ ms}$)
  - `schedule_interval_scan(...)`
- `OpenTrackIoPacket`:
  - 14-byte `"OTrk"` (`0x4F54726B`) header + JSON (`0x01`) / CBOR (`0x02`) payload + Fletcher-16 (mod 256) checksum.
- `FreeDPacket`:
  - Standard 29-byte `0xD1` and Extended 41-byte `0xD2` (`FreeD-TC` with 32-bit BCD timecode + 64-bit TAI nanoseconds).
