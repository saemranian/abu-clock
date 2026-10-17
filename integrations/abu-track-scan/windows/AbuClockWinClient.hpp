#pragma once

#include <cstdint>
#include <string>
#include <vector>
#include "../../../crates/abu-ffi/include/abu_clock.h"

namespace abu::trackscan {

/// Synchronized multi-sensor frame entry recorded by `AbuClockWinClient` for
/// RealityCapture, Colmap (`images.txt`), LixelCyberColor, and Jawset 3DGS.
struct SynchronizedCaptureEntry {
    uint32_t image_id;
    std::string sensor_id;
    std::string file_path;
    uint64_t tai_ns;
    std::string smpte_timecode;
    uint32_t bcd_timecode;
    int64_t kalman_error_ns;
    // Quaternion (qw, qx, qy, qz) and translation (tx, ty, tz) in world coordinates
    double qw;
    double qx;
    double qy;
    double qz;
    double tx_m;
    double ty_m;
    double tz_m;
    uint32_t camera_model_id;
};

/// Windows C++ aggregator client for Abu Track & Scan (`CamTrack-Pro-01`).
/// Links against `libabuclock` (`abu_clock.h`) to synchronize multi-camera photogrammetry
/// rigs and RoboSense E1R / XGRIDS L2PRO LiDAR streams.
class AbuClockWinClient {
public:
    AbuClockWinClient(
        const std::string& session_name,
        uint64_t initial_tai_ns,
        uint32_t fps_num = 30000,
        uint32_t fps_den = 1001,
        bool drop_frame = true
    );
    ~AbuClockWinClient();

    AbuClockWinClient(const AbuClockWinClient&) = delete;
    AbuClockWinClient& operator=(const AbuClockWinClient&) = delete;

    /// Queries the current disciplined TAI nanosecond from `libabuclock`.
    uint64_t NowTaiNanoseconds() const;

    /// Translates a Windows `QueryPerformanceCounter` nanosecond timestamp to TAI nanoseconds.
    uint64_t TranslateQpcToTaiNanoseconds(uint64_t qpc_ns) const;

    /// Records a synchronized camera or LiDAR frame into the session manifest buffer.
    void RecordCapture(const SynchronizedCaptureEntry& entry);

    /// Exports the unified `.abuscan.json` session manifest.
    std::string ExportAbuScanJsonManifest() const;

    /// Exports a Colmap / Jawset 3DGS compatible `images.txt` pose table.
    std::string ExportColmapImagesTxt() const;

private:
    std::string session_name_;
    AbuClockOpaqueHandle* handle_;
    std::vector<SynchronizedCaptureEntry> entries_;
};

} // namespace abu::trackscan
