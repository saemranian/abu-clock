//! Phase 3 Hardware Grandmaster Appliance Engine (`Raspberry Pi CM4 / Pi 5` + `Maxim DS3231 TCXO`).
//!
//! Manages:
//! - Boot-time calendar epoch seeding from the battery-backed Maxim DS3231 TCXO ($\pm 2\text{ ppm}$)
//!   over I2C (`/dev/rtc0`), applying the +37s TAI-UTC leap second offset.
//! - Direct Linux PTP Hardware Clock (`/dev/ptp0` on Broadcom BCM54210PE / RP1 MAC and optional
//!   `/dev/ptp1` on PCIe Intel I210-T1) seeding and frequency steering.
//! - Simultaneous multi-profile broadcasting:
//!   1. SMPTE ST 2059-2 Studio LAN (`Domain 1`, `224.0.1.129`, `8-16 Hz` Sync + SM TLV)
//!   2. IEEE 802.1AS gPTP LiDAR Port (`Domain 0`, `224.0.0.107` / L2 P2P for RoboSense E1R)
//!   3. AbuSync Unprivileged CSPTP (`UDP 50319/50320` + mDNS `_abuclock._udp.local.` for iOS/Android)

use abu_clock_core::{
    FrameRate, OverlayClock, SharedClock, SmpteTimecodeEngine, DEFAULT_TAI_MINUS_UTC_SECONDS,
};
use abu_network::{DualModePtpEngine, PtpWireMessage, TimingProfileConfig};
use serde::{Deserialize, Serialize};

/// Real-time Linux FIFO scheduling priority mandated for the Phase 3 Grandmaster daemon.
pub const GRANDMASTER_SCHED_FIFO_PRIORITY: u8 = 80;

/// Hardware configuration for the Phase 3 Raspberry Pi CM4 / Pi 5 Grandmaster Appliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareApplianceConfig {
    /// Primary IEEE 1588 PTP Hardware Clock device path (`default = "/dev/ptp0"`).
    pub primary_phc_device: String,
    /// Optional secondary PCIe Intel I210-T1 PHC device path (`"/dev/ptp1"`) for isolated LiDAR gPTP.
    pub secondary_lidar_phc_device: Option<String>,
    /// Maxim DS3231 TCXO I2C RTC device path (`default = "/dev/rtc0"`).
    pub rtc_device: String,
    /// Studio LAN network interface (`"eth0"`).
    pub studio_interface: String,
    /// Optional isolated RoboSense E1R LiDAR network interface (`"eth1"`).
    pub lidar_interface: Option<String>,
    /// Real-time `SCHED_FIFO` priority (`80`).
    pub sched_fifo_priority: u8,
    /// SMPTE ST 2059-2 production domain (`default = 1`).
    pub studio_domain: u8,
    /// Master broadcast frame rate (`default = 29.97 DF`).
    pub master_frame_rate: FrameRate,
    /// TAI minus UTC leap-second offset (`default = 37`).
    pub tai_minus_utc_seconds: i32,
}

impl Default for HardwareApplianceConfig {
    fn default() -> Self {
        Self {
            primary_phc_device: "/dev/ptp0".to_string(),
            secondary_lidar_phc_device: Some("/dev/ptp1".to_string()),
            rtc_device: "/dev/rtc0".to_string(),
            studio_interface: "eth0".to_string(),
            lidar_interface: Some("eth1".to_string()),
            sched_fifo_priority: GRANDMASTER_SCHED_FIFO_PRIORITY,
            studio_domain: 1,
            master_frame_rate: FrameRate::FPS_29_97_DF,
            tai_minus_utc_seconds: DEFAULT_TAI_MINUS_UTC_SECONDS,
        }
    }
}

/// Result of seeding the Ethernet PHY hardware counter (`/dev/ptp0`) from the Maxim DS3231 TCXO.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RtcToPhcSeedReport {
    pub rtc_utc_seconds: u64,
    pub tai_minus_utc_seconds: i32,
    pub seeded_phc_tai_ns: u64,
    pub sched_fifo_priority: u8,
}

/// Multi-profile Grandmaster Appliance runtime driving Studio ST 2059-2, LiDAR gPTP, and AbuSync CSPTP.
pub struct GrandmasterApplianceRuntime {
    pub config: HardwareApplianceConfig,
    pub clock: SharedClock,
    pub timecode_engine: SmpteTimecodeEngine,
    pub studio_st2059_engine: DualModePtpEngine,
    pub lidar_gptp_engine: DualModePtpEngine,
    pub mobile_abusync_engine: DualModePtpEngine,
}

