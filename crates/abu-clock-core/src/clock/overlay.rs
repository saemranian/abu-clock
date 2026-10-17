//! `OverlayClock` virtual Phase-Locked Loop (PLL) and `SharedClock` state sharing.
//!
//! Computes International Atomic Time (TAI) nanoseconds in user-space without mutating
//! the host operating system's global kernel clock:
//!
//! $$T_{\text{TAI}}(t) = T_{\text{anchor}} + (C_{\text{mono}}(t) - C_{\text{mono}}(t_0)) \cdot (1 + \alpha_{\text{freq}}) + \theta_{\text{phase}}$$

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use super::metrics::{ClockConvergenceMetrics, LockState, WARMUP_GATE_NS};
use super::monotonic::PlatformMonotonicClock;

/// Thread-safe handle to a shared `OverlayClock` instance across timing, networking, and UI threads.
pub type SharedClock = Arc<OverlayClock>;

/// Two-state Kalman filter (`[offset_ns, drift_ppb]`) modeled after Project Pendulum's `statime`
/// automatic clock servo. Estimates phase offset and crystal frequency drift without manual PI tuning.
#[derive(Debug, Clone)]
pub struct ClockKalmanFilter {
    /// State vector: `x[0] = offset_ns`, `x[1] = drift_ppb`
    x_offset_ns: f64,
    x_drift_ppb: f64,
    /// 2x2 error covariance matrix `[p00, p01, p10, p11]`
    p00: f64,
    p01: f64,
    p10: f64,
    p11: f64,
    /// Process noise spectral densities
    q_phase_ns2_per_s: f64,
    q_freq_ppb2_per_s: f64,
    /// Measurement noise variance ($\text{ns}^2$)
    r_meas_ns2: f64,
    last_step_mono_ns: Option<u64>,
}

impl Default for ClockKalmanFilter {
    fn default() -> Self {
        Self::new(250.0, 2.0, 4_000.0)
    }
}

impl ClockKalmanFilter {
    /// Creates a new two-state Kalman clock filter.
    pub fn new(q_phase_ns2_per_s: f64, q_freq_ppb2_per_s: f64, r_meas_ns2: f64) -> Self {
        Self {
            x_offset_ns: 0.0,
            x_drift_ppb: 0.0,
            p00: 1.0e12,
            p01: 0.0,
            p10: 0.0,
            p11: 1.0e6,
            q_phase_ns2_per_s,
            q_freq_ppb2_per_s,
            r_meas_ns2,
            last_step_mono_ns: None,
        }
    }

    /// Feeds a measured master-to-follower phase offset (`measured_offset_ns`) taken at `mono_ns`
    /// and returns the filtered `(offset_ns, drift_ppb, variance_ns2)`.
    pub fn ingest_measurement(
        &mut self,
        measured_offset_ns: f64,
        mono_ns: u64,
        measurement_noise_ns2: Option<f64>,
    ) -> (f64, f64, f64) {
        let dt_s = match self.last_step_mono_ns {
            Some(prev) if mono_ns > prev => ((mono_ns - prev) as f64) * 1.0e-9,
            _ => 0.125, // Nominal 8 Hz sync interval on first sample
        };
        self.last_step_mono_ns = Some(mono_ns);

        // 1. State prediction:
        // x_offset_pred = x_offset + x_drift * dt_s
        // x_drift_pred  = x_drift
        let pred_offset = self.x_offset_ns + self.x_drift_ppb * dt_s;
        let pred_drift = self.x_drift_ppb;

        // Covariance prediction: P' = F P F^T + Q, where F = [[1, dt], [0, 1]]
        let q00 = self.q_phase_ns2_per_s * dt_s;
        let q11 = self.q_freq_ppb2_per_s * dt_s;

        let p00 = self.p00 + dt_s * (self.p10 + self.p01) + dt_s * dt_s * self.p11 + q00;
        let p01 = self.p01 + dt_s * self.p11;
        let p10 = self.p10 + dt_s * self.p11;
        let p11 = self.p11 + q11;

        // 2. Innovation (measurement residual y = z - H x_pred, H = [1, 0])
        let y = measured_offset_ns - pred_offset;
        let r = measurement_noise_ns2.unwrap_or(self.r_meas_ns2).max(1.0);
        let s = p00 + r;

        // 3. Kalman gain K = P' H^T S^-1 = [p00 / s, p10 / s]^T
        let k0 = p00 / s;
        let k1 = p10 / s;

        self.x_offset_ns = pred_offset + k0 * y;
        self.x_drift_ppb = pred_drift + k1 * y;

        // 4. Covariance update: P = (I - K H) P'
        self.p00 = (1.0 - k0) * p00;
        self.p01 = (1.0 - k0) * p01;
        self.p10 = p10 - k1 * p00;
        self.p11 = p11 - k1 * p01;

        (self.x_offset_ns, self.x_drift_ppb, self.p00)
    }

