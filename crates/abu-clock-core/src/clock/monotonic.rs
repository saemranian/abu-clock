//! OS-native read-only monotonic hardware counter abstractions.
//!
//! Uses `QueryPerformanceCounter` on Windows, `mach_absolute_time` on macOS/iOS,
//! and `CLOCK_MONOTONIC_RAW` on Linux/Android with 128-bit integer scaling to
//! eliminate floating-point precision loss over multi-day broadcast uptime.

/// Provides direct access to the host platform's highest-resolution monotonic hardware counter.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlatformMonotonicClock;

impl PlatformMonotonicClock {
    /// Returns the human-readable identifier of the active OS monotonic counter backend.
    #[inline]
    pub const fn backend_name() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            "Windows::QueryPerformanceCounter"
        }
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            "Apple::mach_absolute_time"
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            "Posix::CLOCK_MONOTONIC_RAW"
        }
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "linux",
            target_os = "android"
        )))]
        {
            "Std::Instant"
        }
    }

    /// Reads the instantaneous monotonic hardware counter expressed in nanoseconds.
    #[inline]
    pub fn now_ns() -> u64 {
        imp::now_monotonic_ns()
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use std::sync::OnceLock;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn QueryPerformanceCounter(lpPerformanceCount: *mut i64) -> i32;
        fn QueryPerformanceFrequency(lpFrequency: *mut i64) -> i32;
    }

    static QPC_FREQ: OnceLock<u128> = OnceLock::new();

    #[inline]
    fn frequency() -> u128 {
        *QPC_FREQ.get_or_init(|| {
            let mut freq: i64 = 0;
            // SAFETY: QueryPerformanceFrequency always succeeds on Windows XP+.
            let ok = unsafe { QueryPerformanceFrequency(&mut freq) };
            if ok != 0 && freq > 0 {
                freq as u128
            } else {
                10_000_000u128
            }
        })
    }

    #[inline]
    pub fn now_monotonic_ns() -> u64 {
        let mut count: i64 = 0;
        // SAFETY: Passing a valid mutable pointer to stack-allocated i64.
        unsafe {
            QueryPerformanceCounter(&mut count);
        }
        let ticks = count.max(0) as u128;
        let freq = frequency();
        ((ticks * 1_000_000_000u128) / freq) as u64
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod imp {
    use std::sync::OnceLock;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct MachTimebaseInfo {
        numer: u32,
        denom: u32,
    }

    unsafe extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
    }

    static TIMEBASE: OnceLock<MachTimebaseInfo> = OnceLock::new();

    #[inline]
    pub fn now_monotonic_ns() -> u64 {
        let tb = TIMEBASE.get_or_init(|| {
            let mut info = MachTimebaseInfo { numer: 1, denom: 1 };
            unsafe {
                mach_timebase_info(&mut info);
            }
            if info.denom == 0 {
                MachTimebaseInfo { numer: 1, denom: 1 }
            } else {
                info
            }
        });
        let ticks = unsafe { mach_absolute_time() } as u128;
        ((ticks * tb.numer as u128) / tb.denom as u128) as u64
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod imp {
    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }

    const CLOCK_MONOTONIC_RAW: i32 = 4;

    unsafe extern "C" {
        fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
    }

    #[inline]
    pub fn now_monotonic_ns() -> u64 {
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let rc = unsafe { clock_gettime(CLOCK_MONOTONIC_RAW, &mut ts) };
        if rc == 0 {
            (ts.tv_sec.max(0) as u64)
                .saturating_mul(1_000_000_000)
                .saturating_add(ts.tv_nsec.max(0) as u64)
        } else {
            0
        }
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "linux",
    target_os = "android"
)))]
mod imp {
    use std::sync::OnceLock;
    use std::time::Instant;

    static EPOCH: OnceLock<Instant> = OnceLock::new();

    #[inline]
    pub fn now_monotonic_ns() -> u64 {
        let start = EPOCH.get_or_init(Instant::now);
        start.elapsed().as_nanos() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonic_counter_is_non_decreasing() {
        let t0 = PlatformMonotonicClock::now_ns();
        let mut prev = t0;
        for _ in 0..1_000 {
            let cur = PlatformMonotonicClock::now_ns();
            assert!(
                cur >= prev,
                "Monotonic clock regressed: {} < {}",
                cur,
                prev
            );
            prev = cur;
        }
        assert!(!PlatformMonotonicClock::backend_name().is_empty());
    }
}
