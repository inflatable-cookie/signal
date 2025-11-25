#pragma once

/// GraphSnapshot - DTO structure matching Pulse's GraphSnapshot
///
/// Thread: Control thread (read/write)
/// Ownership: Temporary - created from IPC, consumed by GraphEngine
///
/// This structure mirrors Pulse's GraphSnapshot model for IPC deserialization.
/// It is a temporary DTO that GraphEngine consumes to build the runtime graph.

#include "core/GraphNode.hpp"
#include <string>
#include <vector>
#include <optional>

/// Plugin format (matches Pulse PluginFormat)
enum class PluginFormat {
    Clap,
    Vst3,
    Au,
    Lv2,
    Native
};

/// Node descriptor (matches Pulse NodeDesc)
struct NodeDesc {
    NodeId nodeId;
    std::optional<std::string> trackId;
    std::optional<std::string> laneId;
    NodeKind kind;
    std::optional<PluginFormat> pluginFormat;
    std::optional<std::string> pluginId;
    std::optional<uint32_t> numAudioInputs;
    std::optional<uint32_t> numAudioOutputs;
    std::optional<uint32_t> numMidiInputs;
    std::optional<uint32_t> numMidiOutputs;
    // Input node fields (Phase 7)
    std::optional<std::string> deviceId;      // For AudioInputNode
    std::optional<int> inputChannelIndex;     // For AudioInputNode
    std::optional<std::string> portId;        // For MidiInputNode
};

/// Connection descriptor (matches Pulse ConnectionDesc)
struct ConnectionDesc {
    std::optional<StreamId> fromStreamId;
    std::optional<NodeId> fromNodeId;
    uint32_t fromOutputIndex;  // Defaults to 0
    NodeId toNodeId;
    uint32_t toInputIndex;     // Defaults to 0
};

/// Graph snapshot (matches Pulse GraphSnapshot)
struct GraphSnapshot {
    std::string id;
    std::vector<NodeDesc> nodes;
    std::vector<ConnectionDesc> connections;
};

