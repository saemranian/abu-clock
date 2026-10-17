//! Dual-Mode PTPv2.1 (`statime`), gPTP (`802.1AS`), and Unprivileged `AbuSync CSPTP` (`ntpd-rs`)
//! Network Synchronization Engine with `timestamped-socket` UDP transport.

use std::io;
use std::net::{SocketAddr, UdpSocket};

use abu_clock_core::{
    ClockConvergenceMetrics, FrameRate, PlatformMonotonicClock, PtpTimestamp80, SharedClock,
};
use serde::{Deserialize, Serialize};

use super::profiles::{TimingProfileConfig, ABUSYNC_EVENT_PORT_UNPRIVILEGED};
use super::smpte_tlv::SmpteSyncMetadataTlv;

/// 8-byte IEEE 1588 Clock Identity (`EUI-64`).
pub type ClockIdentity = [u8; 8];

/// IEEE 1588-2019 Best Master Clock Algorithm (BMCA) dataset used for Grandmaster election.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BmcaDataset {
    pub priority1: u8,
    pub clock_class: u8,
    pub clock_accuracy: u8,
    pub offset_scaled_log_variance: u16,
    pub priority2: u8,
    pub clock_identity: ClockIdentity,
    pub steps_removed: u16,
}

impl BmcaDataset {
    /// Compares `self` against `other` using the IEEE 1588-2019 BMCA comparison order.
    /// Returns `true` if `self` is a better Grandmaster candidate than `other`.
    pub fn is_better_than(&self, other: &Self) -> bool {
        (
            self.priority1,
            self.clock_class,
            self.clock_accuracy,
            self.offset_scaled_log_variance,
            self.priority2,
            self.steps_removed,
            self.clock_identity,
        ) < (
            other.priority1,
            other.clock_class,
            other.clock_accuracy,
            other.offset_scaled_log_variance,
            other.priority2,
            other.steps_removed,
            other.clock_identity,
        )
    }
}

/// Port state machine states per IEEE 1588-2019 (`statime` port state machine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PtpPortState {
    Initializing,
    Listening,
    PreMaster,
    Master,
    Passive,
    Uncalibrated,
    Slave,
}

/// Wire packet formats exchanged across PTPv2.1 and AbuSync CSPTP sockets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PtpWireMessage {
    /// PTP `Announce` message carrying BMCA dataset and optional SMPTE ST 2059-2 SM TLV.
    Announce {
        domain: u8,
        sequence_id: u16,
        dataset: BmcaDataset,
        smpte_tlv: Option<SmpteSyncMetadataTlv>,
    },
    /// Two-step or one-step `Sync` + `Follow_Up` timing event with origin timestamp $t_1$.
    SyncEvent {
        domain: u8,
        sequence_id: u16,
        origin_t1: PtpTimestamp80,
    },
    /// Client-Server PTP (CSPTP / `ntpd-rs`) four-timestamp exchange ($t_1, t_2, t_3, t_4$).
    CsptpExchange {
        domain: u8,
        sequence_id: u16,
        /// Client request transmit timestamp $t_1$ (TAI ns).
        client_tx_t1_ns: u64,
        /// Grandmaster receive timestamp $t_2$ (TAI ns).
        master_rx_t2_ns: u64,
        /// Grandmaster transmit timestamp $t_3$ (TAI ns).
        master_tx_t3_ns: u64,
        /// Active Grandmaster SMPTE ST 2059-2 metadata TLV.
        smpte_tlv: SmpteSyncMetadataTlv,
    },
}

impl PtpWireMessage {
    /// Computes the IEEE 1588 / CSPTP master-to-follower phase offset $\theta$ and one-way
    /// path delay $d$ from the four timestamps $(t_1, t_2, t_3, t_4)$:
    ///
    /// $$\theta = \frac{(t_2 - t_1) + (t_3 - t_4)}{2}, \quad d = \frac{(t_4 - t_1) - (t_3 - t_2)}{2}$$
    #[inline]
    pub fn compute_offset_and_delay_ns(t1: u64, t2: u64, t3: u64, t4: u64) -> (i64, u64) {
        let t2_minus_t1 = (t2 as i128) - (t1 as i128);
        let t3_minus_t4 = (t3 as i128) - (t4 as i128);
        let offset_ns = ((t2_minus_t1 + t3_minus_t4) / 2) as i64;

        let rtt = ((t4 as i128) - (t1 as i128)) - ((t3 as i128) - (t2 as i128));
        let delay_ns = (rtt.max(0) / 2) as u64;
        (offset_ns, delay_ns)
    }
}

/// Timestamped UDP socket wrapper (`timestamped-socket` abstraction) that captures local
/// monotonic and disciplined TAI arrival/departure timestamps right at the socket syscall boundary.
pub struct TimestampedUdpSocket {
    socket: UdpSocket,
    clock: SharedClock,
    pub is_privileged_ptp_port: bool,
}

impl TimestampedUdpSocket {
    /// Attempts to bind to the requested socket address; if binding to privileged port `319`/`320`
    /// fails due to unprivileged user permissions, automatically falls back to unprivileged
    /// AbuSync port `50319` or an ephemeral port.
    pub fn bind_dual_mode(preferred_addr: SocketAddr, clock: SharedClock) -> io::Result<Self> {
        match UdpSocket::bind(preferred_addr) {
            Ok(sock) => {
                let _ = sock.set_nonblocking(true);
                Ok(Self {
                    socket: sock,
                    clock,
                    is_privileged_ptp_port: preferred_addr.port() < 1024,
                })
            }
            Err(_) => {
                let fallback = SocketAddr::new(preferred_addr.ip(), ABUSYNC_EVENT_PORT_UNPRIVILEGED);
                let sock = UdpSocket::bind(fallback)
                    .or_else(|_| UdpSocket::bind(SocketAddr::new(preferred_addr.ip(), 0)))?;
                let _ = sock.set_nonblocking(true);
                Ok(Self {
                    socket: sock,
                    clock,
                    is_privileged_ptp_port: false,
                })
            }
        }
    }

