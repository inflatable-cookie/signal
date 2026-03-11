#pragma once

/// ParameterChange - Parameter update from Pulse automation
///
/// Thread: Control thread (creation, application)
/// Ownership: Temporary - passed to EngineHost::applyParameterChanges
///
/// This structure represents a single parameter change that should be applied
/// to a plugin node. Changes are queued and applied at block boundaries to
/// maintain real-time safety.

#include <string>
#include <vector>

/// Parameter change descriptor
struct ParameterChange {
    std::string nodeId;        // Target node ID
    std::string paramId;        // Parameter ID (plugin's parameter identifier)
    float normalisedValue;      // Normalised value (0.0..1.0)
};

