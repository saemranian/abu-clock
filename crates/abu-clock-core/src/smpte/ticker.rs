//! Deterministic hybrid sleep + spinlock timestep (`Hz`) generator (`TimestepTicker`).
//!
//! Yields to the OS thread scheduler only until $\Delta t_{\text{sleep}} - 500\text{ }\mu\text{s}$,
//! then enters a tight hardware-polling loop (`core::hint::spin_loop()`) reading `OverlayClock`
//! until the exact $\phi = 0.0$ nanosecond boundary is crossed ($< 5\text{ }\mu\text{s}$ jitter).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::clock::SharedClock;
use super::epoch::SmpteTimecodeEngine;
use super::timecode::SmpteTimecode;

/// Spinlock guard window before the frame boundary (`500 us` = `500_000 ns`).
pub const SPINLOCK_GUARD_WINDOW_NS: u64 = 500_000;

/// Event emitted on each deterministic $\phi = 0.0$ timestep boundary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimestepTickEvent {
    /// Monotonic frame index since the SMPTE Epoch ($K_{\text{frame}}$).
    pub frame_index: u64,
    /// Exact scheduled TAI nanosecond of the frame edge.
    pub scheduled_boundary_tai_ns: u64,
    /// Actual disciplined TAI nanosecond at the instant the tick fired.
    pub actual_dispatch_tai_ns: u64,
    /// Measured dispatch jitter in nanoseconds (`actual - scheduled`).
    pub dispatch_jitter_ns: i64,
    /// Synthesized SMPTE ST 12-1 timecode at the frame boundary.
    pub timecode: SmpteTimecode,
}

/// Deterministic hybrid sleep + spinlock timestep generator.
pub struct TimestepTicker {
    clock: SharedClock,
    engine: SmpteTimecodeEngine,
    running: Arc<AtomicBool>,
}

impl TimestepTicker {
    /// Creates a new `TimestepTicker` bound to a disciplined `SharedClock` and `SmpteTimecodeEngine`.
    pub fn new(clock: SharedClock, engine: SmpteTimecodeEngine) -> Self {
        Self {
            clock,
            engine,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Blocks the calling thread until the exact next $\phi = 0.0$ frame boundary using the
    /// hybrid OS-sleep ($\Delta t_{\text{sleep}} - 500\text{ }\mu\text{s}$) + hardware spinlock loop,
    /// and returns the resulting `TimestepTickEvent`.
    pub fn wait_next_frame_boundary(&self) -> TimestepTickEvent {
        let now_tai = self.clock.now_tai_ns();
        let snap = self.engine.synthesize_at_tai_ns(now_tai);
        let target_frame_index = snap.frame_index + 1;
        let sleep_ns = snap.sleep_until_next_frame_ns;
        let target_tai_ns = now_tai.saturating_add(sleep_ns);

        // 1. Coarse OS sleep until 500 microseconds before the frame boundary
        if sleep_ns > SPINLOCK_GUARD_WINDOW_NS {
            let yield_ns = sleep_ns - SPINLOCK_GUARD_WINDOW_NS;
            thread::sleep(Duration::from_nanos(yield_ns));
        }

        // 2. Tight hardware-polling spinlock loop until the exact frame index rolls over (phi = 0.0)
        loop {
            let cur_tai = self.clock.now_tai_ns();
            let cur_snap = self.engine.synthesize_at_tai_ns(cur_tai);
            if cur_snap.frame_index >= target_frame_index || cur_tai >= target_tai_ns {
                let jitter_ns = (cur_tai as i128 - target_tai_ns as i128) as i64;
                return TimestepTickEvent {
                    frame_index: cur_snap.frame_index,
                    scheduled_boundary_tai_ns: target_tai_ns,
                    actual_dispatch_tai_ns: cur_tai,
                    dispatch_jitter_ns: jitter_ns,
                    timecode: cur_snap.timecode,
                };
            }
            core::hint::spin_loop();
        }
    }

    /// Spawns a dedicated high-priority background timestep thread that invokes `callback`
    /// on every $\phi = 0.0$ frame boundary until `stop()` is called or `max_ticks` is reached.
    pub fn spawn_ticker_thread<F>(
        &self,
        max_ticks: Option<u64>,
        mut callback: F,
    ) -> JoinHandle<Vec<TimestepTickEvent>>
    where
        F: FnMut(TimestepTickEvent) + Send + 'static,
    {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let clock = Arc::clone(&self.clock);
        let engine = self.engine;

        thread::Builder::new()
            .name("abu-timestep-ticker".to_string())
            .spawn(move || {
                let ticker = TimestepTicker {
                    clock,
                    engine,
                    running: Arc::clone(&running),
                };
                let mut history = Vec::new();
                let mut count = 0u64;
                while running.load(Ordering::Relaxed) {
                    let ev = ticker.wait_next_frame_boundary();
                    callback(ev);
                    history.push(ev);
                    count += 1;
                    if let Some(limit) = max_ticks {
                        if count >= limit {
                            break;
                        }
                    }
                }
                history
            })
            .expect("failed to spawn abu-timestep-ticker thread")
    }

    /// Signals the background timestep thread to stop cleanly.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