    /// Applies closed-loop phase and frequency steering controls (`u_phase_ns`, `u_drift_ppb`)
    /// that were injected into the hardware/virtual oscillator.
    pub fn apply_steering_control(&mut self, applied_phase_ns: f64, applied_drift_ppb: f64) {
        self.x_offset_ns -= applied_phase_ns;
        self.x_drift_ppb -= applied_drift_ppb;
    }

    /// Resets the filter state (e.g. after a coarse clock step).
    pub fn reset_phase(&mut self) {
        self.x_offset_ns = 0.0;
    }
}

/// Frequency-locked virtual clock wrapping `PlatformMonotonicClock` (`statime::OverlayClock` architecture).
///
/// Uses lock-free atomic sequence locking on the hot reader path (`now_tai_ns` and
/// `translate_monotonic_to_tai`) so high-rate timestep spinlocks and camera frame-timestamp
/// converters experience zero mutex contention.
pub struct OverlayClock {
    /// Even value = stable, odd value = writer updating calibration parameters.
    seq: AtomicU64,
    /// Synchronized TAI epoch anchor $T_{\text{anchor}}$ in nanoseconds.
    t_anchor_tai_ns: AtomicU64,
    /// Monotonic counter reading $C_{\text{mono}}(t_0)$ at the anchor instant.
    c_mono_t0_ns: AtomicU64,
    /// Frequency drift ratio in fixed-point parts-per-trillion ($\alpha_{\text{freq}} \times 10^{12}$)
    /// for sub-ppb atomic precision without floating-point atomics.
    alpha_freq_ppt: AtomicI64,
    /// Residual phase adjustment $\theta_{\text{phase}}$ in nanoseconds.
    theta_phase_ns: AtomicI64,
    /// Monotonic timestamp when continuous convergence started.
    convergence_start_mono_ns: AtomicU64,
    /// Protected Kalman filter and telemetry state for network servo updates.
    servo_state: RwLock<(ClockKalmanFilter, ClockConvergenceMetrics)>,
}

impl OverlayClock {
    /// Creates a new `OverlayClock` anchored at `initial_tai_ns` against the current OS monotonic counter.
    pub fn new(initial_tai_ns: u64) -> Self {
        let mono_now = PlatformMonotonicClock::now_ns();
        Self::with_monotonic_anchor(initial_tai_ns, mono_now)
    }

    /// Creates a new `OverlayClock` with an explicit initial TAI anchor and monotonic reference point.
    pub fn with_monotonic_anchor(initial_tai_ns: u64, c_mono_t0_ns: u64) -> Self {
        Self {
            seq: AtomicU64::new(0),
            t_anchor_tai_ns: AtomicU64::new(initial_tai_ns),
            c_mono_t0_ns: AtomicU64::new(c_mono_t0_ns),
            alpha_freq_ppt: AtomicI64::new(0),
            theta_phase_ns: AtomicI64::new(0),
            convergence_start_mono_ns: AtomicU64::new(c_mono_t0_ns),
            servo_state: RwLock::new((
                ClockKalmanFilter::default(),
                ClockConvergenceMetrics::default(),
            )),
        }
    }

