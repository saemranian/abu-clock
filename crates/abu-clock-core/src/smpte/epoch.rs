//! SMPTE ST 2059-1 TAI Epoch alignment and frame-index derivation mathematics.
//!
//! Anchors all calculations to `1970-01-01T00:00:00` International Atomic Time (TAI).
//! Using the 80-bit PTP timestamp representation ($S_{\text{TAI}}, N_{\text{TAI}}$):
//!
//! $$T_{\text{ns}} = S_{\text{TAI}} \times 10^9 + N_{\text{TAI}} + \Delta_{\text{offset}}$$

use serde::{Deserialize, Serialize};

use super::rate::FrameRate;
use super::timecode::SmpteTimecode;

/// Standard current TAI minus UTC leap-second offset (`37` seconds per IERS Bulletin C).
pub const DEFAULT_TAI_MINUS_UTC_SECONDS: i32 = 37;

/// 80-bit IEEE 1588-2019 / SMPTE ST 2059-1 timestamp (`48-bit` seconds + `32-bit` nanoseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtpTimestamp80 {
    /// 48-bit unsigned TAI seconds since `1970-01-01T00:00:00 TAI`.
    pub seconds_tai: u64,
    /// 32-bit unsigned nanoseconds within the current second (`0..1_000_000_000`).
    pub nanoseconds_tai: u32,
}

impl PtpTimestamp80 {
    /// Constructs an 80-bit PTP timestamp from total TAI nanoseconds.
    #[inline]
    pub const fn from_tai_ns(tai_ns: u64) -> Self {
        Self {
            seconds_tai: (tai_ns / 1_000_000_000) & 0x0000_FFFF_FFFF_FFFF,
            nanoseconds_tai: (tai_ns % 1_000_000_000) as u32,
        }
    }

    /// Converts the 80-bit PTP timestamp into total TAI nanoseconds.
    #[inline]
    pub const fn to_tai_ns(&self) -> u64 {
        self.seconds_tai
            .saturating_mul(1_000_000_000)
            .saturating_add(self.nanoseconds_tai as u64)
    }

    /// Serializes into a 10-byte big-endian IEEE 1588 wire representation.
    #[inline]
    pub fn to_be_bytes(&self) -> [u8; 10] {
        let sec_bytes = self.seconds_tai.to_be_bytes();
        let ns_bytes = self.nanoseconds_tai.to_be_bytes();
        [
            sec_bytes[2],
            sec_bytes[3],
            sec_bytes[4],
            sec_bytes[5],
            sec_bytes[6],
            sec_bytes[7],
            ns_bytes[0],
            ns_bytes[1],
            ns_bytes[2],
            ns_bytes[3],
        ]
    }

    /// Parses a 10-byte big-endian IEEE 1588 wire timestamp.
    #[inline]
    pub fn from_be_bytes(b: [u8; 10]) -> Self {
        let seconds_tai = u64::from_be_bytes([0, 0, b[0], b[1], b[2], b[3], b[4], b[5]]);
        let nanoseconds_tai = u32::from_be_bytes([b[6], b[7], b[8], b[9]]);
        Self {
            seconds_tai,
            nanoseconds_tai,
        }
    }
}

/// Complete real-time synthesis output produced by `SmpteTimecodeEngine` on every query.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrameSynthesisSnapshot {
    /// Disciplined TAI nanoseconds including programmable user offset ($T_{\text{ns}}$).
    pub tai_ns: u64,
    /// Duration of one frame in nanoseconds ($D_{\text{frame}}$).
    pub frame_duration_ns: u64,
    /// Total monotonic frame index since the SMPTE Epoch ($K_{\text{frame}} = \lfloor T_{\text{ns}} / D_{\text{frame}} \rfloor$).
    pub frame_index: u64,
    /// Instantaneous timestep phase within the current frame ($\phi \in [0.0, 1.0)$).
    pub phase: f64,
    /// Exact time remaining in nanoseconds until the next $\phi = 0.0$ frame boundary ($\Delta t_{\text{sleep}}$).
    pub sleep_until_next_frame_ns: u64,
    /// Synthesized SMPTE ST 12-1 timecode (`HH:MM:SS:FF.sf`).
    pub timecode: SmpteTimecode,
}

/// SMPTE ST 2059-1 / ST 12-1 synthesis engine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SmpteTimecodeEngine {
    pub frame_rate: FrameRate,
    /// Programmable user phase offset $\Delta_{\text{offset}}$ in nanoseconds.
    pub user_offset_ns: i64,
    /// Dynamic TAI-to-UTC leap-second offset (`default = 37s`).
    pub tai_minus_utc_seconds: i32,
}

impl Default for SmpteTimecodeEngine {
    fn default() -> Self {
        Self::new(FrameRate::FPS_29_97_DF, 0)
    }
}

impl SmpteTimecodeEngine {
    /// Creates a new `SmpteTimecodeEngine` for the given `FrameRate` and user offset $\Delta_{\text{offset}}$.
    pub const fn new(frame_rate: FrameRate, user_offset_ns: i64) -> Self {
        Self {
            frame_rate,
            user_offset_ns,
            tai_minus_utc_seconds: DEFAULT_TAI_MINUS_UTC_SECONDS,
        }
    }

    /// Synthesizes the frame index $K_{\text{frame}}$, subframe phase $\phi \in [0.0, 1.0)$,
    /// sleep duration $\Delta t_{\text{sleep}}$, and SMPTE ST 12-1 timecode from a raw TAI nanosecond reading.
    #[inline]
    pub fn synthesize_at_tai_ns(&self, raw_tai_ns: u64) -> FrameSynthesisSnapshot {
        let adjusted_tai_ns =
            ((raw_tai_ns as i128) + (self.user_offset_ns as i128)).max(0) as u64;
        let d_frame_ns = self.frame_rate.frame_duration_ns().max(1);

        // Use exact rational arithmetic: K_frame = floor(T_ns * F_num / (10^9 * F_den))
        let num = self.frame_rate.numerator as u128;
        let den_ns = 1_000_000_000u128 * (self.frame_rate.denominator as u128);
        let scaled_t = (adjusted_tai_ns as u128) * num;

        let k_frame = (scaled_t / den_ns) as u64;
        let rem_scaled = scaled_t % den_ns;
        let rem_ns = (rem_scaled / num) as u64;
        let phase = (rem_scaled as f64) / (den_ns as f64);
        let sleep_until_next_frame_ns = d_frame_ns.saturating_sub(rem_ns).max(1);

        let timecode = SmpteTimecode::from_frame_index(k_frame, phase, self.frame_rate);

        FrameSynthesisSnapshot {
            tai_ns: adjusted_tai_ns,
            frame_duration_ns: d_frame_ns,
            frame_index: k_frame,
            phase,
            sleep_until_next_frame_ns,
            timecode,
        }
    }
}
