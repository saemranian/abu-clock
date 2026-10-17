use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use abu_clock_gui_lib::{server::run_http_server, GuiAppController};

fn shutdown_existing_instance(addr: &str) {
    if let Ok(mut stream) = TcpStream::connect(addr) {
        let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
        let _ = stream.write_all(b"POST /api/shutdown HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n");
        let _ = stream.flush();
        std::thread::sleep(Duration::from_millis(180));
    }
}

fn launch_desktop_window(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let edge_launched = Command::new("cmd")
            .args([
                "/C",
                "start",
                "",
                "msedge",
                &format!("--app={}", url),
                "--window-size=1520,980",
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
    if args.iter().any(|a| a == "--mcp") {
        let _ = abu_clock_gui_lib::mcp::run_mcp_stdio_server();
        return;
    }

    let controller = GuiAppController::new();
    let snap = controller.snapshot();

    println!("==================================================================");
    println!(
        "  ABU CLOCK ({}) — Unified PC Master Clock & Sync Studio",
        snap.version
    );
    println!(
        "  Backend: {} | Domain: {} | UDP Socket: {}",
        snap.monotonic_backend, snap.ptp_domain, snap.udp_socket.bound_addr
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
    shutdown_existing_instance(addr);

    let url = format!("http://{}", addr);
    println!("  Live Engine & Studio UI listening on: {}", url);
    println!("  Launching Abu Clock desktop window... (Press Ctrl+C to stop)");

    if !args.iter().any(|a| a == "--no-window") {
        launch_desktop_window(&url);
    }

    let shared = Arc::new(Mutex::new(controller));
    if let Err(e) = run_http_server(addr, shared) {
        eprintln!(
            "Abu Clock server notice: {} (Could not bind {})",
            e, url
        );
    }
}
