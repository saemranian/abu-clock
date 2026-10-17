@echo off
title Abu Clock v0.0.0.13 — Master Clock, Timecode ^& Synchronization Studio
cd /d "%~dp0"
echo ==================================================================
echo   Starting ABU CLOCK (v0.0.0.13) Unified PC Application...
echo ==================================================================
taskkill /F /IM abu-clock.exe >nul 2>&1
if exist "target\release\abu-clock.exe" (
    "target\release\abu-clock.exe"
) else (
    cargo run --release --bin abu-clock
)
pause
