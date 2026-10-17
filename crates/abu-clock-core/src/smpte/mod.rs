//! SMPTE ST 2059-1 / ST 12-1 timecode synthesis and deterministic timestep (`Hz`) generation.

pub mod epoch;
pub mod rate;
pub mod ticker;
pub mod timecode;

pub use epoch::{
    FrameSynthesisSnapshot, PtpTimestamp80, SmpteTimecodeEngine, DEFAULT_TAI_MINUS_UTC_SECONDS,
};
pub use rate::FrameRate;
pub use ticker::{TimestepTickEvent, TimestepTicker, SPINLOCK_GUARD_WINDOW_NS};
pub use timecode::SmpteTimecode;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::OverlayClock;

    #[test]
    fn smpte_st12_drop_frame_rollover_rules() {
        let rate = FrameRate::FPS_29_97_DF;
        // At 29.97 DF, minute 01 skips frames 00 and 01 (frame 1799 is 00:00:59;29, frame 1800 is 00:01:00;02)
        let tc_before = SmpteTimecode::from_frame_index(1_799, 0.0, rate);
        assert_eq!(tc_before.to_string(), "00:00:59;29");

        let tc_min1 = SmpteTimecode::from_frame_index(1_800, 0.0, rate);
        assert_eq!(tc_min1.to_string(), "00:01:00;02");

        // At minute 10 (every 10th minute), frames 00 and 01 are NOT skipped!
        // 10 minutes in 29.97 DF = 17,982 frames.
        let tc_before_min10 = SmpteTimecode::from_frame_index(17_981, 0.0, rate);
        assert_eq!(tc_before_min10.to_string(), "00:09:59;29");

        let tc_min10 = SmpteTimecode::from_frame_index(17_982, 0.0, rate);
        assert_eq!(tc_min10.to_string(), "00:10:00;00");
    }

    #[test]
    fn bcd_packing_and_unpacking_roundtrip() {
        let tc = SmpteTimecode {
            hours: 14,
            minutes: 22,
            seconds: 10,
            frames: 8,
            subframe: 0.45,
            drop_frame: true,
        };
        let bcd = tc.to_bcd_u32();
        assert_eq!(bcd, 0x1422_1008);

        let unpacked = SmpteTimecode::from_bcd_u32(bcd, true);
        assert_eq!(unpacked.hours, 14);
        assert_eq!(unpacked.minutes, 22);
        assert_eq!(unpacked.seconds, 10);
        assert_eq!(unpacked.frames, 8);
        assert_eq!(tc.to_compact_filename_token(), "14221008");
        assert_eq!(tc.to_slate_string(), "14:22:10;08.45");
    }

    #[test]
    fn st2059_epoch_and_80bit_timestamp_roundtrip() {
        let ts = PtpTimestamp80 {
            seconds_tai: 1_729_180_800,
            nanoseconds_tai: 500_000_000,
        };
        let bytes = ts.to_be_bytes();
        let decoded = PtpTimestamp80::from_be_bytes(bytes);
        assert_eq!(ts, decoded);

        let engine = SmpteTimecodeEngine::new(FrameRate::FPS_25, 0);
        // At 25 fps, 40_000_000 ns = 1 frame; 500_000_000 ns = 12.5 frames (frame 12, phase 0.5)
        let snap = engine.synthesize_at_tai_ns(ts.to_tai_ns());
        assert_eq!(snap.timecode.frames, 12);
        assert!((snap.phase - 0.5).abs() < 1e-9);
        assert_eq!(snap.sleep_until_next_frame_ns, 20_000_000);
    }

    #[test]
    fn spinlock_timestep_ticker_dispatches_deterministic_boundaries() {
        let clock = OverlayClock::new_shared(1_700_000_000_000_000_000);
        let engine = SmpteTimecodeEngine::new(FrameRate::FPS_120, 0);
        let ticker = TimestepTicker::new(clock, engine);

        let handle = ticker.spawn_ticker_thread(Some(3), |_| {});
        let events = handle.join().expect("ticker thread panicked");
        assert_eq!(events.len(), 3);
        assert_eq!(events[1].frame_index, events[0].frame_index + 1);
        assert_eq!(events[2].frame_index, events[1].frame_index + 1);
    }
}
