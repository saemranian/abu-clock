# Abu Clock — Running & Testing Guide (`v0.0.0.12`)

This guide explains how to launch the unified **Abu Clock** PC application and run/test every component of the workspace on your Windows PC.

---

## 1. Launch the Unified `Abu Clock` PC Application (Recommended)

You can monitor, configure, and interactively test **all Phase 1, Phase 2, and Phase 3 features** in one unified desktop window using the **`abu-clock`** binary:

### Option A: One-Click Windows Launcher
Double-click **[`Start-AbuClock.bat`](../Start-AbuClock.bat)** in the repository root folder (`c:\Users\USer\Documents\antigravity\abu-clock\Start-AbuClock.bat`).

### Option B: Launch from PowerShell
```powershell
cd c:\Users\USer\Documents\antigravity\abu-clock
cargo run --bin abu-clock
```

### What opens:
The live Rust engine starts on `http://127.0.0.1:7878` and automatically launches the native **Abu Clock** desktop window containing **5 interactive studio tabs**:
1. **Tab 1 — Clock, Settings & Kalman PLL**: Configure Role (`Auto BMCA`, `Master`, `Follower`), Profile (`AbuSync CSPTP`, `SMPTE ST 2059-2`, `gPTP`, `AES67`), Frame Rate (`23.976` .. `120 fps`, `DF/NDF`), Domain, Priority 1/2, and Phase Trim. Includes a live 40-sample SVG Phase Oscilloscope and fault/jitter injection buttons.
2. **Tab 2 — PTP Engine & ST 2059-2 TLV**: Run interactive IEEE 1588 BMCA Grandmaster elections and inspect the live 20-byte big-endian `SmpteSyncMetadataTlv` wire hex dump.
3. **Tab 3 — Barrier Trigger, Telemetry & Manifests**: Fire `SCHEDULE_CAPTURE_AT` ($T_{\text{now}} + 50\text{ ms}$) multi-sensor triggers, inspect per-rig nanosecond jitter receipts, synthesize 6-DoF **OpenTrackIO (`OTrk`)** and **FreeD (`0xD1`/`0xD2`)** packets, and export **`.abuscan.json`** & **Colmap `images.txt`** manifests.
4. **Tab 4 — Grandmaster Hardware Appliance**: Monitor the Phase 3 Raspberry Pi CM4 + Maxim DS3231 TCXO + `/dev/ptp0` & `/dev/ptp1` PHC appliance and test `Class 6` vs `Class 7` Holdover transitions.
5. **Tab 5 — Help, Fundamentals & Graphical Guide**: Built-in engineering manual with interactive SVG architecture diagrams and a numbered visual walkthrough.

For the full standalone manual and fundamentals reference, see **[`docs/ABU_CLOCK_USER_GUIDE.md`](ABU_CLOCK_USER_GUIDE.md)**.

---

## 2. Run All Automated Workspace Unit & Integration Tests

Runs all automated tests across the 5 Rust workspace crates (`abu-clock-core`, `abu-network`, `abu-ffi`, `abu-clock-gui-shell`, and `abu-daemon`):

```powershell
cargo test --workspace
```

---

## 3. Run the Headless CLI Grandmaster Daemon (`abu-daemon`)

```powershell
# Standard headless node startup check
cargo run -p abu-daemon

# Grandmaster Appliance Mode (prints 1 Hz JSON status with GNSS/TCXO + PTP + SMPTE lock for 5 ticks)
cargo run -p abu-daemon -- --appliance-mode --ticks 5
```

---

## 4. Build & Test the Native Shared Library (`abu_ffi.dll`) with Python & Omniverse

### Step 4.1: Build `abu_ffi.dll`
```powershell
cargo build --release -p abu-ffi
```
This produces `target\release\abu_ffi.dll`.

### Step 4.2: Run the NVIDIA Omniverse Extension (`omni.abu.clock`) Test Suite
```powershell
python integrations/omniverse-ext/omni.abu.clock/tests/test_extension.py
```

---

## 5. Inspect Engine & Mobile Integrations

- **C Header (`extern "C"` ABI)**: [`crates/abu-ffi/include/abu_clock.h`](../crates/abu-ffi/include/abu_clock.h)
- **Windows C++17 Native Wrapper (`Abu-Track-Scan`)**: [`integrations/abu-track-scan/windows/AbuClockWinClient.hpp`](../integrations/abu-track-scan/windows/AbuClockWinClient.hpp)
- **iOS Swift Package (`AbuClockKit`)**: [`integrations/abu-track-scan/ios/AbuClockKit/Package.swift`](../integrations/abu-track-scan/ios/AbuClockKit/Package.swift)
- **Unreal Engine 5 Plugin (`AbuClockUE`)**: [`integrations/unreal-plugin/AbuClockUE`](../integrations/unreal-plugin/AbuClockUE)
- **NVIDIA Omniverse Kit Extension (`omni.abu.clock`)**: [`integrations/omniverse-ext/omni.abu.clock/`](../integrations/omniverse-ext/omni.abu.clock/)
