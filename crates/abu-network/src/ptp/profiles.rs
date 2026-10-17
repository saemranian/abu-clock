//! Multi-Profile Network Synchronization Definitions (`SMPTE ST 2059-2`, `IEEE 802.1AS gPTP`,
//! `AES67 Harmonized`, and Unprivileged `AbuSync CSPTP`).

use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

/// Privileged IEEE 1588-2019 Event UDP port (`Sync`, `Delay_Req`, `Pdelay_Req`, `Pdelay_Resp`).
pub const PTP_EVENT_PORT_PRIVILEGED: u16 = 319;
/// Privileged IEEE 1588-2019 General UDP port (`Announce`, `Follow_Up`, `Delay_Resp`, `Management`).
pub const PTP_GENERAL_PORT_PRIVILEGED: u16 = 320;

/// Unprivileged AbuSync Event UDP port (`CSPTP` Request/Response & high-rate `Sync`).
pub const ABUSYNC_EVENT_PORT_UNPRIVILEGED: u16 = 50319;
/// Unprivileged AbuSync General UDP port (`Announce`, `Follow_Up`, SM TLV).
pub const ABUSYNC_GENERAL_PORT_UNPRIVILEGED: u16 = 50320;

/// Standard PTPv2 / SMPTE ST 2059-2 primary IPv4 multicast address (`224.0.1.129`).
pub const PTP_PRIMARY_MULTICAST_IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 1, 129);
/// Standard IEEE 802.1AS (gPTP) peer-delay IPv4 multicast address (`224.0.0.107`).
pub const GPTP_PDELAY_MULTICAST_IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 107);

/// mDNS service type advertised by Abu Clock Grandmasters on the local subnet.
pub const ABUCLOCK_MDNS_SERVICE_TYPE: &str = "_abuclock._udp.local.";

/// Supported network synchronization profiles in `abu-network`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimingProfileKind {
    /// Profile 1: SMPTE ST 2059-2 Broadcast Media Profile (`224.0.1.129`, UDP `319`/`320`, Default Domain `1`).
    SmpteSt2059_2,
    /// Profile 2: IEEE 802.1AS (gPTP) LiDAR & Robotics Profile (`224.0.0.107` / L2 P2P, `8 Hz` Sync).
    Ieee802_1AsGptp,
    /// Profile 3: AES67 + SMPTE ST 2059-2 Harmonized Profile (`AES-R16-2016`).
    Aes67Harmonized,
    /// Profile 4: AbuSync Unprivileged High-Port Profile (`UDP 50319/50320` + CSPTP + mDNS).
    AbuSyncCsptp,
}

/// Path delay measurement mechanism (`E2E` vs `P2P` vs `CSPTP`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayMechanism {
    /// End-to-End (`Delay_Req` / `Delay_Resp`) used by SMPTE ST 2059-2 and AES67.
    EndToEnd,
    /// Peer-to-Peer (`Pdelay_Req` / `Pdelay_Resp`) required by IEEE 802.1AS (gPTP) LiDARs.
    PeerToPeer,
    /// Stateless Client-Server PTP (`ntpd-rs` CSPTP unicast/broadcast exchange).
    ClientServerPtp,
}

/// Complete configuration for one of the four Abu Clock timing profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimingProfileConfig {
    pub kind: TimingProfileKind,
    /// PTP Domain number (`0..=127`). Defaults to `1` per Cisco/Arista/Leader field guidance
    /// to avoid Domain `0` congestion and Domain `127` test-equipment reservation.
    pub domain_number: u8,
    /// Log2 Sync interval (`-3` = `8 Hz` / `125 ms`, `-4` = `16 Hz` / `62.5 ms`).
    pub log_sync_interval: i8,
    /// Log2 Announce interval (`0` = `1 Hz`, `-2` = `4 Hz` / `250 ms`).
    pub log_announce_interval: i8,
    /// Announce receipt timeout multiplier (`default = 3`).
    pub announce_receipt_timeout: u8,
    /// BMCA Priority 1 (`0..=255`, lower takes precedence; `default = 128`).
    pub priority1: u8,
    /// BMCA Priority 2 (`0..=255`, lower takes precedence; `default = 128`).
    pub priority2: u8,
    /// Delay measurement mechanism.
    pub delay_mechanism: DelayMechanism,
    /// Multicast IPv4 group address.
    pub multicast_ipv4: Ipv4Addr,
    /// Event UDP port (`319` or `50319`).
    pub event_port: u16,
    /// General UDP port (`320` or `50320`).
    pub general_port: u16,
    /// Whether mandatory SMPTE ST 2059-2 Synchronization Metadata (SM) TLV is appended to `Announce`.
    pub append_smpte_sm_tlv: bool,
    /// Whether mDNS auto-discovery (`_abuclock._udp.local.`) is active.
    pub mdns_discovery_enabled: bool,
}

