//! SMPTE ST 12-1 Timecode formatting, Drop-Frame rollover compensation, and 32-bit BCD packing.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::rate::FrameRate;

/// Broadcast-grade SMPTE ST 12-1 timecode value (`HH:MM:SS:FF` or `HH:MM:SS;FF` with subframe phase).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SmpteTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    /// Instantaneous fractional subframe phase $\phi \in [0.0, 1.0)$.
    pub subframe: f64,
    pub drop_frame: bool,
}

impl SmpteTimecode {
    /// Converts a continuous frame index $K_{\text{frame}}$ since the epoch into a 24-hour
    /// SMPTE ST 12-1 timecode vector (`HH:MM:SS:FF`), applying standard Drop-Frame skipping
    /// (dropping frames `00` and `01` for `29.97 DF`, or `00..03` for `59.94 DF`, at the start
    /// of every minute except every 10th minute).
    pub fn from_frame_index(frame_index: u64, subframe: f64, rate: FrameRate) -> Self {
        let nominal = rate.nominal_fps().max(1) as u64;

        if rate.drop_frame && (nominal == 30 || nominal == 60) {
            let drop_frames = if nominal == 30 { 2u64 } else { 4u64 };
            let frames_per_10_min = nominal * 60 * 10 - 9 * drop_frames; // 17982 for 29.97, 35964 for 59.94
            let frames_per_min = nominal * 60 - drop_frames; // 1798 for 29.97, 3596 for 59.94
            let frames_per_24h = frames_per_10_min * 6 * 24;

            let mut frame_in_day = frame_index % frames_per_24h;
            let d = frame_in_day / frames_per_10_min;
            let m = frame_in_day % frames_per_10_min;

            if m >= drop_frames {
                frame_in_day +=
                    drop_frames * 9 * d + drop_frames * ((m - drop_frames) / frames_per_min);
            } else {
                frame_in_day += drop_frames * 9 * d;
            }

            let frames = (frame_in_day % nominal) as u8;
            let total_seconds = frame_in_day / nominal;
            let seconds = (total_seconds % 60) as u8;
            let total_minutes = total_seconds / 60;
            let minutes = (total_minutes % 60) as u8;
            let hours = ((total_minutes / 60) % 24) as u8;

            Self {
                hours,
                minutes,
                seconds,
                frames,
                subframe: subframe.clamp(0.0, 0.999_999),
                drop_frame: true,
            }
        } else {
            let frames_per_24h = nominal * 3600 * 24;
            let frame_in_day = frame_index % frames_per_24h;

            let frames = (frame_in_day % nominal) as u8;
            let total_seconds = frame_in_day / nominal;
            let seconds = (total_seconds % 60) as u8;
            let total_minutes = total_seconds / 60;
            let minutes = (total_minutes % 60) as u8;
            let hours = ((total_minutes / 60) % 24) as u8;

            Self {
                hours,
                minutes,
                seconds,
                frames,
                subframe: subframe.clamp(0.0, 0.999_999),
                drop_frame: false,
            }
        }
    }

    /// Packs `HH:MM:SS:FF` into a 32-bit Binary-Coded Decimal (BCD) SMPTE ST 12-1 word
    /// (`0xHHMMSSFF`, with bit 30 indicating Drop-Frame when `include_flags` is used, or
    /// pure 8-digit BCD `HHMMSSFF` for Extended FreeD-TC `0xD2` transport).
    #[inline]
    pub const fn to_bcd_u32(&self) -> u32 {
        let h_bcd = (((self.hours / 10) as u32) << 4) | ((self.hours % 10) as u32);
        let m_bcd = (((self.minutes / 10) as u32) << 4) | ((self.minutes % 10) as u32);
        let s_bcd = (((self.seconds / 10) as u32) << 4) | ((self.seconds % 10) as u32);
        let f_bcd = (((self.frames / 10) as u32) << 4) | ((self.frames % 10) as u32);
        (h_bcd << 24) | (m_bcd << 16) | (s_bcd << 8) | f_bcd
    }

    /// Unpacks a 32-bit BCD SMPTE ST 12-1 word (`0xHHMMSSFF`) into `SmpteTimecode`.
    #[inline]
    pub const fn from_bcd_u32(bcd: u32, drop_frame: bool) -> Self {
        let h_byte = ((bcd >> 24) & 0xFF) as u8;
        let m_byte = ((bcd >> 16) & 0xFF) as u8;
        let s_byte = ((bcd >> 8) & 0xFF) as u8;
        let f_byte = (bcd & 0xFF) as u8;

        let hours = ((h_byte >> 4) & 0x0F) * 10 + (h_byte & 0x0F);
        let minutes = ((m_byte >> 4) & 0x0F) * 10 + (m_byte & 0x0F);
        let seconds = ((s_byte >> 4) & 0x0F) * 10 + (s_byte & 0x0F);
        let frames = ((f_byte >> 4) & 0x0F) * 10 + (f_byte & 0x0F);

        Self {
            hours,
            minutes,
            seconds,
            frames,
            subframe: 0.0,
            drop_frame,
        }
    }

    /// Formats the compact filename-safe token `HHMMSSFF` (used by Abu Scan `<SensorID>_<TC_HHMMSSFF>_<TAI_ns>.dng`).
    pub fn to_compact_filename_token(&self) -> String {
        format!(
            "{:02}{:02}{:02}{:02}",
            self.hours, self.minutes, self.seconds, self.frames
        )
    }

    /// Formats the digital slate string `HH:MM:SS:FF.sf` (or `;FF.sf` in Drop-Frame mode).
    pub fn to_slate_string(&self) -> String {
        let sep = if self.drop_frame { ';' } else { ':' };
        let sf = ((self.subframe * 100.0).floor() as u8).min(99);
        format!(
            "{:02}:{:02}:{:02}{}{:02}.{:02}",
            self.hours, self.minutes, self.seconds, sep, self.frames, sf
        )
    }
}

impl fmt::Display for SmpteTimecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = if self.drop_frame { ';' } else { ':' };
        write!(
            f,
            "{:02}:{:02}:{:02}{}{:02}",
            self.hours, self.minutes, self.seconds, sep, self.frames
        )
    }
}
