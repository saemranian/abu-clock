//! Standard 29-byte FreeD (`0xD1`) and Extended 41-byte AbuTrack FreeD-TC (`0xD2`)
//! UDP Telemetry Packet Serializers.
//!
//! - Standard FreeD (`0xD1`): 29 bytes fixed, embeds `uint16` rolling frame-of-minute counter
//!   (`seconds * 60 + frame`) in bytes `26..27`, checksum `0x40 - sum(bytes[0..27]) (mod 256)` at byte `28`.
//! - Extended FreeD-TC (`0xD2`): 41 bytes fixed, appends `uint32` SMPTE ST 12-1 BCD Timecode at
//!   bytes `28..31`, `uint64` TAI Nanoseconds at bytes `32..39`, checksum `0x40 - sum(bytes[0..39]) (mod 256)` at byte `40`.

use abu_clock_core::SmpteTimecode;
use serde::{Deserialize, Serialize};

use super::opentrackio::CameraPose6DoF;

/// Command identifier for Standard 29-byte FreeD position/orientation message (`0xD1`).
pub const FREED_STANDARD_ID: u8 = 0xD1;
/// Command identifier for Extended 41-byte AbuTrack FreeD-TC message (`0xD2`).
pub const FREED_EXTENDED_TC_ID: u8 = 0xD2;

/// Fixed byte length of a Standard FreeD (`0xD1`) packet.
pub const FREED_STANDARD_LEN: usize = 29;
/// Fixed byte length of an `Abu Track & Scan` 37-byte Extended FreeD (`0xD1` + 8-byte `int64` ms) packet.
pub const FREED_ABUTRACK_37_LEN: usize = 37;
/// Fixed byte length of an Extended FreeD-TC (`0xD2`) packet.
pub const FREED_EXTENDED_TC_LEN: usize = 41;

/// Encodes a signed angle (in degrees) as a 24-bit big-endian FreeD fixed-point integer (`angle * 32768`).
#[inline]
fn encode_angle_24bit(deg: f64) -> [u8; 3] {
    let raw = (deg * 32_768.0).round() as i32;
    let b = raw.to_be_bytes();
    [b[1], b[2], b[3]]
}

/// Decodes a 24-bit big-endian signed FreeD angle back into degrees.
#[inline]
fn decode_angle_24bit(b: [u8; 3]) -> f64 {
    let sign_ext = if (b[0] & 0x80) != 0 { 0xFF } else { 0x00 };
    let raw = i32::from_be_bytes([sign_ext, b[0], b[1], b[2]]);
    (raw as f64) / 32_768.0
}

/// Encodes a signed position (in millimeters) as a 24-bit big-endian FreeD fixed-point integer (`pos_mm * 64`).
#[inline]
fn encode_pos_24bit(mm: f64) -> [u8; 3] {
    let raw = (mm * 64.0).round() as i32;
    let b = raw.to_be_bytes();
    [b[1], b[2], b[3]]
}

/// Decodes a 24-bit big-endian signed FreeD position back into millimeters.
#[inline]
fn decode_pos_24bit(b: [u8; 3]) -> f64 {
    let sign_ext = if (b[0] & 0x80) != 0 { 0xFF } else { 0x00 };
    let raw = i32::from_be_bytes([sign_ext, b[0], b[1], b[2]]);
    (raw as f64) / 64.0
}

/// Encodes an unsigned 24-bit lens encoder value (`Zoom` / `Focus`).
#[inline]
fn encode_u24(val: u32) -> [u8; 3] {
    let b = val.to_be_bytes();
    [b[1], b[2], b[3]]
}

/// Decodes an unsigned 24-bit lens encoder value.
#[inline]
fn decode_u24(b: [u8; 3]) -> u32 {
    u32::from_be_bytes([0, b[0], b[1], b[2]])
}

/// Computes the standard FreeD modulo-256 checksum: `0x40 - sum(bytes) (mod 256)`.
#[inline]
pub fn freed_checksum(prefix: &[u8]) -> u8 {
    let mut acc: u8 = 0x40;
    for &b in prefix {
        acc = acc.wrapping_sub(b);
    }
    acc
}

