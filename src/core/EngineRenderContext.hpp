#pragma once

/// EngineRenderContext - Context information for audio rendering
///
/// Thread: Audio thread (read-only)
/// Ownership: Created by AudioBackend, passed to EngineHost::renderBlock

#include <cstdint>

struct EngineRenderContext {
    double hostTimeSeconds;  // Host time in seconds (monotonic, from backend)
    double sampleRate;       // Current sample rate
    int blockSize;           // Number of frames in this block

    // Transport position in samples (if available from engine state)
    // EngineHost will derive this from its internal playhead state
    // This field is informational and may be updated by EngineHost
    uint64_t playheadSamples;
};