    /// Creates a new reference-counted `SharedClock`.
    pub fn new_shared(initial_tai_ns: u64) -> SharedClock {
        Arc::new(Self::new(initial_tai_ns))
    }

    /// Reads the instantaneous disciplined International Atomic Time (TAI) in nanoseconds.
    #[inline]
    pub fn now_tai_ns(&self) -> u64 {
        let mono_now = PlatformMonotonicClock::now_ns();
        self.translate_monotonic_to_tai(mono_now)
    }

    /// Translates an arbitrary OS monotonic hardware timestamp $C_{\text{mono}}(t)$ (such as
    /// iOS `CMSampleBuffer` / `ARFrame.timestamp` or Windows QPC) into disciplined TAI nanoseconds:
    ///
    /// $$T_{\text{TAI}}(t) = T_{\text{anchor}} + (C_{\text{mono}}(t) - C_{\text{mono}}(t_0)) \cdot (1 + \alpha_{\text{freq}}) + \theta_{\text{phase}}$$
    #[inline]
    pub fn translate_monotonic_to_tai(&self, c_mono_t_ns: u64) -> u64 {
        let (t_anchor, c_t0, alpha_ppt, theta_ns) = self.read_calibration_snapshot();
        let delta_mono = (c_mono_t_ns as i128) - (c_t0 as i128);
        // Scale delta_mono by (1 + alpha_ppt * 1e-12)
        let freq_correction = (delta_mono * (alpha_ppt as i128)) / 1_000_000_000_000i128;
        let disciplined = (t_anchor as i128) + delta_mono + freq_correction + (theta_ns as i128);
        disciplined.max(0) as u64
    }

