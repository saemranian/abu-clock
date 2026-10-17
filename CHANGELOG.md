# Abu Clock (`abu-clock`) Changelog

All notable version transitions (`vA.B.C.D`, incremented by `+1` per completed atomic task) are logged below.

| Version | Date (UTC) | Agent | Task ID | Summary |
| :--- | :--- | :--- | :--- | :--- |
| `v0.0.0.1` | `2026-10-17T14:38:00Z` | `antigravity` | `P0-T1` | Monorepo scaffold, Cargo workspace (`abu-clock-core`, `abu-network`, `abu-daemon`, `abu-ffi`), `.agent-lock.json`, `INTERFACES.md`, and `scripts/bump_version.py`. |
| `v0.0.0.2` | `2026-10-17T14:45:05Z` | `antigravity` | `P1-T1` | Cross-platform monotonic timers and statime::OverlayClock virtual PLL |
| `v0.0.0.3` | `2026-10-17T14:47:58Z` | `antigravity` | `P1-T2` | SMPTE ST 2059-1 / ST 12-1 timecode generator and spinlock timestep (Hz) ticker |
| `v0.0.0.4` | `2026-10-17T14:51:25Z` | `antigravity` | `P1-T3` | Tauri v2 GUI for PC, Mac, Linux, iOS, and Android with mobile wake-lock plugins |
| `v0.0.0.5` | `2026-10-17T14:54:42Z` | `antigravity` | `P1-T4` | Dual-mode PTPv2.1 (statime), gPTP (802.1AS), and unprivileged AbuSync CSPTP (ntpd-rs) |
| `v0.0.0.6` | `2026-10-17T14:57:33Z` | `antigravity` | `P2-T1` | Scheduled future-time sensor triggering, OpenTrackIO (OTrk), and FreeD (0xD1/0xD2) packets |
