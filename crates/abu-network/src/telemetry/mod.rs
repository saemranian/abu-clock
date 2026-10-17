//! Real-Time Virtual Production Camera Tracking Telemetry (`OpenTrackIO` & `FreeD 0xD1/0xD2`).

pub mod freed;
pub mod opentrackio;

pub use freed::{
    freed_checksum, FreeDPacket, FREED_EXTENDED_TC_ID, FREED_EXTENDED_TC_LEN, FREED_STANDARD_ID,
    FREED_STANDARD_LEN,
};
pub use opentrackio::{
    CameraPose6DoF, OpenTrackIoEncoding, OpenTrackIoPacket, OpenTrackIoPayload,
    OpenTrackIoTimecodeField, OpenTrackIoTiming, OTRK_HEADER_LEN, OTRK_MAGIC,
};

#[cfg(test)]
mod tests {
    use super::*;
    use abu_clock_core::{FrameRate, SmpteTimecode};

    #[test]
    fn opentrackio_json_and_cbor_packets_roundtrip_with_fletcher16() {
        let tc = SmpteTimecode {
            hours: 10,
            minutes: 15,
            seconds: 30,
            frames: 12,
            subframe: 0.25,
            drop_frame: true,
        };
        let pose = CameraPose6DoF {
            camera_id: 2,
            pan_deg: 45.5,
            tilt_deg: -12.25,
            roll_deg: 1.5,
            x_mm: 1200.0,
            y_mm: -450.0,
            z_mm: 1650.0,
            zoom_raw: 5000,
            focus_raw: 8200,
        };
        let payload = OpenTrackIoPayload::new(
            42,
            1_729_180_800_500_000_000,
            tc,
            FrameRate::FPS_29_97_DF,
            pose,
        );

        for enc in [OpenTrackIoEncoding::Json, OpenTrackIoEncoding::Cbor] {
            let pkt = OpenTrackIoPacket {
                encoding: enc,
                sequence_num: 42,
                offset: 0,
                payload: payload.clone(),
            };
            let wire = pkt.to_bytes();
            assert_eq!(&wire[0..4], b"OTrk");
            assert_eq!(wire[4], enc as u8);

            let decoded = OpenTrackIoPacket::from_bytes(&wire).expect("valid OTrk packet");
            assert_eq!(decoded, pkt);
            assert_eq!(decoded.payload.timing.frame_rate_denom, 1001);
        }
    }

    #[test]
    fn freed_standard_29byte_and_extended_41byte_roundtrip() {
        let tc = SmpteTimecode {
            hours: 14,
            minutes: 22,
            seconds: 10,
            frames: 8,
            subframe: 0.0,
            drop_frame: true,
        };
        let pose = CameraPose6DoF {
            camera_id: 1,
            pan_deg: 30.0,
            tilt_deg: -15.0,
            roll_deg: 2.0,
            x_mm: 500.0,
            y_mm: -250.0,
            z_mm: 1800.0,
            zoom_raw: 4096,
            focus_raw: 8192,
        };
        let tai_ns = 1_729_180_810_266_933_600u64;
        let freed = FreeDPacket::new(pose, tc, tai_ns);

        // 1. Standard 29-byte FreeD (0xD1)
        let d1 = freed.to_standard_d1_bytes();
        assert_eq!(d1.len(), 29);
        assert_eq!(d1[0], 0xD1);
        let (decoded_pose, rolling) = FreeDPacket::from_standard_d1_bytes(&d1).unwrap();
        assert_eq!(rolling, 10 * 60 + 8);
        assert!((decoded_pose.pan_deg - 30.0).abs() < 0.001);
        assert!((decoded_pose.x_mm - 500.0).abs() < 0.02);

        // 2. Extended 41-byte FreeD-TC (0xD2)
        let d2 = freed.to_extended_d2_bytes();
        assert_eq!(d2.len(), 41);
        assert_eq!(d2[0], 0xD2);
        let decoded_d2 = FreeDPacket::from_extended_d2_bytes(&d2, true).unwrap();
        assert_eq!(decoded_d2.timecode.to_bcd_u32(), 0x1422_1008);
        assert_eq!(decoded_d2.tai_ns, tai_ns);
    }
}
