//! Standalone Model Context Protocol (MCP) Server binary (`abu-clock-mcp.exe`).
//!
//! Communicates via JSON-RPC 2.0 over `stdin` / `stdout` for Antigravity AI agents,
//! Cursor/Claude MCP clients, and `Abu Track & Scan` automation pipelines.

fn main() {
    let _ = abu_clock_gui_lib::mcp::run_mcp_stdio_server();
}
