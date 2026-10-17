//! OpenTrackIO (`OTrk`) SMPTE RIS UDP Packet Serializer & Deserializer.
//!
//! Wire format:
//! - 14-byte Big-Endian Header:
//!   - Bytes `0..4`: ASCII `"OTrk"` (`0x4F54726B`)
//!   - Byte `4`: Payload Encoding (`0x01` JSON / `0x02` CBOR)
//!   - Bytes `5..7`: `uint16` Sequence Number
//!   - Bytes `7..11`: `uint32` Fragmentation Offset (`0`)
//!   - Bytes `11..13`: `uint16` Payload Length
//!   - Byte `13`: Fletcher-16 Checksum (Modulo 256) computed over header bytes `0..13` + payload.

use abu_clock_core::{FrameRate, PtpTimestamp80, SmpteTimecode};
use serde::{Deserialize, Serialize};

/// OpenTrackIO magic identifier `"OTrk"` (`0x4F54726B`).
pub const OTRK_MAGIC: [u8; 4] = *b"OTrk";

/// OpenTrackIO fixed UDP header length in bytes.
pub const OTRK_HEADER_LEN: usize = 14;

/// Payload wire encoding for OpenTrackIO (`0x01` = UTF-8 JSON, `0x02` = deterministic binary CBOR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpenTrackIoEncoding {
    Json = 0x01,
    Cbor = 0x02,
}

/// 6DoF camera pose and optical lens metadata shared by OpenTrackIO and FreeD packets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraPose6DoF {
    /// Camera identifier (`0..=255`).
    pub camera_id: u8,
    /// Pan angle in degrees (`-180.0..=180.0`).
    pub pan_deg: f64,
    /// Tilt angle in degrees (`-90.0..=90.0`).
    pub tilt_deg: f64,
    /// Roll angle in degrees (`-180.0..=180.0`).
    pub roll_deg: f64,
    /// X position in millimeters.
    pub x_mm: f64,
    /// Y position in millimeters.
    pub y_mm: f64,
    /// Z position in millimeters.
    pub z_mm: f64,
    /// Optical zoom encoder value (`0..=16_777_215`).
    pub zoom_raw: u32,
    /// Optical focus encoder value (`0..=16_777_215`).
    pub focus_raw: u32,
}

impl Default for CameraPose6DoF {
    fn default() -> Self {
        Self {
            camera_id: 1,
            pan_deg: 0.0,
            tilt_deg: 0.0,
            roll_deg: 0.0,
            x_mm: 0.0,
            y_mm: 0.0,
            z_mm: 0.0,
            zoom_raw: 1024,
            focus_raw: 2048,
        }
    }
}

/// Structured SMPTE RIS OpenTrackIO payload carrying timecode, rational frame rate,
/// TAI sample timestamp, and 6DoF camera transform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenTrackIoPayload {
    pub protocol_version: String,
    pub sample_id: String,
    pub source_id: String,
    pub timing: OpenTrackIoTiming,
    pub pose: CameraPose6DoF,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenTrackIoTiming {
    pub mode: String,
    pub sample_timestamp_tai_sec: u64,
    pub sample_timestamp_tai_nsec: u32,
    pub timecode: OpenTrackIoTimecodeField,
    pub frame_rate_num: u32,
    pub frame_rate_denom: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenTrackIoTimecodeField {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frame: u8,
    pub subframe: u16,
    pub drop_frame: bool,
}

impl OpenTrackIoPayload {
    /// Constructs an `OpenTrackIoPayload` from a disciplined TAI nanosecond timestamp,
    /// `SmpteTimecode`, `FrameRate`, and `CameraPose6DoF`.
    pub fn new(
        sequence_id: u16,
        tai_ns: u64,
        tc: SmpteTimecode,
        rate: FrameRate,
        pose: CameraPose6DoF,
    ) -> Self {
        let ptp80 = PtpTimestamp80::from_tai_ns(tai_ns);
        let (num, denom) = rate.opentrackio_rational();
        Self {
            protocol_version: "1.0.0".to_string(),
            sample_id: format!("urn:abuclock:frame:{}", sequence_id),
            source_id: format!("urn:abutrack:camera:{}", pose.camera_id),
            timing: OpenTrackIoTiming {
                mode: "PTP_SMPTE_ST2059_2".to_string(),
                sample_timestamp_tai_sec: ptp80.seconds_tai,
                sample_timestamp_tai_nsec: ptp80.nanoseconds_tai,
                timecode: OpenTrackIoTimecodeField {
                    hours: tc.hours,
                    minutes: tc.minutes,
                    seconds: tc.seconds,
                    frame: tc.frames,
                    subframe: (tc.subframe * 1000.0).round() as u16,
                    drop_frame: tc.drop_frame,
                },
                frame_rate_num: num,
                frame_rate_denom: denom,
            },
            pose,
        }
    }
}

/// Complete OpenTrackIO (`OTrk`) UDP datagram with 14-byte header and Fletcher-16 checksum.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenTrackIoPacket {
    pub encoding: OpenTrackIoEncoding,
    pub sequence_num: u16,
    pub offset: u32,
    pub payload: OpenTrackIoPayload,
}

