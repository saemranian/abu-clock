//! `abu-clock-core`: Tier 1 Core Timing Engine and Hardware Clock Abstraction.
//!
//! Provides cross-platform monotonic hardware counter access, the `OverlayClock`
//! virtual Phase-Locked Loop (PLL), SMPTE ST 2059-1 / ST 12-1 timecode math,
//! and the hybrid sleep + spinlock deterministic timestep (`Hz`) ticker.

/// Canonical 4-part crate version synchronized by `scripts/bump_version.py`.
pub const ABU_CLOCK_VERSION: &str = "v0.0.0.1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_initialized() {
        assert!(ABU_CLOCK_VERSION.starts_with("v0.0.0."));
    }
}
