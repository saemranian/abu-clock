export type OperatingRole = "Master" | "Follower" | "AutoBmca";

export type GuiTimingProfile =
  | "SmpteSt2059_2"
  | "Ieee802_1AsGptp"
  | "Aes67Harmonized"
  | "AbuSyncCsptp";

export type LockState = "Unlocked" | "Locking" | "Locked";

export interface ClockConvergenceMetrics {
  offset_ns: number;
  drift_ppb: number;
  path_delay_ns: number;
  offset_variance_ns2: number;
  elapsed_lock_ns: number;
  warmup_progress: number;
  is_wireless: boolean;
  lock_state: LockState;
}

export interface GuiSlateSnapshot {
  version: string;
  monotonic_backend: string;
  role: OperatingRole;
  profile: GuiTimingProfile;
  ptp_domain: number;
  priority1: number;
  priority2: number;
  slate_timecode: string;
  bcd_hex: string;
  frame_index: number;
  phase: number;
  flash_marker_active: boolean;
  tai_ns: number;
  frame_rate_hz: number;
  drop_frame: boolean;
  user_offset_ns: number;
  keep_screen_on_active: boolean;
  battery_optimization_unrestricted: boolean;
  convergence: ClockConvergenceMetrics;
}

export interface FrameRatePreset {
  label: string;
  numerator: number;
  denominator: number;
  drop_frame: boolean;
}

export const FRAME_RATE_PRESETS: FrameRatePreset[] = [
  { label: "23.976 fps (Cinema Fractional)", numerator: 24000, denominator: 1001, drop_frame: false },
  { label: "24 fps (True Cinema)", numerator: 24, denominator: 1, drop_frame: false },
  { label: "25 fps (PAL Broadcast)", numerator: 25, denominator: 1, drop_frame: false },
  { label: "29.97 fps DF (NTSC Drop-Frame)", numerator: 30000, denominator: 1001, drop_frame: true },
  { label: "29.97 fps NDF (NTSC Non-Drop)", numerator: 30000, denominator: 1001, drop_frame: false },
  { label: "30 fps (Integer Broadcast)", numerator: 30, denominator: 1, drop_frame: false },
  { label: "48 fps (HFR Cinema)", numerator: 48, denominator: 1, drop_frame: false },
  { label: "50 fps (PAL Progressive)", numerator: 50, denominator: 1, drop_frame: false },
  { label: "59.94 fps DF (Broadcast HFR Drop)", numerator: 60000, denominator: 1001, drop_frame: true },
  { label: "59.94 fps NDF (Broadcast HFR NDF)", numerator: 60000, denominator: 1001, drop_frame: false },
  { label: "60 fps (Virtual Production / UE5)", numerator: 60, denominator: 1, drop_frame: false },
  { label: "120 fps (Ultra-HFR / LED Volume)", numerator: 120, denominator: 1, drop_frame: false },
  { label: "10 Hz (RoboSense E1R LiDAR Sweep)", numerator: 10, denominator: 1, drop_frame: false },
  { label: "15 Hz (LiDAR Mid-Cadence Sweep)", numerator: 15, denominator: 1, drop_frame: false },
  { label: "20 Hz (LiDAR High-Speed Sweep)", numerator: 20, denominator: 1, drop_frame: false },
];
