//! SMPTE ST 2059-2 Synchronization Metadata (SM) Organization Extension TLV.
//!
//! Appended to PTP `Announce` frames (`tlvType = 0x0003`, Organization ID `0x000E9B`,
//! Subtype `0x000001`) so follower nodes automatically lock to the Grandmaster's
//! rational frame rate, drop-frame flag, and leap-second configuration.

use abu_clock_core::{FrameRate, DEFAULT_TAI_MINUS_UTC_SECONDS};
use serde::{Deserialize, Serialize};

/// IEEE 1588 Organization Extension TLV Type (`0x0003`).
pub const TLV_TYPE_ORGANIZATION_EXTENSION: u16 = 0x0003;

/// SMPTE Organization Unique Identifier (`00-0E-9B`).
pub const SMPTE_ORGANIZATION_ID: [u8; 3] = [0x00, 0x0E, 0x9B];

/// SMPTE ST 2059-2 Synchronization Metadata Organization Subtype (`0x000001`).
pub const SMPTE_ST2059_SUBTYPE: [u8; 3] = [0x00, 0x00, 0x01];

/// Wire-serializable SMPTE ST 2059-2 Synchronization Metadata (SM) TLV payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmpteSyncMetadataTlv {
    /// Current TAI minus UTC offset in seconds (e.g. `37`).
    pub current_utc_offset_sec: i16,
    /// Rational frame rate numerator ($F_{\text{num}}$).
    pub default_system_frame_rate_num: u32,
    /// Rational frame rate denominator ($F_{\text{den}}$).
    pub default_system_frame_rate_den: u32,
    /// Timecode flags: bit 0 = Drop-Frame (`1` = DF, `0` = NDF), bit 1 = Color Frame.
    pub timecode_flags: u8,
}

impl SmpteSyncMetadataTlv {
    /// Fixed serialized wire size of the SMPTE SM TLV (20 bytes).
    pub const WIRE_SIZE: usize = 20;

    /// Constructs a new `SmpteSyncMetadataTlv` from an `abu-clock-core` `FrameRate`.
    pub const fn from_frame_rate(rate: FrameRate) -> Self {
        Self {
            current_utc_offset_sec: DEFAULT_TAI_MINUS_UTC_SECONDS as i16,
            default_system_frame_rate_num: rate.numerator,
            default_system_frame_rate_den: rate.denominator,
            timecode_flags: if rate.drop_frame { 0x01 } else { 0x00 },
        }
    }

    /// Converts the TLV metadata back into an `abu-clock-core` `FrameRate`.
    pub const fn to_frame_rate(&self) -> FrameRate {
        FrameRate::new(
            self.default_system_frame_rate_num,
            self.default_system_frame_rate_den,
            (self.timecode_flags & 0x01) != 0,
        )
    }

    /// Serializes the SMPTE ST 2059-2 SM TLV into a 20-byte big-endian wire buffer.
    pub fn to_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut out = [0u8; Self::WIRE_SIZE];
        // Bytes 0..2: tlvType (0x0003)
        out[0..2].copy_from_slice(&TLV_TYPE_ORGANIZATION_EXTENSION.to_be_bytes());
        // Bytes 2..4: lengthField (16 bytes following lengthField)
        out[2..4].copy_from_slice(&16u16.to_be_bytes());
        // Bytes 4..7: organizationId (00-0E-9B)
        out[4..7].copy_from_slice(&SMPTE_ORGANIZATION_ID);
        // Bytes 7..10: organizationSubType (00-00-01)
        out[7..10].copy_from_slice(&SMPTE_ST2059_SUBTYPE);
        // Bytes 10..12: currentUtcOffset
        out[10..12].copy_from_slice(&self.current_utc_offset_sec.to_be_bytes());
        // Bytes 12..16: defaultSystemFrameRate numerator
        out[12..16].copy_from_slice(&self.default_system_frame_rate_num.to_be_bytes());
        // Bytes 16..19: defaultSystemFrameRate denominator (24-bit) + Byte 19: timecodeFlags
        let den_bytes = self.default_system_frame_rate_den.to_be_bytes();
        out[16..19].copy_from_slice(&den_bytes[1..4]);
        out[19] = self.timecode_flags;
        out
    }

    /// Parses a 20-byte big-endian SMPTE ST 2059-2 SM TLV slice.
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::WIRE_SIZE {
            return None;
        }
        let tlv_type = u16::from_be_bytes([buf[0], buf[1]]);
        let length = u16::from_be_bytes([buf[2], buf[3]]);
        if tlv_type != TLV_TYPE_ORGANIZATION_EXTENSION || length < 16 {
            return None;
        }
        if buf[4..7] != SMPTE_ORGANIZATION_ID || buf[7..10] != SMPTE_ST2059_SUBTYPE {
            return None;
        }
        let current_utc_offset_sec = i16::from_be_bytes([buf[10], buf[11]]);
        let default_system_frame_rate_num =
            u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let default_system_frame_rate_den = u32::from_be_bytes([0, buf[16], buf[17], buf[18]]);
        let timecode_flags = buf[19];

        Some(Self {
            current_utc_offset_sec,
            default_system_frame_rate_num,
            default_system_frame_rate_den,
            timecode_flags,
        })
    }
}
