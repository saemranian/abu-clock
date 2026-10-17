//! Rational frame rate and LiDAR sensor sweep cadence definitions.
//!
//! Represents every supported broadcast, cinema, high-frame-rate, and LiDAR cadence
//! as an exact rational fraction $F = F_{\text{num}} / F_{\text{den}}$ with optional
//! SMPTE ST 12-1 Drop-Frame skipping.

use serde::{Deserialize, Serialize};

/// Exact rational frame rate or sensor sweep cadence ($F = F_{\text{num}} / F_{\text{den}}$).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameRate {
    /// Numerator $F_{\text{num}}$ (e.g. `24000`, `30000`, `60000`, `24`, `25`, `30`, `60`, `10`).
    pub numerator: u32,
    /// Denominator $F_{\text{den}}$ (`1001` for fractional broadcast rates, `1` for integer rates).
    pub denominator: u32,
    /// Whether SMPTE ST 12-1 Drop-Frame frame-number compensation is active.
    pub drop_frame: bool,
}

impl FrameRate {
    pub const FPS_23_976: Self = Self {
        numerator: 24_000,
        denominator: 1_001,
        drop_frame: false,
    };
    pub const FPS_24: Self = Self {
        numerator: 24,
        denominator: 1,
        drop_frame: false,
    };
    pub const FPS_25: Self = Self {
        numerator: 25,
        denominator: 1,
        drop_frame: false,
    };
    pub const FPS_29_97_NDF: Self = Self {
        numerator: 30_000,
        denominator: 1_001,
        drop_frame: false,
    };
    pub const FPS_29_97_DF: Self = Self {
        numerator: 30_000,
        denominator: 1_001,
        drop_frame: true,
    };
    pub const FPS_30: Self = Self {
        numerator: 30,
        denominator: 1,
        drop_frame: false,
    };
    pub const FPS_48: Self = Self {
        numerator: 48,
        denominator: 1,
        drop_frame: false,
    };
    pub const FPS_50: Self = Self {
        numerator: 50,
        denominator: 1,
        drop_frame: false,
    };
    pub const FPS_59_94_NDF: Self = Self {
        numerator: 60_000,
        denominator: 1_001,
        drop_frame: false,
    };
    pub const FPS_59_94_DF: Self = Self {
        numerator: 60_000,
        denominator: 1_001,
        drop_frame: true,
    };
    pub const FPS_60: Self = Self {
        numerator: 60,
        denominator: 1,
        drop_frame: false,
    };
    pub const FPS_120: Self = Self {
        numerator: 120,
        denominator: 1,
        drop_frame: false,
    };
    pub const LIDAR_10HZ: Self = Self {
        numerator: 10,
        denominator: 1,
        drop_frame: false,
    };
    pub const LIDAR_15HZ: Self = Self {
        numerator: 15,
        denominator: 1,
        drop_frame: false,
    };
    pub const LIDAR_20HZ: Self = Self {
        numerator: 20,
        denominator: 1,
        drop_frame: false,
    };

    /// Creates a custom rational `FrameRate`.
    pub const fn new(numerator: u32, denominator: u32, drop_frame: bool) -> Self {
        Self {
            numerator,
            denominator: if denominator == 0 { 1 } else { denominator },
            drop_frame,
        }
    }

    /// Returns the floating-point frequency in Hertz ($\text{Hz}$).
    #[inline]
    pub fn as_hz(&self) -> f64 {
        (self.numerator as f64) / (self.denominator as f64)
    }

    /// Returns the nominal integer timecode counting base (e.g. `24` for `23.976`, `30` for `29.97`, `60` for `59.94`).
    #[inline]
    pub const fn nominal_fps(&self) -> u32 {
        (self.numerator + self.denominator - 1) / self.denominator
    }

    /// Returns the exact duration of a single timestep frame in nanoseconds:
    ///
    /// $$D_{\text{frame}} = \frac{10^9 \times F_{\text{den}}}{F_{\text{num}}}$$
    #[inline]
    pub const fn frame_duration_ns(&self) -> u64 {
        ((1_000_000_000u128 * (self.denominator as u128)) / (self.numerator as u128)) as u64
    }

    /// Returns the OpenTrackIO / SMPTE RIS rational denominator (`1001` for fractional rates, `1000` or `1` for integer rates).
    #[inline]
    pub const fn opentrackio_rational(&self) -> (u32, u32) {
        if self.denominator == 1001 {
            (self.numerator, 1001)
        } else {
            (self.numerator * 1000, 1000)
        }
    }

    /// Returns all standard presets supported by Abu Clock.
    pub const fn all_presets() -> &'static [FrameRate] {
        &[
            Self::FPS_23_976,
            Self::FPS_24,
            Self::FPS_25,
            Self::FPS_29_97_NDF,
            Self::FPS_29_97_DF,
            Self::FPS_30,
            Self::FPS_48,
            Self::FPS_50,
            Self::FPS_59_94_NDF,
            Self::FPS_59_94_DF,
            Self::FPS_60,
            Self::FPS_120,
            Self::LIDAR_10HZ,
            Self::LIDAR_15HZ,
            Self::LIDAR_20HZ,
        ]
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        Self::FPS_29_97_DF
    }
}
