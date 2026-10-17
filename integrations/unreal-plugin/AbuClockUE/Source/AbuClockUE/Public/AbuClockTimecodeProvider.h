#pragma once

#include "CoreMinimal.h"
#include "Engine/TimecodeProvider.h"
#include "Misc/QualifiedFrameTime.h"
#include "AbuClockTimecodeProvider.generated.h"

struct AbuClockOpaqueHandle;

/**
 * UAbuClockTimecodeProvider
 *
 * Stamps Unreal Engine 5's global timecode and TimecodeSynchronizer directly from
 * libabuclock's disciplined OverlayClock (SMPTE ST 2059-1 / ST 12-1).
 */
UCLASS(Blueprintable, EditInlineNew, meta = (DisplayName = "Abu Clock SMPTE ST 2059 Timecode Provider"))
class ABUCLOCKUE_API UAbuClockTimecodeProvider : public UTimecodeProvider
{
    GENERATED_BODY()

public:
    UPROPERTY(EditAnywhere, Category = "AbuClock")
    int32 FrameRateNumerator = 30000;

    UPROPERTY(EditAnywhere, Category = "AbuClock")
    int32 FrameRateDenominator = 1001;

    UPROPERTY(EditAnywhere, Category = "AbuClock")
    bool bDropFrame = true;

    UPROPERTY(EditAnywhere, Category = "AbuClock")
    int32 PtpDomain = 1;

    virtual bool FetchTimecode(FQualifiedFrameTime& OutFrameTime) override;
    virtual ETimecodeProviderSynchronizationState GetSynchronizationState() const override;
    virtual bool Initialize(class UEngine* InEngine) override;
    virtual void Shutdown(class UEngine* InEngine) override;

private:
    AbuClockOpaqueHandle* NativeClockHandle = nullptr;
    ETimecodeProviderSynchronizationState SyncState = ETimecodeProviderSynchronizationState::Closed;
};
