# Phase 3 Dedicated Hardware Master Clock Appliance — Bill of Materials & Setup Guide

## Hardware Bill of Materials (BOM)

| Component | Model & Interface Specification | Architectural Role in the Abu Clock Appliance | Estimated Cost (AED) |
| :--- | :--- | :--- | :--- |
| **Compute & Hardware PTP PHY** | Raspberry Pi Compute Module 4 (CM4) + CM4 IO Board (or Raspberry Pi 5) | Integrates Broadcom `BCM54210PE` (CM4) or `RP1` (Pi 5) Ethernet MAC/PHY with native silicon IEEE 1588 Hardware Timestamping (`/dev/ptp0`). | 200–300 AED |
| **Ground Holdover Oscillator** | Maxim DS3231 TCXO RTC Module (I2C GPIO Header) | Temperature-Compensated Crystal Oscillator ($\pm 2\text{ ppm}$ thermal stability) with battery backup; preserves true TAI/UTC epoch across power cycles without a roof antenna. | 15–25 AED |
| **Optional Dual-Port PTP NIC** | Intel I210-T1 PCIe x1 Gigabit NIC (fits CM4 IO Board PCIe x1 slot) | Adds a second independent hardware-timestamped PTP port (`/dev/ptp1`) to isolate RoboSense E1R gPTP LiDAR traffic from the SMPTE ST 2059-2 Studio LAN. | 70–100 AED |
| **LiDAR Media Converter** | RoboSense E1R Power & Ethernet Interface Box (`1000BASE-T1` to `RJ-45`) | Bridges the E1R's 2-wire automotive Ethernet connector into a standard `RJ-45` Gigabit port for direct connection to the Abu Clock switch. | Included with E1R |

---

## Provisioning on Raspberry Pi CM4 / Pi 5

1. Enable the I2C bus and Maxim DS3231 RTC overlay in `/boot/firmware/config.txt`:
   ```ini
   dtparam=i2c_arm=on
   dtoverlay=i2c-rtc,ds3231
   ```
2. Install `abu-daemon` and copy the provisioning scripts:
   ```bash
   sudo install -m 0755 target/release/abu-daemon /usr/local/bin/abu-daemon
   sudo mkdir -p /opt/abu-clock/hardware/pi-cm4 /opt/abu-clock/hardware/configs
   sudo install -m 0755 hardware/pi-cm4/init-ground-clock.sh /opt/abu-clock/hardware/pi-cm4/
   sudo install -m 0644 hardware/configs/*.cfg /opt/abu-clock/hardware/configs/
   sudo install -m 0644 hardware/pi-cm4/abu-clock-grandmaster.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now abu-clock-grandmaster.service
   ```