/// Standard 29-byte FreeD (`0xD1`) and Extended 41-byte FreeD-TC (`0xD2`) telemetry frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FreeDPacket {
    pub pose: CameraPose6DoF,
    pub timecode: SmpteTimecode,
    pub tai_ns: u64,
}

impl FreeDPacket {
    /// Creates a new `FreeDPacket` from a 6DoF camera pose, SMPTE ST 12-1 timecode, and TAI timestamp.
    pub const fn new(pose: CameraPose6DoF, timecode: SmpteTimecode, tai_ns: u64) -> Self {
        Self {
            pose,
            timecode,
            tai_ns,
        }
    }

    /// Returns the 16-bit rolling frame-of-minute counter (`seconds * 60 + frame`) embedded
    /// into spare bytes `26..27` of the FreeD payload.
    #[inline]
    pub const fn rolling_frame_counter(&self) -> u16 {
        (self.timecode.seconds as u16) * 60 + (self.timecode.frames as u16)
    }

    /// Writes the common 26-byte body (Bytes `2..27`: Pan, Tilt, Roll, X, Y, Z, Zoom, Focus, Rolling Counter).
    fn write_pose_body(&self, buf: &mut [u8]) {
        buf[2..5].copy_from_slice(&encode_angle_24bit(self.pose.pan_deg));
        buf[5..8].copy_from_slice(&encode_angle_24bit(self.pose.tilt_deg));
        buf[8..11].copy_from_slice(&encode_angle_24bit(self.pose.roll_deg));
        buf[11..14].copy_from_slice(&encode_pos_24bit(self.pose.x_mm));
        buf[14..17].copy_from_slice(&encode_pos_24bit(self.pose.y_mm));
        buf[17..20].copy_from_slice(&encode_pos_24bit(self.pose.z_mm));
        buf[20..23].copy_from_slice(&encode_u24(self.pose.zoom_raw));
        buf[23..26].copy_from_slice(&encode_u24(self.pose.focus_raw));
        buf[26..28].copy_from_slice(&self.rolling_frame_counter().to_be_bytes());
    }

    /// Serializes a Standard 29-byte FreeD (`0xD1`) packet.
    pub fn to_standard_d1_bytes(&self) -> [u8; FREED_STANDARD_LEN] {
        let mut out = [0u8; FREED_STANDARD_LEN];
        out[0] = FREED_STANDARD_ID;
        out[1] = self.pose.camera_id;
        self.write_pose_body(&mut out);
        out[28] = freed_checksum(&out[0..28]);
        out
    }

    /// Serializes an Extended 41-byte AbuTrack FreeD-TC (`0xD2`) packet appending the 32-bit
    /// SMPTE ST 12-1 BCD timecode (bytes `28..31`) and 64-bit TAI nanoseconds (bytes `32..39`).
    pub fn to_extended_d2_bytes(&self) -> [u8; FREED_EXTENDED_TC_LEN] {
        let mut out = [0u8; FREED_EXTENDED_TC_LEN];
        out[0] = FREED_EXTENDED_TC_ID;
        out[1] = self.pose.camera_id;
        self.write_pose_body(&mut out);
        out[28..32].copy_from_slice(&self.timecode.to_bcd_u32().to_be_bytes());
        out[32..40].copy_from_slice(&self.tai_ns.to_be_bytes());
        out[40] = freed_checksum(&out[0..40]);
        out
    }

