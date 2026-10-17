#!/usr/bin/env bash
# =============================================================================
# Abu Clock Phase 3 Hardware Grandmaster Boot Provisioning Script
# Target: Raspberry Pi Compute Module 4 (CM4) / Raspberry Pi 5 + Maxim DS3231 TCXO
# =============================================================================
set -euo pipefail

RTC_DEV="${RTC_DEV:-/dev/rtc0}"
PRIMARY_PHC="${PRIMARY_PHC:-/dev/ptp0}"
SECONDARY_PHC="${SECONDARY_PHC:-/dev/ptp1}"
STUDIO_IFACE="${STUDIO_IFACE:-eth0}"
LIDAR_IFACE="${LIDAR_IFACE:-eth1}"
TAI_LEAP_SECONDS="${TAI_LEAP_SECONDS:-37}"

echo "[abu-clock] Step 1: Reading battery-backed Maxim DS3231 TCXO (${RTC_DEV})..."
if command -v hwclock >/dev/null 2>&1 && [ -e "${RTC_DEV}" ]; then
    hwclock --hctosys --rtc="${RTC_DEV}" --utc
fi

UTC_EPOCH_SEC="$(date -u +%s)"
TAI_EPOCH_SEC="$((UTC_EPOCH_SEC + TAI_LEAP_SECONDS))"
echo "[abu-clock] Step 2: Seeding IEEE 1588 PHC (${PRIMARY_PHC}) with TAI epoch ${TAI_EPOCH_SEC}s (+${TAI_LEAP_SECONDS}s TAI-UTC)..."

if command -v phc_ctl >/dev/null 2>&1 && [ -e "${PRIMARY_PHC}" ]; then
    phc_ctl "${PRIMARY_PHC}" set "${TAI_EPOCH_SEC}.000000000"
fi

if command -v phc_ctl >/dev/null 2>&1 && [ -e "${SECONDARY_PHC}" ]; then
    echo "[abu-clock] Step 3: Seeding secondary Intel I210-T1 LiDAR PHC (${SECONDARY_PHC})..."
    phc_ctl "${SECONDARY_PHC}" set "${TAI_EPOCH_SEC}.000000000"
fi

if command -v ethtool >/dev/null 2>&1; then
    ethtool -T "${STUDIO_IFACE}" || true
fi

echo "[abu-clock] Step 4: Launching abu-daemon with SCHED_FIFO priority 80..."
exec chrt -f 80 /usr/local/bin/abu-daemon --grandmaster --phc "${PRIMARY_PHC}" --rtc "${RTC_DEV}"
