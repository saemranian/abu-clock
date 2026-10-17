"""
Python ctypes / UniFFI binding wrapper for libabuclock (`abu-ffi`).
Used by the NVIDIA Omniverse Kit extension (`omni.abu.clock`) and Python automation scripts.
"""

from __future__ import annotations
import ctypes
from dataclasses import dataclass
from typing import Optional


class AbuClockCFrameSnapshot(ctypes.Structure):
    _fields_ = [
        ("tai_ns", ctypes.c_uint64),
        ("frame_index", ctypes.c_uint64),
        ("phase", ctypes.c_double),
        ("sleep_until_next_frame_ns", ctypes.c_uint64),
        ("bcd_timecode", ctypes.c_uint32),
        ("hours", ctypes.c_uint8),
        ("minutes", ctypes.c_uint8),
        ("seconds", ctypes.c_uint8),
        ("frames", ctypes.c_uint8),
        ("drop_frame", ctypes.c_uint8),
    ]


@dataclass
class PyTimecodeSnapshot:
    tai_ns: int
    frame_index: int
    phase: float
    bcd_timecode: int
    hours: int
    minutes: int
    seconds: int
    frames: int
    drop_frame: bool

    def format_smpte(self) -> str:
        sep = ";" if self.drop_frame else ":"
        return f"{self.hours:02d}:{self.minutes:02d}:{self.seconds:02d}{sep}{self.frames:02d}"
