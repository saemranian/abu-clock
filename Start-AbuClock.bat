@echo off
title Abu Clock — Master Clock, Timecode ^& Synchronization Studio
cd /d "%~dp0"
echo ==================================================================
echo   Starting ABU CLOCK Unified PC Application...
echo ==================================================================
if exist "target\release\abu-clock.exe" (
    "target\release\abu-clock.exe"
) else (
    cargo run --release --bin abu-clock
)
pause
