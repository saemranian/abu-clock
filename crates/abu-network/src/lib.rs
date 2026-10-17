//! `abu-network`: Tier 2 Network Synchronization and Transport Bus.
//!
//! Multiplexes SMPTE ST 2059-2 multicast PTP packets, IEEE 802.1AS gPTP frames,
//! unprivileged AbuSync CSPTP/SNTP datagrams (ports 50319/50320), scheduled
//! future-time sensor triggers, OpenTrackIO (`OTrk`), and FreeD (`0xD1`/`0xD2`).

pub mod ptp;
pub mod telemetry;
pub mod trigger;

pub use abu_clock_core::ABU_CLOCK_VERSION;
pub use ptp::{
    BmcaDataset, ClockIdentity, DelayMechanism, DualModePtpEngine, PtpPortState, PtpWireMessage,
    SmpteSyncMetadataTlv, TimestampedUdpSocket, TimingProfileConfig, TimingProfileKind,
    ABUCLOCK_MDNS_SERVICE_TYPE, ABUSYNC_EVENT_PORT_UNPRIVILEGED, ABUSYNC_GENERAL_PORT_UNPRIVILEGED,
    GPTP_PDELAY_MULTICAST_IPV4, PTP_EVENT_PORT_PRIVILEGED, PTP_GENERAL_PORT_PRIVILEGED,
    PTP_PRIMARY_MULTICAST_IPV4,
};
pub use telemetry::{
    freed_checksum, CameraPose6DoF, FreeDPacket, OpenTrackIoEncoding, OpenTrackIoPacket,
    OpenTrackIoPayload, OpenTrackIoTimecodeField, OpenTrackIoTiming, FREED_EXTENDED_TC_ID,
    FREED_EXTENDED_TC_LEN, FREED_STANDARD_ID, FREED_STANDARD_LEN, OTRK_HEADER_LEN, OTRK_MAGIC,
};
pub use trigger::{
    AbuScanCoordinator, CaptureTriggerMode, ScheduleCaptureAt, DEFAULT_TRIGGER_LEAD_TIME_NS,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_linkage_works() {
        assert!(!ABU_CLOCK_VERSION.is_empty());
    }
}
