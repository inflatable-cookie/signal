#pragma once

/// NodeProcessContext - Extended context for node processing
///
/// Thread: Audio thread (read-only)
/// Ownership: Created by GraphEngine, passed to nodes
///
/// This structure provides additional context information to nodes during processing,
/// including sample rate, block size, timing, and future automation/tempo data.

#include <cstdint>

struct NodeProcessContext {
    int sampleRate;           // Current sample rate (e.g., 44100)
    int blockSize;             // Number of frames in this block
    uint64_t blockStartSample; // Absolute sample position of block start

    // Transport/tempo information (Phase 8)
    double tempo;              // Current tempo in BPM
    bool isPlaying;            // Is transport playing?
    bool loopEnabled;          // Is loop enabled?
    double loopStartBeats;     // Loop start in beats (if enabled)
    double loopEndBeats;       // Loop end in beats (if enabled)
};