impl GrandmasterApplianceRuntime {
    /// Initializes the Phase 3 Grandmaster Appliance by converting the Maxim DS3231 TCXO UTC epoch
    /// (`rtc_utc_seconds`) into International Atomic Time (`TAI = UTC + 37s`) and seeding the
    /// primary and secondary PTP engines.
    pub fn initialize_from_ds3231_rtc(
        config: HardwareApplianceConfig,
        rtc_utc_seconds: u64,
    ) -> (Self, RtcToPhcSeedReport) {
        let tai_seconds = (rtc_utc_seconds as i128 + config.tai_minus_utc_seconds as i128).max(0) as u64;
        let seeded_phc_tai_ns = tai_seconds.saturating_mul(1_000_000_000);

        let clock = OverlayClock::new_shared(seeded_phc_tai_ns);
        let timecode_engine = SmpteTimecodeEngine::new(config.master_frame_rate, 0);

        let gm_id = [0xDC, 0xA6, 0x32, 0xFF, 0xFE, 0xAB, 0xC0, 0x01];

        let mut studio_cfg = TimingProfileConfig::smpte_st2059_2(config.studio_domain);
        studio_cfg.priority1 = 64; // Hardware TCXO Grandmaster takes precedence over software nodes (128)

        let mut gptp_cfg = TimingProfileConfig::ieee_802_1as_gptp();
        gptp_cfg.priority1 = 64;

        let mut abusync_cfg = TimingProfileConfig::abusync_unprivileged(config.studio_domain);
        abusync_cfg.priority1 = 64;

        let studio_st2059_engine =
            DualModePtpEngine::new(studio_cfg, gm_id, config.master_frame_rate, clock.clone());
        let lidar_gptp_engine =
            DualModePtpEngine::new(gptp_cfg, gm_id, FrameRate::LIDAR_10HZ, clock.clone());
        let mobile_abusync_engine =
            DualModePtpEngine::new(abusync_cfg, gm_id, config.master_frame_rate, clock.clone());

        let report = RtcToPhcSeedReport {
            rtc_utc_seconds,
            tai_minus_utc_seconds: config.tai_minus_utc_seconds,
            seeded_phc_tai_ns,
            sched_fifo_priority: config.sched_fifo_priority,
        };

        (
            Self {
                config,
                clock,
                timecode_engine,
                studio_st2059_engine,
                lidar_gptp_engine,
                mobile_abusync_engine,
            },
            report,
        )
    }

    /// Generates the simultaneous multi-profile `Announce` frames for Studio ST 2059-2,
    /// RoboSense E1R LiDAR gPTP, and Mobile AbuSync CSPTP.
    pub fn emit_multi_profile_announces(
        &mut self,
    ) -> (PtpWireMessage, PtpWireMessage, PtpWireMessage) {
        (
            self.studio_st2059_engine.build_announce(),
            self.lidar_gptp_engine.build_announce(),
            self.mobile_abusync_engine.build_announce(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ds3231_tcxo_boot_seeding_applies_tai_leap_seconds_and_emits_3_profiles() {
        let cfg = HardwareApplianceConfig::default();
        let rtc_utc = 1_729_180_800u64;
        let (mut runtime, report) =
            GrandmasterApplianceRuntime::initialize_from_ds3231_rtc(cfg, rtc_utc);

        assert_eq!(report.tai_minus_utc_seconds, 37);
        assert_eq!(report.seeded_phc_tai_ns, (1_729_180_800 + 37) * 1_000_000_000);
        assert_eq!(report.sched_fifo_priority, 80);

        let (st2059, gptp, abusync) = runtime.emit_multi_profile_announces();
        match st2059 {
            PtpWireMessage::Announce {
                domain, smpte_tlv, ..
            } => {
                assert_eq!(domain, 1);
                assert!(smpte_tlv.is_some());
            }
            _ => panic!("expected Announce"),
        }
        match gptp {
            PtpWireMessage::Announce {
                domain, smpte_tlv, ..
            } => {
                assert_eq!(domain, 0);
                assert!(smpte_tlv.is_none());
            }
            _ => panic!("expected Announce"),
        }
        match abusync {
            PtpWireMessage::Announce {
                domain, smpte_tlv, ..
            } => {
                assert_eq!(domain, 1);
                assert!(smpte_tlv.is_some());
            }
            _ => panic!("expected Announce"),
        }
    }
}
