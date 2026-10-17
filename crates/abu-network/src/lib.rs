//! `abu-network`: Tier 2 Network Synchronization and Transport Bus.
//!
//! Multiplexes SMPTE ST 2059-2 multicast PTP packets, IEEE 802.1AS gPTP frames,
//! unprivileged AbuSync CSPTP/SNTP datagrams (ports 50319/50320), scheduled
//! future-time sensor triggers, OpenTrackIO (`OTrk`), and FreeD (`0xD1`/`0xD2`).

pub mod ptp;

pub use abu_clock_core::ABU_CLOCK_VERSION;
pub use ptp::{
    BmcaDataset, ClockIdentity, DelayMechanism, DualModePtpEngine, PtpPortState, PtpWireMessage,
    SmpteSyncMetadataTlv, TimestampedUdpSocket, TimingProfileConfig, TimingProfileKind,
    ABUCLOCK_MDNS_SERVICE_TYPE, ABUSYNC_EVENT_PORT_UNPRIVILEGED, ABUSYNC_GENERAL_PORT_UNPRIVILEGED,
    GPTP_PDELAY_MULTICAST_IPV4, PTP_EVENT_PORT_PRIVILEGED, PTP_GENERAL_PORT_PRIVILEGED,
    PTP_PRIMARY_MULTICAST_IPV4,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_linkage_works() {
        assert!(!ABU_CLOCK_VERSION.is_empty());
    }
}
