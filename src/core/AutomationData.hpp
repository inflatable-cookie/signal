#pragma once

/// AutomationData - Compiled automation state for audio thread
///
/// Thread: Control thread (creation, updates), Audio thread (read-only)
/// Ownership: Owned by EngineHost, accessed via atomic pointer swap
///
/// This structure contains automation events compiled to sample-based timing.
/// It is immutable once created and swapped into the audio thread.

#include "core/GraphNode.hpp" // For NodeId
#include "core/ScheduleData.hpp" // For TempoMap
#include <string>
#include <vector>
#include <cstdint>

/// Automation curve type
enum class AutomationCurveType {
    Step,    // Step (no interpolation, hold value until next point)
    Linear,  // Linear interpolation
};

/// Compiled automation event (sample-based)
struct AutomationEventCompiled {
    NodeId nodeId;              // Target node ID
    std::string paramId;        // Parameter ID within the target node
    uint64_t timeSamples;       // Absolute time in samples
    float valueNorm;            // Normalised value (0.0..1.0)
    AutomationCurveType curve;  // Interpolation curve type
};

/// Automation data snapshot
///
/// This structure is immutable and thread-safe for read-only access from the audio thread.
/// Updates are done by creating a new instance and swapping the atomic pointer.
struct AutomationData {
    TempoMap tempoMap;                              // Tempo map for timebase conversion
    std::vector<AutomationEventCompiled> events;    // Sorted by timeSamples

    /// Create empty automation data
    static AutomationData empty() {
        AutomationData data;
        // Initialize with default tempo map
        data.tempoMap.defaultTempo = 120.0;
        data.tempoMap.entries.clear();
        return data;
    }
};

