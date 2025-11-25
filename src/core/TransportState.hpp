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

struct LoopRegionSamples {
    uint64_t startSamples;
    uint64_t endSamples;
};

struct TransportState {
    bool isPlaying;
    double positionSeconds;
    uint64_t positionSamples; // Current playhead in samples
    bool loopEnabled;
    std::optional<LoopRegion> loopRegion;
    std::optional<LoopRegionSamples> loopRegionSamples; // Sample-based loop region
    double tempo; // Tempo in BPM

    TransportState()
        : isPlaying(false)
        , positionSeconds(0.0)
        , positionSamples(0)
        , loopEnabled(false)
        , loopRegion(std::nullopt)
        , loopRegionSamples(std::nullopt)
        , tempo(120.0) // Default 120 BPM
    {
    }
};

