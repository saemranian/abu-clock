#include "AbuClockWinClient.hpp"
#include <sstream>
#include <iomanip>

namespace abu::trackscan {

AbuClockWinClient::AbuClockWinClient(
    const std::string& session_name,
    uint64_t initial_tai_ns,
    uint32_t fps_num,
    uint32_t fps_den,
    bool drop_frame
)
    : session_name_(session_name),
      handle_(abu_clock_create(initial_tai_ns, fps_num, fps_den, drop_frame)) {}

AbuClockWinClient::~AbuClockWinClient() {
    if (handle_ != nullptr) {
        abu_clock_destroy(handle_);
        handle_ = nullptr;
    }
}

uint64_t AbuClockWinClient::NowTaiNanoseconds() const {
    return abu_clock_now_tai_ns(handle_);
}

uint64_t AbuClockWinClient::TranslateQpcToTaiNanoseconds(uint64_t qpc_ns) const {
    return abu_clock_translate_monotonic_to_tai_ns(handle_, qpc_ns);
}

void AbuClockWinClient::RecordCapture(const SynchronizedCaptureEntry& entry) {
    entries_.push_back(entry);
}

std::string AbuClockWinClient::ExportAbuScanJsonManifest() const {
    std::ostringstream out;
    out << "{\n"
        << "  \"schema\": \"abuscan.manifest.v1\",\n"
        << "  \"session_name\": \"" << session_name_ << "\",\n"
        << "  \"captures\": [\n";
    for (size_t i = 0; i < entries_.size(); ++i) {
        const auto& e = entries_[i];
        out << "    {\n"
            << "      \"image_id\": " << e.image_id << ",\n"
            << "      \"sensor_id\": \"" << e.sensor_id << "\",\n"
            << "      \"file_path\": \"" << e.file_path << "\",\n"
            << "      \"tai_ns\": " << e.tai_ns << ",\n"
            << "      \"smpte_timecode\": \"" << e.smpte_timecode << "\",\n"
            << "      \"bcd_timecode\": " << e.bcd_timecode << ",\n"
            << "      \"error_ns\": " << e.kalman_error_ns << "\n"
            << "    }" << (i + 1 < entries_.size() ? "," : "") << "\n";
    }
    out << "  ]\n}\n";
    return out.str();
}

std::string AbuClockWinClient::ExportColmapImagesTxt() const {
    std::ostringstream out;
    out << "# Image list with two lines of data per image:\n"
        << "#   IMAGE_ID, QW, QX, QY, QZ, TX, TY, TZ, CAMERA_ID, NAME\n"
        << "#   POINTS2D[] as (X, Y, POINT3D_ID)\n";
    out << std::fixed << std::setprecision(6);
    for (const auto& e : entries_) {
        out << e.image_id << " "
            << e.qw << " " << e.qx << " " << e.qy << " " << e.qz << " "
            << e.tx_m << " " << e.ty_m << " " << e.tz_m << " "
            << e.camera_model_id << " " << e.file_path << "\n\n";
    }
    return out.str();
}

} // namespace abu::trackscan