    /// Returns the local bound `SocketAddr`.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Sends a serialized `PtpWireMessage` and returns `(bytes_sent, departure_tai_ns, departure_mono_ns)`.
    pub fn send_timestamped(
        &self,
        msg: &PtpWireMessage,
        target: SocketAddr,
    ) -> io::Result<(usize, u64, u64)> {
        let payload =
            serde_json::to_vec(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mono_tx = PlatformMonotonicClock::now_ns();
        let tai_tx = self.clock.translate_monotonic_to_tai(mono_tx);
        let n = self.socket.send_to(&payload, target)?;
        Ok((n, tai_tx, mono_tx))
    }

    /// Receives a `PtpWireMessage` along with its exact arrival `(arrival_tai_ns, arrival_mono_ns)` timestamp.
    pub fn recv_timestamped(&self) -> io::Result<(PtpWireMessage, SocketAddr, u64, u64)> {
        let mut buf = [0u8; 2048];
        let (len, src) = self.socket.recv_from(&mut buf)?;
        let mono_rx = PlatformMonotonicClock::now_ns();
        let tai_rx = self.clock.translate_monotonic_to_tai(mono_rx);
        let msg: PtpWireMessage = serde_json::from_slice(&buf[..len])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok((msg, src, tai_rx, mono_rx))
    }
}

/// Dual-Mode Network Synchronization Engine coordinating BMCA, Port state machines,
/// SMPTE ST 2059-2 SM TLV propagation, and `OverlayClock` Kalman discipline.
pub struct DualModePtpEngine {
    pub profile: TimingProfileConfig,
    pub port_state: PtpPortState,
    pub local_dataset: BmcaDataset,
    pub grandmaster_dataset: BmcaDataset,
    pub active_frame_rate: FrameRate,
    pub clock: SharedClock,
    sequence_counter: u16,
}

impl DualModePtpEngine {
    /// Creates a new `DualModePtpEngine` with the specified `TimingProfileConfig`, `ClockIdentity`,
    /// and shared `OverlayClock`.
    pub fn new(
        profile: TimingProfileConfig,
        clock_identity: ClockIdentity,
        active_frame_rate: FrameRate,
        clock: SharedClock,
    ) -> Self {
        let ds = BmcaDataset {
            priority1: profile.priority1,
            clock_class: 248,
            clock_accuracy: 0x21, // Accurate to within 100 ns
            offset_scaled_log_variance: 0x436A,
            priority2: profile.priority2,
            clock_identity,
            steps_removed: 0,
        };
        Self {
            profile,
            port_state: PtpPortState::Listening,
            local_dataset: ds,
            grandmaster_dataset: ds,
            active_frame_rate,
            clock,
            sequence_counter: 0,
        }
    }

    /// Runs the Best Master Clock Algorithm (BMCA) upon receiving a remote `Announce` message
    /// and automatically adopts the Grandmaster's SMPTE ST 2059-2 frame rate when elected Follower.
    pub fn process_announce(
        &mut self,
        remote_dataset: BmcaDataset,
        smpte_tlv: Option<SmpteSyncMetadataTlv>,
    ) -> PtpPortState {
        if remote_dataset.is_better_than(&self.local_dataset) {
            self.grandmaster_dataset = remote_dataset;
            self.port_state = PtpPortState::Slave;
            if let Some(tlv) = smpte_tlv {
                self.active_frame_rate = tlv.to_frame_rate();
            }
        } else {
            self.grandmaster_dataset = self.local_dataset;
            self.port_state = PtpPortState::Master;
        }
        self.port_state
    }

    /// Builds an outbound `Announce` message with the mandatory SMPTE ST 2059-2 SM TLV.
    pub fn build_announce(&mut self) -> PtpWireMessage {
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        let tlv = if self.profile.append_smpte_sm_tlv {
            Some(SmpteSyncMetadataTlv::from_frame_rate(self.active_frame_rate))
        } else {
            None
        };
        PtpWireMessage::Announce {
            domain: self.profile.domain_number,
            sequence_id: self.sequence_counter,
            dataset: self.local_dataset,
            smpte_tlv: tlv,
        }
    }

    /// Processes a completed CSPTP / PTP 4-timestamp exchange $(t_1, t_2, t_3, t_4)$ and
    /// disciplines the underlying `OverlayClock`.
    pub fn process_four_timestamp_exchange(
        &mut self,
        t1_ns: u64,
        t2_ns: u64,
        t3_ns: u64,
        t4_ns: u64,
        mono_rx_ns: u64,
        smpte_tlv: Option<SmpteSyncMetadataTlv>,
        is_wireless: bool,
    ) -> ClockConvergenceMetrics {
        if let Some(tlv) = smpte_tlv {
            self.active_frame_rate = tlv.to_frame_rate();
        }
        let (offset_ns, delay_ns) =
            PtpWireMessage::compute_offset_and_delay_ns(t1_ns, t2_ns, t3_ns, t4_ns);
        self.clock
            .discipline_from_network_sample(offset_ns, delay_ns, mono_rx_ns, is_wireless)
    }
}
