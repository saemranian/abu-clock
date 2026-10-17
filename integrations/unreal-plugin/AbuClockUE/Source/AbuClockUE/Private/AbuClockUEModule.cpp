#include "AbuClockTimecodeProvider.h"
#include "AbuClockCustomTimeStep.h"
#include "AbuTrackLiveLinkSource.h"
#include "abu_clock.h"

bool UAbuClockTimecodeProvider::Initialize(UEngine* InEngine)
{
    NativeClockHandle = abu_clock_create(
        1729180800000000000ULL,
        static_cast<uint32_t>(FrameRateNumerator),
        static_cast<uint32_t>(FrameRateDenominator),
        bDropFrame
    );
    SyncState = (NativeClockHandle != nullptr)
        ? ETimecodeProviderSynchronizationState::Synchronized
        : ETimecodeProviderSynchronizationState::Error;
    return NativeClockHandle != nullptr;
}

void UAbuClockTimecodeProvider::Shutdown(UEngine* InEngine)
{
    if (NativeClockHandle != nullptr)
    {
        abu_clock_destroy(NativeClockHandle);
        NativeClockHandle = nullptr;
    }
    SyncState = ETimecodeProviderSynchronizationState::Closed;
}

bool UAbuClockTimecodeProvider::FetchTimecode(FQualifiedFrameTime& OutFrameTime)
{
    if (NativeClockHandle == nullptr)
    {
        return false;
    }
    AbuClockCFrameSnapshot Snap{};
    if (!abu_clock_synthesize_now(NativeClockHandle, &Snap))
    {
        return false;
    }
    const FTimecode Tc(
        static_cast<int32>(Snap.hours),
        static_cast<int32>(Snap.minutes),
        static_cast<int32>(Snap.seconds),
        static_cast<int32>(Snap.frames),
        Snap.drop_frame != 0
    );
    const FFrameRate Rate(FrameRateNumerator, FrameRateDenominator);
    OutFrameTime = FQualifiedFrameTime(Tc, Rate);
    return true;
}

ETimecodeProviderSynchronizationState UAbuClockTimecodeProvider::GetSynchronizationState() const
{
    return SyncState;
}

bool UAbuClockCustomTimeStep::Initialize(UEngine* InEngine)
{
    NativeClockHandle = abu_clock_create(
        1729180800000000000ULL,
        static_cast<uint32_t>(TargetFrameRate.Numerator),
        static_cast<uint32_t>(TargetFrameRate.Denominator),
        false
    );
    State = (NativeClockHandle != nullptr)
        ? ECustomTimeStepSynchronizationState::Synchronized
        : ECustomTimeStepSynchronizationState::Error;
    return NativeClockHandle != nullptr;
}

void UAbuClockCustomTimeStep::Shutdown(UEngine* InEngine)
{
    if (NativeClockHandle != nullptr)
    {
        abu_clock_destroy(NativeClockHandle);
        NativeClockHandle = nullptr;
    }
    State = ECustomTimeStepSynchronizationState::Closed;
}

bool UAbuClockCustomTimeStep::Execute(UEngine* InEngine)
{
    if (NativeClockHandle == nullptr)
    {
        return true;
    }
    UpdateApplicationLastTime();
    AbuClockCFrameSnapshot Snap{};
    // Hybrid OS sleep + spinlock wait until exact phi = 0.0 nanosecond boundary
    abu_clock_wait_next_timestep(NativeClockHandle, &Snap);
    return false;
}

ECustomTimeStepSynchronizationState UAbuClockCustomTimeStep::GetSynchronizationState() const
{
    return State;
}

FAbuTrackLiveLinkSource::FAbuTrackLiveLinkSource(uint16 InUdpPort)
    : UdpPort(InUdpPort)
{
}

FAbuTrackLiveLinkSource::~FAbuTrackLiveLinkSource() = default;