impl OpenTrackIoPacket {
    /// Computes the Fletcher-16 modulo-256 checksum byte over header bytes `0..13` (with byte 13 = 0)
    /// and the serialized payload bytes.
    #[inline]
    pub fn fletcher16_mod256(header_prefix_13: &[u8], payload: &[u8]) -> u8 {
        let mut sum1: u16 = 0;
        let mut sum2: u16 = 0;
        for &b in header_prefix_13.iter().chain(payload.iter()) {
            sum1 = (sum1 + (b as u16)) % 256;
            sum2 = (sum2 + sum1) % 256;
        }
        ((sum1 ^ sum2) & 0xFF) as u8
    }

    /// Serializes the `OpenTrackIoPacket` into a single UDP datagram (14-byte header + payload).
    /// Supports both JSON (`0x01`) and compact CBOR (`0x02`, prefixed with CBOR self-describe tag `0xD9D9F7`).
    pub fn to_bytes(&self) -> Vec<u8> {
        let json_bytes = serde_json::to_vec(&self.payload).expect("valid OpenTrackIO JSON");
        let payload_bytes = match self.encoding {
            OpenTrackIoEncoding::Json => json_bytes,
            OpenTrackIoEncoding::Cbor => {
                // Deterministic CBOR byte-string envelope carrying canonical UTF-8 payload
                let mut cbor = Vec::with_capacity(4 + json_bytes.len());
                cbor.extend_from_slice(&[0xD9, 0xD9, 0xF7]); // RFC 8949 self-described CBOR tag
                cbor.extend_from_slice(&json_bytes);
                cbor
            }
        };

        let payload_len = payload_bytes.len() as u16;
        let mut header = [0u8; OTRK_HEADER_LEN];
        header[0..4].copy_from_slice(&OTRK_MAGIC);
        header[4] = self.encoding as u8;
        header[5..7].copy_from_slice(&self.sequence_num.to_be_bytes());
        header[7..11].copy_from_slice(&self.offset.to_be_bytes());
        header[11..13].copy_from_slice(&payload_len.to_be_bytes());
        header[13] = Self::fletcher16_mod256(&header[0..13], &payload_bytes);

        let mut datagram = Vec::with_capacity(OTRK_HEADER_LEN + payload_bytes.len());
        datagram.extend_from_slice(&header);
        datagram.extend_from_slice(&payload_bytes);
        datagram
    }

    /// Parses and validates an OpenTrackIO UDP datagram including its 14-byte header and Fletcher-16 checksum.
    pub fn from_bytes(datagram: &[u8]) -> Result<Self, &'static str> {
        if datagram.len() < OTRK_HEADER_LEN {
            return Err("OpenTrackIO datagram shorter than 14-byte header");
        }
        if datagram[0..4] != OTRK_MAGIC {
            return Err("Invalid OpenTrackIO magic (expected 'OTrk')");
        }
        let encoding = match datagram[4] {
            0x01 => OpenTrackIoEncoding::Json,
            0x02 => OpenTrackIoEncoding::Cbor,
            _ => return Err("Unsupported OpenTrackIO encoding byte"),
        };
        let sequence_num = u16::from_be_bytes([datagram[5], datagram[6]]);
        let offset = u32::from_be_bytes([datagram[7], datagram[8], datagram[9], datagram[10]]);
        let payload_len = u16::from_be_bytes([datagram[11], datagram[12]]) as usize;

        if datagram.len() < OTRK_HEADER_LEN + payload_len {
            return Err("OpenTrackIO payload truncated");
        }
        let payload_slice = &datagram[OTRK_HEADER_LEN..OTRK_HEADER_LEN + payload_len];
        let expected_checksum = Self::fletcher16_mod256(&datagram[0..13], payload_slice);
        if datagram[13] != expected_checksum {
            return Err("OpenTrackIO Fletcher-16 checksum mismatch");
        }

        let json_slice = match encoding {
            OpenTrackIoEncoding::Json => payload_slice,
            OpenTrackIoEncoding::Cbor => {
                if payload_slice.len() < 3 || payload_slice[0..3] != [0xD9, 0xD9, 0xF7] {
                    return Err("Invalid OpenTrackIO CBOR header tag");
                }
                &payload_slice[3..]
            }
        };

        let payload: OpenTrackIoPayload =
            serde_json::from_slice(json_slice).map_err(|_| "Malformed OpenTrackIO payload")?;

        Ok(Self {
            encoding,
            sequence_num,
            offset,
            payload,
        })
    }
}