impl TimingProfileConfig {
    /// Profile 1: SMPTE ST 2059-2 Broadcast Media Profile (Domain `1` default, `8 Hz` Sync, `1 Hz` Announce).
    pub const fn smpte_st2059_2(domain_number: u8) -> Self {
        Self {
            kind: TimingProfileKind::SmpteSt2059_2,
            domain_number: if domain_number > 127 { 1 } else { domain_number },
            log_sync_interval: -3,     // 8 Hz (125 ms)
            log_announce_interval: -1, // 2 Hz (500 ms)
            announce_receipt_timeout: 3,
            priority1: 128,
            priority2: 128,
            delay_mechanism: DelayMechanism::EndToEnd,
            multicast_ipv4: PTP_PRIMARY_MULTICAST_IPV4,
            event_port: PTP_EVENT_PORT_PRIVILEGED,
            general_port: PTP_GENERAL_PORT_PRIVILEGED,
            append_smpte_sm_tlv: true,
            mdns_discovery_enabled: true,
        }
    }

    /// Profile 2: IEEE 802.1AS (gPTP) LiDAR & Robotics Profile for RoboSense E1R, RS-LiDAR-M1, Airy, Ruby Plus.
    pub const fn ieee_802_1as_gptp() -> Self {
        Self {
            kind: TimingProfileKind::Ieee802_1AsGptp,
            domain_number: 0,
            log_sync_interval: -3,     // 8 Hz (125 ms)
            log_announce_interval: 0,  // 1 Hz
            announce_receipt_timeout: 3,
            priority1: 128,
            priority2: 128,
            delay_mechanism: DelayMechanism::PeerToPeer,
            multicast_ipv4: GPTP_PDELAY_MULTICAST_IPV4,
            event_port: PTP_EVENT_PORT_PRIVILEGED,
            general_port: PTP_GENERAL_PORT_PRIVILEGED,
            append_smpte_sm_tlv: false,
            mdns_discovery_enabled: false,
        }
    }

    /// Profile 3: AES67 + SMPTE ST 2059-2 Harmonized Profile (AES-R16-2016).
    pub const fn aes67_harmonized(domain_number: u8) -> Self {
        Self {
            kind: TimingProfileKind::Aes67Harmonized,
            domain_number: if domain_number > 127 { 1 } else { domain_number },
            log_sync_interval: -3,     // 8 Hz (125 ms)
            log_announce_interval: 0,  // 1 Hz
            announce_receipt_timeout: 3,
            priority1: 128,
            priority2: 128,
            delay_mechanism: DelayMechanism::EndToEnd,
            multicast_ipv4: PTP_PRIMARY_MULTICAST_IPV4,
            event_port: PTP_EVENT_PORT_PRIVILEGED,
            general_port: PTP_GENERAL_PORT_PRIVILEGED,
            append_smpte_sm_tlv: true,
            mdns_discovery_enabled: true,
        }
    }

    /// Profile 4: AbuSync Unprivileged High-Port Profile (`UDP 50319/50320`, `16 Hz` Sync, CSPTP + mDNS).
    pub const fn abusync_unprivileged(domain_number: u8) -> Self {
        Self {
            kind: TimingProfileKind::AbuSyncCsptp,
            domain_number: if domain_number > 127 { 1 } else { domain_number },
            log_sync_interval: -4,     // 16 Hz (62.5 ms)
            log_announce_interval: -2, // 4 Hz (250 ms)
            announce_receipt_timeout: 3,
            priority1: 128,
            priority2: 128,
            delay_mechanism: DelayMechanism::ClientServerPtp,
            multicast_ipv4: PTP_PRIMARY_MULTICAST_IPV4,
            event_port: ABUSYNC_EVENT_PORT_UNPRIVILEGED,
            general_port: ABUSYNC_GENERAL_PORT_UNPRIVILEGED,
            append_smpte_sm_tlv: true,
            mdns_discovery_enabled: true,
        }
    }

    /// Returns the Sync message interval duration in nanoseconds ($2^{\text{log\_sync\_interval}}$ seconds).
    pub fn sync_interval_ns(&self) -> u64 {
        if self.log_sync_interval >= 0 {
            1_000_000_000u64 << (self.log_sync_interval as u32)
        } else {
            1_000_000_000u64 >> ((-self.log_sync_interval) as u32)
        }
    }
}

impl Default for TimingProfileConfig {
    fn default() -> Self {
        Self::smpte_st2059_2(1)
    }
}
