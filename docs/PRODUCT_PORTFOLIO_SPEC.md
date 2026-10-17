# Abu Creative Technology Ecosystem — Product Portfolio Specification (`ahmademranian.dev/products`)

## 1. Abu Clock (`abu-clock`)
- **Repository:** `https://github.com/saemranian/abu-clock`
- **Role:** Cross-Platform Precision SMPTE ST 2059-1/2 Timecode, Deterministic Timestep (`Hz`) Ticker, and Hardware-Ready PTP/gPTP Grandmaster Clock.
- **Highlights:**
  - Zero-hardware software synchronization across Windows, macOS, Linux, iOS, and Android via `OverlayClock` virtual PLL and `AbuSync` CSPTP (`UDP 50319/50320`).
  - Sub-microsecond hardware PHY timestamping (`/dev/ptp0`) on Raspberry Pi CM4 / Pi 5 with Maxim DS3231 TCXO ground holdover.
  - Native Unreal Engine 5 (`AbuClockUE`) and NVIDIA Omniverse (`omni.abu.clock`) plugins.

## 2. Abu Track & Scan (`CamTrack-Pro-01`)
- **Repository:** `https://github.com/saemranian/CamTrack-Pro-01`
- **Role:** 6DoF Virtual Production Camera Tracking & Synchronized Multi-Sensor Spatial Capture (`LiDAR` + Photogrammetry + 3D Gaussian Splatting).
- **Integration with Abu Clock:**
  - `AbuClockKit` (iOS Swift) stamps every `AVFoundation` video frame and `ARKit` 6DoF pose at its exact optical exposure midpoint, eliminating CMOS rolling-shutter spatial slippage.
  - Broadcasts SMPTE RIS **OpenTrackIO (`OTrk`)** and **Extended FreeD-TC (`0xD2`)** UDP packets to Unreal Engine 5 and Omniverse.
  - Executes scheduled future-time captures ($T_{\text{now}} + 50\text{ ms}$) across iPhone arrays and **RoboSense E1R** / **XGRIDS L2PRO** LiDAR sensors, exporting `.abuscan.json` and Colmap `images.txt` manifests for RealityCapture, Colmap, LixelCyberColor, and Jawset Postshot.

## 3. Abu Cam
- **Role:** Synchronized Cinema & Mobile Camera Array Node.
- **Integration with Abu Clock:**
  - Locks shutter exposure phase and SMPTE ST 12-1 timecode metadata (`HH:MM:SS:FF.sf`) to the Abu Clock Grandmaster over wired USB-C Gigabit LAN or Wi-Fi.
