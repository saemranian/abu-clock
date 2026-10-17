use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};

use abu_clock_gui_lib::{server::run_http_server, GuiAppController};

fn launch_desktop_window(url: &str) {
    #[cfg(target_os = "windows")]
    {
        // Try launching a native chromeless desktop window via Microsoft Edge App Mode first
        let edge_launched = Command::new("cmd")
            .args([
                "/C",
                "start",
                "",
                "msedge",
                &format!("--app={}", url),
                "--window-size=1480,960",
            ])
            .spawn()
            .is_ok();

        if !edge_launched {
            let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let controller = GuiAppController::new();
    let snap = controller.snapshot();

    println!("==================================================================");
    println!("  ABU CLOCK — Unified PC Master Clock & Synchronization Studio");
    println!(
        "  Version: {} | Backend: {} | Domain: {}",
        snap.version, snap.monotonic_backend, snap.ptp_domain
    );
    println!(
        "  Initial Slate: {} ({}) | LockState: {:?}",
        snap.slate_timecode, snap.bcd_hex, snap.convergence.lock_state
    );
    println!("==================================================================");

    if args.iter().any(|a| a == "--check" || a == "--once") {
        println!("Self-check passed. Exiting (--check).");
        return;
    }

    let addr = "127.0.0.1:7878";
    let url = format!("http://{}", addr);
    println!("  Live Engine & Studio UI listening on: {}", url);
    println!("  Launching Abu Clock desktop window... (Press Ctrl+C to stop)");

    if !args.iter().any(|a| a == "--no-window") {
        launch_desktop_window(&url);
    }

    let shared = Arc::new(Mutex::new(controller));
    if let Err(e) = run_http_server(addr, shared) {
        eprintln!("Abu Clock server notice: {} (Is Abu Clock already running on {}?)", e, url);
    }
}
