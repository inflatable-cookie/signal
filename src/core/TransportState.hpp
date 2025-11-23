#pragma once

/// TransportState - Transport playback state
///
/// Thread: Main thread (owned by EngineHost or SignalApp)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by TransportDomain handlers (IPC thread)
///   - Read by audio thread via lock-free snapshot or atomic flags
///   - No concurrent mutations

#include <optional>

struct LoopRegion {
    double startSeconds;
    double endSeconds;
};

struct TransportState {
    bool isPlaying;
    double positionSeconds;
    bool loopEnabled;
    std::optional<LoopRegion> loopRegion;

    TransportState()
        : isPlaying(false)
        , positionSeconds(0.0)
        , loopEnabled(false)
        , loopRegion(std::nullopt)
    {
    }
};

