//! Convergence metrics, 7-second warm-up gate, and tri-state lock status for `OverlayClock`.
//!
//! Models the empirical SMPTE ST 2059 follower convergence trajectory:
//! - `2.630 s` -> `501..1,000 ns`
//! - `3.130 s` -> `201..500 ns`
//! - `5.631 s` -> `101..200 ns`
//! - `6.135 s` -> `0..100 ns`
//! Enforces a mandatory `7.0 s` (`7_000_000_000 ns`) warm-up gate prior to asserting `LockState::Locked`.

use serde::{Deserialize, Serialize};

/// Mandatory 7-second warm-up convergence gate in nanoseconds (`7_000_000_000 ns`).
pub const WARMUP_GATE_NS: u64 = 7_000_000_000;

/// Threshold above which the clock is considered `Unlocked` (`1 ms` = `1_000_000 ns`).
pub const UNLOCKED_THRESHOLD_NS: i64 = 1_000_000;

/// Threshold below which a wired LAN follower is considered `Locked` (`1 us` = `1_000 ns`).
pub const WIRED_LOCKED_THRESHOLD_NS: i64 = 1_000;

/// Threshold below which a wireless (Wi-Fi) follower is considered `Locked` (`500 us` = `500_000 ns`).
pub const WIRELESS_LOCKED_THRESHOLD_NS: i64 = 500_000;

/// Tri-state synchronization lock indicator exposed to the UI, SDK, and hardware daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockState {
    /// Phase offset exceeds `1 ms` (`1_000_000 ns`) or no valid master packets received.
    Unlocked,
    /// Offset is between `1 us` and `1 ms`, or the 7-second warm-up gate has not yet elapsed.
    Locking,
    /// Warm-up gate (`>= 7.0 s`) has elapsed and offset is `< 1 us` (wired) or `< 500 us` (wireless).
    Locked,
}

impl LockState {
    /// Evaluates the tri-state lock status given the current phase offset, elapsed convergence time,
    /// and whether the network transport is wireless.
    #[inline]
    pub fn evaluate(offset_ns: i64, elapsed_lock_ns: u64, is_wireless: bool) -> Self {
        let abs_offset = offset_ns.saturating_abs();
        if abs_offset > UNLOCKED_THRESHOLD_NS {
            return LockState::Unlocked;
        }
        let target_bound = if is_wireless {
            WIRELESS_LOCKED_THRESHOLD_NS
        } else {
            WIRED_LOCKED_THRESHOLD_NS
        };
        if elapsed_lock_ns >= WARMUP_GATE_NS && abs_offset <= target_bound {
            LockState::Locked
        } else {
            LockState::Locking
        }
    }
}

/// Snapshot of the `OverlayClock` Kalman filter and convergence state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClockConvergenceMetrics {
    /// Instantaneous phase offset from Grandmaster in nanoseconds ($\theta_{\text{phase}}$).
    pub offset_ns: i64,
    /// Estimated local crystal frequency drift in parts-per-billion ($\alpha_{\text{freq}} \times 10^9$).
    pub drift_ppb: f64,
    /// Estimated one-way network path delay in nanoseconds.
    pub path_delay_ns: u64,
    /// Estimated phase variance (uncertainty bound in $\text{ns}^2$) from the Kalman filter.
    pub offset_variance_ns2: f64,
    /// Elapsed nanoseconds since continuous convergence began.
    pub elapsed_lock_ns: u64,
    /// Warm-up gate progress ratio in $[0.0, 1.0]$.
    pub warmup_progress: f64,
    /// Whether the active transport profile is operating over wireless media.
    pub is_wireless: bool,
    /// Tri-state lock status (`Unlocked`, `Locking`, `Locked`).
    pub lock_state: LockState,
}

impl Default for ClockConvergenceMetrics {
    fn default() -> Self {
        Self {
            offset_ns: 0,
            drift_ppb: 0.0,
            path_delay_ns: 0,
            offset_variance_ns2: 1.0e12,
            elapsed_lock_ns: 0,
            warmup_progress: 0.0,
            is_wireless: false,
            lock_state: LockState::Unlocked,
        }
    }
}

/// Empirical SMPTE ST 2059 convergence reference curve for telemetry & verification.
#[derive(Debug, Clone, Copy)]
pub struct EmpiricalSt2059Trajectory;

impl EmpiricalSt2059Trajectory {
    /// Returns the expected upper phase jitter bound (in nanoseconds) at a given elapsed time
    /// in nanoseconds after initial Grandmaster packet exchange.
    pub fn expected_jitter_bound_ns(elapsed_ns: u64) -> u64 {
        if elapsed_ns >= 6_135_000_000 {
            100
        } else if elapsed_ns >= 5_631_000_000 {
            200
        } else if elapsed_ns >= 3_130_000_000 {
            500
        } else if elapsed_ns >= 2_630_000_000 {
            1_000
        } else {
            1_000_000
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_second_warmup_gate_and_tristate_transitions() {
        // Above 1 ms -> Unlocked even after 10 seconds
        assert_eq!(
            LockState::evaluate(1_500_000, 10_000_000_000, false),
            LockState::Unlocked
        );

        // 500 ns at 3.13s -> Locking (because < 7.0s warm-up gate)
        assert_eq!(
            LockState::evaluate(500, 3_130_000_000, false),
            LockState::Locking
        );

        // 80 ns at 6.135s -> Still Locking until 7.000s warm-up gate completes
        assert_eq!(
            LockState::evaluate(80, 6_135_000_000, false),
            LockState::Locking
        );

        // 80 ns at 7.000s on wired LAN -> Locked
        assert_eq!(
            LockState::evaluate(80, 7_000_000_000, false),
            LockState::Locked
        );

        // 250 us at 7.5s -> Locking on wired (< 1 us required), Locked on wireless (< 500 us required)
        assert_eq!(
            LockState::evaluate(250_000, 7_500_000_000, false),
            LockState::Locking
        );
        assert_eq!(
            LockState::evaluate(250_000, 7_500_000_000, true),
            LockState::Locked
        );
    }

    #[test]
    fn empirical_st2059_curve_matches_spec() {
        assert_eq!(
            EmpiricalSt2059Trajectory::expected_jitter_bound_ns(2_630_000_000),
            1_000
        );
        assert_eq!(
            EmpiricalSt2059Trajectory::expected_jitter_bound_ns(3_130_000_000),
            500
        );
        assert_eq!(
            EmpiricalSt2059Trajectory::expected_jitter_bound_ns(5_631_000_000),
            200
        );
        assert_eq!(
            EmpiricalSt2059Trajectory::expected_jitter_bound_ns(6_135_000_000),
            100
        );
    }
}
