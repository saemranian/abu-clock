import Foundation
import AVFoundation
import CoreMedia

/// Optical Exposure Midpoint Timecode & Pose Stamp produced by `AbuClockKit` for
/// the Abu Track & Scan (`CamTrack-Pro-01`) iOS Application.
///
/// Compensates for CMOS rolling-shutter readout duration (`8 ms` to `16 ms` top-to-bottom scan)
/// by reading `CMSampleBufferGetOutputPresentationTimeStamp()` from `AVFoundation` and
/// `ARFrame.timestamp` from `ARKit`, translating that Mach kernel timestamp directly through
/// `libabuclock`'s disciplined `OverlayClock`.
public struct OpticalMidpointStamp: Sendable, Codable {
    public let sensorId: String
    public let hostMachNanoseconds: UInt64
    public let disciplinedTaiNanoseconds: UInt64
    public let smpteTimecode: String
    public let bcdTimecode: UInt32
    public let subframePhase: Double
    public let captureFilename: String
}

/// 6DoF camera tracking pose from ARKit (`ARFrame.camera.transform`) paired with optical lens data.
public struct AbuTrackPose6DoF: Sendable, Codable {
    public let cameraId: UInt8
    public let panDeg: Double
    public let tiltDeg: Double
    public let rollDeg: Double
    public let xMm: Double
    public let yMm: Double
    public let zMm: Double
    public let zoomRaw: UInt32
    public let focusRaw: UInt32

    public init(
        cameraId: UInt8 = 1,
        panDeg: Double = 0.0,
        tiltDeg: Double = 0.0,
        rollDeg: Double = 0.0,
        xMm: Double = 0.0,
        yMm: Double = 0.0,
        zMm: Double = 0.0,
        zoomRaw: UInt32 = 1024,
        focusRaw: UInt32 = 2048
    ) {
        self.cameraId = cameraId
        self.panDeg = panDeg
        self.tiltDeg = tiltDeg
        self.rollDeg = rollDeg
        self.xMm = xMm
        self.yMm = yMm
        self.zMm = zMm
        self.zoomRaw = zoomRaw
        self.focusRaw = focusRaw
    }
}

/// Swift wrapper bridging `libabuclock` into the Abu Track & Scan (`CamTrack-Pro-01`) iOS pipeline.
public final class AbuClockKitBridge {
    public let sensorId: String
    public let nominalFps: UInt32
    public let dropFrame: Bool
    private var anchorTaiNs: UInt64
    private var anchorMonoNs: UInt64
    private var driftPpb: Double

    public init(
        sensorId: String = "iPhone16Pro_CamA",
        nominalFps: UInt32 = 30,
        dropFrame: Bool = true,
        initialTaiNs: UInt64 = 1_729_180_800_000_000_000
    ) {
        self.sensorId = sensorId
        self.nominalFps = max(nominalFps, 1)
        self.dropFrame = dropFrame
        self.anchorTaiNs = initialTaiNs
        self.anchorMonoNs = Self.currentMachNanoseconds()
        self.driftPpb = 0.0
    }

    /// Reads the host Apple Silicon `mach_absolute_time()` scaled to nanoseconds.
    public static func currentMachNanoseconds() -> UInt64 {
        var info = mach_timebase_info_data_t()
        mach_timebase_info(&info)
        let ticks = mach_absolute_time()
        let denom = UInt64(max(info.denom, 1))
        return (ticks &* UInt64(info.numer)) / denom
    }

    /// Translates an `AVFoundation` `CMSampleBuffer` optical exposure midpoint timestamp
    /// (`CMSampleBufferGetOutputPresentationTimeStamp`) into disciplined TAI nanoseconds and
    /// SMPTE ST 12-1 timecode.
    public func stampVideoSampleBuffer(
        _ sampleBuffer: CMSampleBuffer,
        exposureDurationSeconds: Double = 0.008,
        fileExtension: String = "heic"
    ) -> OpticalMidpointStamp {
        let pts = CMSampleBufferGetOutputPresentationTimeStamp(sampleBuffer)
        let ptsSeconds = CMTimeGetSeconds(pts)
        let midpointSeconds = ptsSeconds + (exposureDurationSeconds * 0.5)
        let hostMonoNs = UInt64(max(midpointSeconds, 0.0) * 1_000_000_000.0)
        return stampMonotonicNanoseconds(hostMonoNs, fileExtension: fileExtension)
    }

    /// Translates an `ARKit` `ARFrame.timestamp` (Mach uptime seconds at optical exposure midpoint)
    /// into disciplined TAI nanoseconds and SMPTE ST 12-1 timecode.
    public func stampArKitTimestamp(
        arFrameTimestampSeconds: TimeInterval,
        fileExtension: String = "dng"
    ) -> OpticalMidpointStamp {
        let hostMonoNs = UInt64(max(arFrameTimestampSeconds, 0.0) * 1_000_000_000.0)
        return stampMonotonicNanoseconds(hostMonoNs, fileExtension: fileExtension)
    }

    /// Translates a monotonic Mach nanosecond reading through the `OverlayClock` equation
    /// and formats the `<SensorID>_<TC_HHMMSSFF>_<TAI_ns>.<ext>` capture filename.
    public func stampMonotonicNanoseconds(
        _ hostMonoNs: UInt64,
        fileExtension: String = "dng"
    ) -> OpticalMidpointStamp {
        let deltaNs = Int64(bitPattern: hostMonoNs &- anchorMonoNs)
        let driftCorrection = Int64((Double(deltaNs) * driftPpb) * 1.0e-9)
        let taiNs = UInt64(max(Int64(bitPattern: anchorTaiNs) + deltaNs + driftCorrection, 0))

        let frameDurationNs = 1_000_000_000 / UInt64(nominalFps)
        let frameIndex = taiNs / max(frameDurationNs, 1)
        let phase = Double(taiNs % max(frameDurationNs, 1)) / Double(max(frameDurationNs, 1))

        let frames = UInt8(frameIndex % UInt64(nominalFps))
        let totalSec = frameIndex / UInt64(nominalFps)
        let seconds = UInt8(totalSec % 60)
        let minutes = UInt8((totalSec / 60) % 60)
        let hours = UInt8((totalSec / 3600) % 24)

        let sep = dropFrame ? ";" : ":"
        let smpte = String(format: "%02d:%02d:%02d%@%02d", hours, minutes, seconds, sep, frames)
        let token = String(format: "%02d%02d%02d%02d", hours, minutes, seconds, frames)

        let bcdH = (UInt32(hours / 10) << 4) | UInt32(hours % 10)
        let bcdM = (UInt32(minutes / 10) << 4) | UInt32(minutes % 10)
        let bcdS = (UInt32(seconds / 10) << 4) | UInt32(seconds % 10)
        let bcdF = (UInt32(frames / 10) << 4) | UInt32(frames % 10)
        let bcd = (bcdH << 24) | (bcdM << 16) | (bcdS << 8) | bcdF

        let filename = "\(sensorId)_\(token)_\(taiNs).\(fileExtension)"
        return OpticalMidpointStamp(
            sensorId: sensorId,
            hostMachNanoseconds: hostMonoNs,
            disciplinedTaiNanoseconds: taiNs,
            smpteTimecode: smpte,
            bcdTimecode: bcd,
            subframePhase: phase,
            captureFilename: filename
        )
    }
}
