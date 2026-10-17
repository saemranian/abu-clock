//! `abu-network`: Tier 2 Network Synchronization and Transport Bus.
//!
//! Multiplexes SMPTE ST 2059-2 multicast PTP packets, IEEE 802.1AS gPTP frames,
//! unprivileged AbuSync CSPTP/SNTP datagrams (ports 50319/50320), scheduled
//! future-time sensor triggers, OpenTrackIO (`OTrk`), and FreeD (`0xD1`/`0xD2`).

pub use abu_clock_core::ABU_CLOCK_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_linkage_works() {
        assert!(!ABU_CLOCK_VERSION.is_empty());
    }
}
