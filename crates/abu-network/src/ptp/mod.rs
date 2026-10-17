//! PTPv2.1 (`SMPTE ST 2059-2`), IEEE 802.1AS (`gPTP`), AES67, and Unprivileged `AbuSync CSPTP`
//! network synchronization module.

pub mod engine;
pub mod profiles;
pub mod smpte_tlv;

pub use engine::{
    BmcaDataset, ClockIdentity, DualModePtpEngine, PtpPortState, PtpWireMessage,
    TimestampedUdpSocket,
};
pub use profiles::{
    DelayMechanism, TimingProfileConfig, TimingProfileKind, ABUCLOCK_MDNS_SERVICE_TYPE,
    ABUSYNC_EVENT_PORT_UNPRIVILEGED, ABUSYNC_GENERAL_PORT_UNPRIVILEGED, GPTP_PDELAY_MULTICAST_IPV4,
    PTP_EVENT_PORT_PRIVILEGED, PTP_GENERAL_PORT_PRIVILEGED, PTP_PRIMARY_MULTICAST_IPV4,
};
pub use smpte_tlv::{
    SmpteSyncMetadataTlv, SMPTE_ORGANIZATION_ID, SMPTE_ST2059_SUBTYPE,
    TLV_TYPE_ORGANIZATION_EXTENSION,
};

#[cfg(test)]
mod tests {
    use super::*;
    use abu_clock_core::{FrameRate, OverlayClock};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn smpte_st2059_sm_tlv_wire_roundtrip() {
        let tlv = SmpteSyncMetadataTlv::from_frame_rate(FrameRate::FPS_59_94_DF);
        let bytes = tlv.to_bytes();
        assert_eq!(bytes.len(), 20);
        assert_eq!(&bytes[4..7], &SMPTE_ORGANIZATION_ID);

        let decoded = SmpteSyncMetadataTlv::from_bytes(&bytes).expect("valid SM TLV");
        assert_eq!(decoded, tlv);
        assert_eq!(decoded.to_frame_rate(), FrameRate::FPS_59_94_DF);
    }

    #[test]
    fn bmca_elects_lower_priority1_grandmaster_and_adopts_frame_rate() {
        let clock = OverlayClock::new_shared(1_729_180_000_000_000_000);
        let profile = TimingProfileConfig::smpte_st2059_2(1);
        let mut follower = DualModePtpEngine::new(
            profile,
            [0x00, 0x1A, 0x2B, 0xFF, 0xFE, 0x01, 0x02, 0x02],
            FrameRate::FPS_24,
            clock,
        );

        let gm_dataset = BmcaDataset {
            priority1: 64, // Higher priority than default 128
            clock_class: 6,
            clock_accuracy: 0x20,
            offset_scaled_log_variance: 0x4000,
            priority2: 128,
            clock_identity: [0x00, 0x1A, 0x2B, 0xFF, 0xFE, 0x01, 0x01, 0x01],
            steps_removed: 0,
        };
        let tlv = SmpteSyncMetadataTlv::from_frame_rate(FrameRate::FPS_29_97_DF);
        let state = follower.process_announce(gm_dataset, Some(tlv));
        assert_eq!(state, PtpPortState::Slave);
        assert_eq!(follower.active_frame_rate, FrameRate::FPS_29_97_DF);
    }

    #[test]
    fn unprivileged_loopback_socket_exchange_and_offset_math() {
        let gm_clock = OverlayClock::new_shared(1_729_180_000_000_000_000);
        let follower_clock = OverlayClock::new_shared(1_729_180_000_000_000_000);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let gm_sock = TimestampedUdpSocket::bind_dual_mode(addr, gm_clock).unwrap();
        let follower_sock = TimestampedUdpSocket::bind_dual_mode(addr, follower_clock).unwrap();

        let msg = PtpWireMessage::CsptpExchange {
            domain: 1,
            sequence_id: 1,
            client_tx_t1_ns: 1_000_000_000,
            master_rx_t2_ns: 1_000_000_600,
            master_tx_t3_ns: 1_000_000_700,
            smpte_tlv: SmpteSyncMetadataTlv::from_frame_rate(FrameRate::FPS_60),
        };

        let gm_addr = gm_sock.local_addr().unwrap();
        follower_sock.send_timestamped(&msg, gm_addr).unwrap();

        let (rx_msg, _, _, _) = gm_sock.recv_timestamped().unwrap();
        assert_eq!(rx_msg, msg);

        // Verify 4-timestamp offset & delay calculation:
        // t1=1000, t2=1600, t3=1700, t4=1900 -> offset = ((600) + (-200))/2 = +200 ns, delay = (900 - 100)/2 = 400 ns
        let (offset, delay) = PtpWireMessage::compute_offset_and_delay_ns(
            1_000_000_000,
            1_000_000_600,
            1_000_000_700,
            1_000_000_900,
        );
        assert_eq!(offset, 200);
        assert_eq!(delay, 400);
    }
}
