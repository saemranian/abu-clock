#ifndef ABU_CLOCK_H
#define ABU_CLOCK_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AbuClockOpaqueHandle AbuClockOpaqueHandle;

typedef struct AbuClockCFrameSnapshot {
    uint64_t tai_ns;
    uint64_t frame_index;
    double phase;
    uint64_t sleep_until_next_frame_ns;
    uint32_t bcd_timecode;
    uint8_t hours;
    uint8_t minutes;
    uint8_t seconds;
    uint8_t frames;
    uint8_t drop_frame;
} AbuClockCFrameSnapshot;

typedef struct AbuClockCPose6DoF {
    uint8_t camera_id;
    double pan_deg;
    double tilt_deg;
    double roll_deg;
    double x_mm;
    double y_mm;
    double z_mm;
    uint32_t zoom_raw;
    uint32_t focus_raw;
} AbuClockCPose6DoF;

AbuClockOpaqueHandle* abu_clock_create(
    uint64_t initial_tai_ns,
    uint32_t fps_num,
    uint32_t fps_den,
    bool drop_frame
);

void abu_clock_destroy(AbuClockOpaqueHandle* handle);

uint64_t abu_clock_now_tai_ns(const AbuClockOpaqueHandle* handle);

uint64_t abu_clock_translate_monotonic_to_tai_ns(
    const AbuClockOpaqueHandle* handle,
    uint64_t host_mono_ns
);

bool abu_clock_synthesize_now(
    const AbuClockOpaqueHandle* handle,
    AbuClockCFrameSnapshot* out_snapshot
);

bool abu_clock_wait_next_timestep(
    const AbuClockOpaqueHandle* handle,
    AbuClockCFrameSnapshot* out_snapshot
);

size_t abu_clock_pack_freed_d1(
    const AbuClockOpaqueHandle* handle,
    const AbuClockCPose6DoF* pose,
    uint8_t* out_buf_29
);

size_t abu_clock_pack_freed_d2(
    const AbuClockOpaqueHandle* handle,
    const AbuClockCPose6DoF* pose,
    uint8_t* out_buf_41
);

#ifdef __cplusplus
}
#endif

#endif /* ABU_CLOCK_H */
