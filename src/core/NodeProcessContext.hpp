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
    // Future: tempo, time signature, transport flags, automation slices, etc.
};

