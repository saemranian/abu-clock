#pragma once

#include "CoreMinimal.h"
#include "ILiveLinkSource.h"

/**
 * FAbuTrackLiveLinkSource
 *
 * Ingests OpenTrackIO ("OTrk" JSON/CBOR) and Extended FreeD-TC (0xD1 / 0xD2)
 * UDP tracking datagrams from Abu Track (iOS / Windows) and routes frame-accurate
 * 6DoF transforms + lens zoom/focus + SMPTE ST 12-1 timecode into UE5's
 * TimecodeSynchronizer.
 */
class ABUCLOCKUE_API FAbuTrackLiveLinkSource
{
public:
    explicit FAbuTrackLiveLinkSource(uint16 InUdpPort = 40000);
    ~FAbuTrackLiveLinkSource();

    uint16 GetUdpPort() const { return UdpPort; }

private:
    uint16 UdpPort;
};