    /// Parses a Standard 29-byte FreeD (`0xD1`) packet and verifies its `0x40 - sum` checksum.
    pub fn from_standard_d1_bytes(buf: &[u8]) -> Result<(CameraPose6DoF, u16), &'static str> {
        if buf.len() != FREED_STANDARD_LEN {
            return Err("Standard FreeD (0xD1) packet must be exactly 29 bytes");
        }
        if buf[0] != FREED_STANDARD_ID {
            return Err("Invalid FreeD command ID (expected 0xD1)");
        }
        if buf[28] != freed_checksum(&buf[0..28]) {
            return Err("Standard FreeD (0xD1) checksum mismatch");
        }
        let pose = CameraPose6DoF {
            camera_id: buf[1],
            pan_deg: decode_angle_24bit([buf[2], buf[3], buf[4]]),
            tilt_deg: decode_angle_24bit([buf[5], buf[6], buf[7]]),
            roll_deg: decode_angle_24bit([buf[8], buf[9], buf[10]]),
            x_mm: decode_pos_24bit([buf[11], buf[12], buf[13]]),
            y_mm: decode_pos_24bit([buf[14], buf[15], buf[16]]),
            z_mm: decode_pos_24bit([buf[17], buf[18], buf[19]]),
            zoom_raw: decode_u24([buf[20], buf[21], buf[22]]),
            focus_raw: decode_u24([buf[23], buf[24], buf[25]]),
        };
        let rolling = u16::from_be_bytes([buf[26], buf[27]]);
        Ok((pose, rolling))
    }

    /// Serializes a 37-byte `Abu Track & Scan` Extended FreeD (`0xD1` + 8-byte big-endian `int64` ms)
    /// packet matching `UDPStreamingManager.swift`, `camtrack_desktop_gui.py`, and `LiveLinkAbuTrackSource.cpp`.
    pub fn to_abutrack_37b_bytes(&self) -> [u8; FREED_ABUTRACK_37_LEN] {
        let mut out = [0u8; FREED_ABUTRACK_37_LEN];
        let d1 = self.to_standard_d1_bytes();
        out[0..29].copy_from_slice(&d1);
        let ts_ms = (self.tai_ns / 1_000_000) as i64;
        out[29..37].copy_from_slice(&ts_ms.to_be_bytes());
        out
    }

    /// Parses a 37-byte `Abu Track & Scan` Extended FreeD (`0xD1` + 8-byte big-endian `int64` ms) packet.
    pub fn from_abutrack_37b_bytes(buf: &[u8]) -> Result<(CameraPose6DoF, u16, i64), &'static str> {
        if buf.len() != FREED_ABUTRACK_37_LEN {
            return Err("AbuTrack 37-byte Extended FreeD packet must be exactly 37 bytes");
        }
        let (pose, rolling) = Self::from_standard_d1_bytes(&buf[0..29])?;
        let ts_ms = i64::from_be_bytes([
            buf[29], buf[30], buf[31], buf[32], buf[33], buf[34], buf[35], buf[36],
        ]);
        Ok((pose, rolling, ts_ms))
    }

    /// Parses an Extended 41-byte AbuTrack FreeD-TC (`0xD2`) packet and verifies its checksum.
    pub fn from_extended_d2_bytes(buf: &[u8], drop_frame: bool) -> Result<Self, &'static str> {
        if buf.len() != FREED_EXTENDED_TC_LEN {
            return Err("Extended FreeD-TC (0xD2) packet must be exactly 41 bytes");
        }
        if buf[0] != FREED_EXTENDED_TC_ID {
            return Err("Invalid Extended FreeD command ID (expected 0xD2)");
        }
        if buf[40] != freed_checksum(&buf[0..40]) {
            return Err("Extended FreeD-TC (0xD2) checksum mismatch");
        }
        let pose = CameraPose6DoF {
            camera_id: buf[1],
            pan_deg: decode_angle_24bit([buf[2], buf[3], buf[4]]),
            tilt_deg: decode_angle_24bit([buf[5], buf[6], buf[7]]),
            roll_deg: decode_angle_24bit([buf[8], buf[9], buf[10]]),
            x_mm: decode_pos_24bit([buf[11], buf[12], buf[13]]),
            y_mm: decode_pos_24bit([buf[14], buf[15], buf[16]]),
            z_mm: decode_pos_24bit([buf[17], buf[18], buf[19]]),
            zoom_raw: decode_u24([buf[20], buf[21], buf[22]]),
            focus_raw: decode_u24([buf[23], buf[24], buf[25]]),
        };
        let bcd = u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]);
        let timecode = SmpteTimecode::from_bcd_u32(bcd, drop_frame);
        let tai_ns = u64::from_be_bytes([
            buf[32], buf[33], buf[34], buf[35], buf[36], buf[37], buf[38], buf[39],
        ]);
        Ok(Self {
            pose,
            timecode,
            tai_ns,
        })
    }
}