    /// Lock-free seqlock reader returning `(t_anchor_tai_ns, c_mono_t0_ns, alpha_freq_ppt, theta_phase_ns)`.
    #[inline]
    fn read_calibration_snapshot(&self) -> (u64, u64, i64, i64) {
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            if s1 & 1 != 0 {
                core::hint::spin_loop();
                continue;
            }
            let t_anchor = self.t_anchor_tai_ns.load(Ordering::Relaxed);
            let c_t0 = self.c_mono_t0_ns.load(Ordering::Relaxed);
            let alpha_ppt = self.alpha_freq_ppt.load(Ordering::Relaxed);
            let theta_ns = self.theta_phase_ns.load(Ordering::Relaxed);
            let s2 = self.seq.load(Ordering::Acquire);
            if s1 == s2 {
                return (t_anchor, c_t0, alpha_ppt, theta_ns);
            }
        }
    }

    /// Steers the virtual oscillator frequency in parts-per-billion ($\alpha_{\text{freq}}$ in ppb)
    /// while re-anchoring $(T_{\text{anchor}}, C_{\text{mono}}(t_0))$ to prevent phase discontinuities.
    pub fn steer_frequency_ppb(&self, drift_ppb: f64) {
        let mono_now = PlatformMonotonicClock::now_ns();
        self.steer_frequency_ppb_at(drift_ppb, mono_now);
    }

    /// Steers the virtual oscillator frequency at an explicit monotonic timestamp `mono_now_ns`.
    pub fn steer_frequency_ppb_at(&self, drift_ppb: f64, mono_now_ns: u64) {
        let current_tai = self.translate_monotonic_to_tai(mono_now_ns);
        let new_alpha_ppt = (drift_ppb * 1_000.0).round() as i64;

        let s = self.seq.fetch_add(1, Ordering::AcqRel);
        debug_assert_eq!(s & 1, 0);
        self.t_anchor_tai_ns.store(current_tai, Ordering::Relaxed);
        self.c_mono_t0_ns.store(mono_now_ns, Ordering::Relaxed);
        self.alpha_freq_ppt.store(new_alpha_ppt, Ordering::Relaxed);
        self.theta_phase_ns.store(0, Ordering::Relaxed);
        self.seq.fetch_add(1, Ordering::Release);

        if let Ok(mut guard) = self.servo_state.write() {
            guard.1.drift_ppb = drift_ppb;
        }
    }

    /// Applies a residual phase adjustment $\theta_{\text{phase}}$ (in nanoseconds).
    pub fn adjust_phase_ns(&self, theta_phase_ns: i64) {
        let s = self.seq.fetch_add(1, Ordering::AcqRel);
        debug_assert_eq!(s & 1, 0);
        self.theta_phase_ns.store(theta_phase_ns, Ordering::Relaxed);
        self.seq.fetch_add(1, Ordering::Release);
    }

    /// Performs a coarse phase step to a new TAI anchor at `mono_now_ns`.
    pub fn step_to_tai_ns(&self, target_tai_ns: u64, mono_now_ns: u64) {
        let s = self.seq.fetch_add(1, Ordering::AcqRel);
        debug_assert_eq!(s & 1, 0);
        self.t_anchor_tai_ns.store(target_tai_ns, Ordering::Relaxed);
        self.c_mono_t0_ns.store(mono_now_ns, Ordering::Relaxed);
        self.theta_phase_ns.store(0, Ordering::Relaxed);
        self.seq.fetch_add(1, Ordering::Release);
    }

    /// Disciplines the `OverlayClock` using a PTP/CSPTP phase error measurement (`offset_from_master_ns`,
    /// defined as $T_{\text{master}} - T_{\text{local}}$) and network `path_delay_ns` at `mono_now_ns`.
    pub fn discipline_from_network_sample(
        &self,
        offset_from_master_ns: i64,
        path_delay_ns: u64,
        mono_now_ns: u64,
        is_wireless: bool,
    ) -> ClockConvergenceMetrics {
        let mut guard = self.servo_state.write().expect("servo lock poisoned");
        let (ref mut kf, ref mut metrics) = *guard;

        // If offset exceeds 10 ms on initial boot, perform a coarse step and reset the warm-up timer.
        if offset_from_master_ns.saturating_abs() > 10_000_000 {
            let current_local = self.translate_monotonic_to_tai(mono_now_ns);
            let target_tai = (current_local as i128 + offset_from_master_ns as i128).max(0) as u64;
            self.step_to_tai_ns(target_tai, mono_now_ns);
            self.convergence_start_mono_ns
                .store(mono_now_ns, Ordering::Release);
            kf.reset_phase();
        }

        let (filtered_offset, filtered_drift_delta_ppb, variance_ns2) =
            kf.ingest_measurement(offset_from_master_ns as f64, mono_now_ns, None);

        // Apply closed-loop phase correction + cumulative Kalman drift steering
        let current_tai = self.translate_monotonic_to_tai(mono_now_ns);
        let phase_correction_ns = filtered_offset.round() as i64;
        let corrected_anchor = (current_tai as i128 + phase_correction_ns as i128).max(0) as u64;
        let total_drift_ppb = metrics.drift_ppb + filtered_drift_delta_ppb;
        let new_alpha_ppt = (total_drift_ppb * 1_000.0).round() as i64;

        let s = self.seq.fetch_add(1, Ordering::AcqRel);
        debug_assert_eq!(s & 1, 0);
        self.t_anchor_tai_ns
            .store(corrected_anchor, Ordering::Relaxed);
        self.c_mono_t0_ns.store(mono_now_ns, Ordering::Relaxed);
        self.alpha_freq_ppt.store(new_alpha_ppt, Ordering::Relaxed);
        self.theta_phase_ns.store(0, Ordering::Relaxed);
        self.seq.fetch_add(1, Ordering::Release);

        kf.apply_steering_control(phase_correction_ns as f64, filtered_drift_delta_ppb);

        let start_mono = self.convergence_start_mono_ns.load(Ordering::Acquire);
        let elapsed_lock_ns = mono_now_ns.saturating_sub(start_mono);
        let residual_offset_ns = offset_from_master_ns - phase_correction_ns;
        let warmup_progress =
            ((elapsed_lock_ns as f64) / (WARMUP_GATE_NS as f64)).clamp(0.0, 1.0);
        let lock_state = LockState::evaluate(residual_offset_ns, elapsed_lock_ns, is_wireless);

        *metrics = ClockConvergenceMetrics {
            offset_ns: residual_offset_ns,
            drift_ppb: total_drift_ppb,
            path_delay_ns,
            offset_variance_ns2: variance_ns2,
            elapsed_lock_ns,
            warmup_progress,
            is_wireless,
            lock_state,
        };
        *metrics
    }

    /// Directly updates convergence telemetry (useful when driven by external hardware PHC status).
    pub fn update_telemetry(
        &self,
        offset_ns: i64,
        drift_ppb: f64,
        path_delay_ns: u64,
        elapsed_lock_ns: u64,
        is_wireless: bool,
    ) -> ClockConvergenceMetrics {
        let mut guard = self.servo_state.write().expect("servo lock poisoned");
        let warmup_progress =
            ((elapsed_lock_ns as f64) / (WARMUP_GATE_NS as f64)).clamp(0.0, 1.0);
        let lock_state = LockState::evaluate(offset_ns, elapsed_lock_ns, is_wireless);
        guard.1 = ClockConvergenceMetrics {
            offset_ns,
            drift_ppb,
            path_delay_ns,
            offset_variance_ns2: guard.1.offset_variance_ns2,
            elapsed_lock_ns,
            warmup_progress,
            is_wireless,
            lock_state,
        };
        guard.1
    }

    /// Returns a snapshot of the current convergence metrics and lock state.
    pub fn metrics(&self) -> ClockConvergenceMetrics {
        self.servo_state
            .read()
            .map(|g| g.1)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_clock_discipline_equation_matches_spec() {
        let t_anchor = 1_700_000_000_000_000_000u64;
        let c_t0 = 1_000_000_000u64;
        let clock = OverlayClock::with_monotonic_anchor(t_anchor, c_t0);

        // Steer +25,000 ppb (+25 ppm) at t0 and add +400 ns residual phase
        clock.steer_frequency_ppb_at(25_000.0, c_t0);
        clock.adjust_phase_ns(400);

        // 1 second of monotonic counter elapsed:
        // Expected TAI = t_anchor + 1_000_000_000 + 25_000 + 400
        let c_t1 = c_t0 + 1_000_000_000;
        let tai_t1 = clock.translate_monotonic_to_tai(c_t1);
        assert_eq!(tai_t1, t_anchor + 1_000_025_400);
    }

    #[test]
    fn kalman_servo_converges_to_locked_after_7s_warmup() {
        let gm_anchor = 1_729_180_000_000_000_000u64;
        let c_t0 = 5_000_000_000u64;
        let clock = OverlayClock::with_monotonic_anchor(gm_anchor, c_t0);

        // Simulate 8 Hz (125 ms) sync messages over 8 seconds with synthetic +15 ppm crystal drift
        let mut last_metrics = ClockConvergenceMetrics::default();
        for step in 1..=64 {
            let mono_ns = c_t0 + step * 125_000_000;
            // True master time has +15,000 ns per second relative to unsteered local crystal
            let true_master_tai =
                gm_anchor + step * 125_000_000 + (step * 125_000_000 * 15_000) / 1_000_000_000;
            let local_tai = clock.translate_monotonic_to_tai(mono_ns);
            let measured_offset_ns = (true_master_tai as i128 - local_tai as i128) as i64;
            last_metrics =
                clock.discipline_from_network_sample(measured_offset_ns, 420, mono_ns, false);

            if mono_ns - c_t0 < WARMUP_GATE_NS {
                assert_ne!(
                    last_metrics.lock_state,
                    LockState::Locked,
                    "Must not report Locked before 7.0s warm-up gate"
                );
            }
        }

        assert_eq!(last_metrics.lock_state, LockState::Locked);
        assert!(
            last_metrics.offset_ns.abs() < 100,
            "Expected sub-100ns lock after 8s, got {} ns",
            last_metrics.offset_ns
        );
    }
}
