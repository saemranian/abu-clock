#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use abu_clock_gui_lib::GuiAppController;

fn main() {
    let controller = GuiAppController::new();
    let snap = controller.snapshot();
    println!(
        "Abu Clock GUI Shell initialized: {} | Slate={} | Domain={} | Backend={}",
        snap.version, snap.slate_timecode, snap.ptp_domain, snap.monotonic_backend
    );
}
