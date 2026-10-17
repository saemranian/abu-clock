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
| `v0.0.0.7` | `2026-10-17T15:00:22Z` | `antigravity` | `P2-T2` | Mozilla UniFFI SDK (libabuclock) generating Swift, C++, C#, and Python bindings |
| `v0.0.0.8` | `2026-10-17T15:03:14Z` | `antigravity` | `P2-T3` | Native integration modules for Abu Track & Scan (CamTrack-Pro-01) iOS and Windows apps |
| `v0.0.0.9` | `2026-10-17T15:05:40Z` | `antigravity` | `P2-T4` | Unreal Engine 5 (TimecodeProvider + CustomTimeStep) and NVIDIA Omniverse (omni.abu.clock) plugins |
| `v0.0.0.10` | `2026-10-17T15:08:11Z` | `antigravity` | `P3-T1` | Dedicated Linux hardware PTP daemon (/dev/ptp0), Maxim DS3231 RTC boot sync, and RoboSense gPTP configs |
| `v0.0.0.11` | `2026-10-17T15:10:22Z` | `antigravity` | `P3-T2` | Open-source release packaging, multi-OS installers, and ahmademranian.dev/products spec docs |
