//! `abu-daemon`: Standalone CLI and Hardware/Software Grandmaster & Follower Service.

pub mod appliance;

use std::env;

use abu_clock_core::ABU_CLOCK_VERSION;
use appliance::{GrandmasterApplianceRuntime, HardwareApplianceConfig};

fn main() {
    let args: Vec<String> = env::args().collect();
    let cfg = HardwareApplianceConfig::default();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("abu-daemon {}", ABU_CLOCK_VERSION);
        return;
    }

    let (runtime, seed_report) =
        GrandmasterApplianceRuntime::initialize_from_ds3231_rtc(cfg, 1_729_180_800);
    let snap = runtime
        .timecode_engine
        .synthesize_at_tai_ns(runtime.clock.now_tai_ns());

    println!(
        "Abu Clock Grandmaster Daemon ({}) | PHC={} | RTC={} | TAI={} ns | TC={} | SCHED_FIFO={}",
        ABU_CLOCK_VERSION,
        runtime.config.primary_phc_device,
        runtime.config.rtc_device,
        seed_report.seeded_phc_tai_ns,
        snap.timecode.to_slate_string(),
        seed_report.sched_fifo_priority
    );
}
