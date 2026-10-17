"""
NVIDIA Omniverse Kit Extension (`omni.abu.clock`).

Binds `libabuclock` to `omni.timeline` so OpenUSD scene graph evaluations
and Virtual Production AR frustums tick on the exact SMPTE ST 2059-1 / AbuSync
master timestep cadence.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Optional


@dataclass
class TimelineSyncSnapshot:
    tai_ns: int
    frame_index: int
    timecode_str: str
    fps: float
    usd_time_code: float


class AbuClockOmniverseExtension:
    """
    Synchronizes `omni.timeline` timeCodesPerSecond and current USD frame time
    to the disciplined `libabuclock` OverlayClock.
    """

    def __init__(self, fps_num: int = 60, fps_den: int = 1, drop_frame: bool = False) -> None:
        self.fps_num = max(fps_num, 1)
        self.fps_den = max(fps_den, 1)
        self.drop_frame = drop_frame
        self._active = False
        self._epoch_anchor_tai_ns = 1_729_180_800_000_000_000

    def on_startup(self, ext_id: str = "omni.abu.clock") -> None:
        self._active = True

    def on_shutdown(self) -> None:
        self._active = False

    @property
    def fps(self) -> float:
        return float(self.fps_num) / float(self.fps_den)

    def evaluate_usd_timeline_at_tai_ns(self, tai_ns: int) -> TimelineSyncSnapshot:
        frame_duration_ns = (1_000_000_000 * self.fps_den) // self.fps_num
        rel_ns = max(tai_ns - self._epoch_anchor_tai_ns, 0)
        frame_index = rel_ns // max(frame_duration_ns, 1)
        nominal_fps = (self.fps_num + self.fps_den - 1) // self.fps_den

        frames = frame_index % nominal_fps
        total_sec = frame_index // nominal_fps
        seconds = total_sec % 60
        minutes = (total_sec // 60) % 60
        hours = (total_sec // 3600) % 24
        sep = ";" if self.drop_frame else ":"
        tc_str = f"{hours:02d}:{minutes:02d}:{seconds:02d}{sep}{frames:02d}"

        return TimelineSyncSnapshot(
            tai_ns=tai_ns,
            frame_index=frame_index,
            timecode_str=tc_str,
            fps=self.fps,
            usd_time_code=float(frame_index),
        )
