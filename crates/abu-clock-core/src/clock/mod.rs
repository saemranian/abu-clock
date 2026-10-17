//! Cross-platform hardware monotonic counters, `OverlayClock` virtual PLL, and convergence telemetry.

pub mod metrics;
pub mod monotonic;
pub mod overlay;

pub use metrics::{
    ClockConvergenceMetrics, EmpiricalSt2059Trajectory, LockState, UNLOCKED_THRESHOLD_NS,
    WARMUP_GATE_NS, WIRED_LOCKED_THRESHOLD_NS, WIRELESS_LOCKED_THRESHOLD_NS,
};
pub use monotonic::PlatformMonotonicClock;
pub use overlay::{ClockKalmanFilter, OverlayClock, SharedClock};
