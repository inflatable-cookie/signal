#pragma once

/// GraphSnapshot - DTO structure matching Pulse's GraphSnapshot
///
/// Thread: Control thread (read/write)
/// Ownership: Temporary - created from IPC, consumed by GraphEngine
///
/// This structure mirrors Pulse's GraphSnapshot model for IPC deserialization.
/// It is a temporary DTO that GraphEngine consumes to build the runtime graph.

#include "core/GraphNode.hpp"
#include <cstdint>
#include <string>
#include <vector>
#include <optional>
#include <nlohmann/json_fwd.hpp>

/// Plugin format (matches Pulse PluginFormat)
enum class PluginFormat {
    Clap,
    Vst3,
    Au,
    Lv2,
    Native
};

/// Audio channel configuration (matches Pulse NodeAudioConfig)
struct NodeAudioConfigDesc {
    uint16_t numInputs = 0;             // 0 = unknown/not specified, 0 = no audio input
    uint16_t numOutputs = 0;            // 0 = unknown/not specified, 0 = no audio output
    std::optional<std::string> layout;  // Optional layout identifier
};

/// Per-node mix configuration (matches Pulse NodeMixConfig)
struct NodeMixConfigDesc {
    std::optional<float> gain;
};

struct NodeSpatialOptionsDesc {
    std::optional<std::string> mixPolicy;
};

struct NodeSpatialConfigDesc {
    std::optional<bool> enabled;
    std::optional<std::string> adapter;
    std::optional<NodeSpatialOptionsDesc> options;
};

/// Node descriptor (matches Pulse NodeDesc)
struct NodeDesc {
    NodeId nodeId;
    std::optional<std::string> trackId;
    std::optional<std::string> laneId;
    // Optional owning Channel identifier from Pulse (Phase 10 output Channel/Fader work).
    // This is used for output mix selection and metering identifiers.
    std::optional<std::string> channelId;
    NodeKind kind;
    std::optional<PluginFormat> pluginFormat;
    std::optional<std::string> pluginId;
    std::optional<std::string> pluginInstanceId;
    std::optional<std::vector<std::uint8_t>> pluginStateChunk;
    /// Audio channel configuration (explicit metadata, preferred over legacy fields)
    std::optional<NodeAudioConfigDesc> audio;
    /// Legacy channel configuration fields (deprecated, use `audio` instead)
    std::optional<uint32_t> numAudioInputs;
    std::optional<uint32_t> numAudioOutputs;
    std::optional<uint32_t> numMidiInputs;
    std::optional<uint32_t> numMidiOutputs;
    /// Optional latency hint in samples (for future latency compensation)
    /// Signal will compute actual latency from plugin/node capabilities, but this can serve as a hint
    std::optional<uint32_t> latencySamples;
    /// Optional tail hint in samples (for future tail-aware transport)
    /// Signal will compute actual tail from plugin/node capabilities, but this can serve as a hint
    std::optional<uint32_t> tailSamples;
    /// Optional per-node mix configuration for channel-related nodes (e.g. Fader)
    std::optional<NodeMixConfigDesc> mix;
    std::optional<NodeSpatialConfigDesc> spatial;
    // Hardware I/O node fields (Phase 7+)
    std::optional<std::string> deviceId;      // For HardwareAudioInputNode / HardwareAudioOutputNode
    std::optional<bool> deviceIsDefault;      // For HardwareAudioOutputNode selection
    std::optional<int> inputChannelIndex;     // For HardwareAudioInputNode
    std::optional<std::string> portId;        // For HardwareMidiInputNode
};

/// Connection descriptor (matches Pulse ConnectionDesc)
struct ConnectionDesc {
    std::optional<StreamId> fromStreamId;
    std::optional<NodeId> fromNodeId;
    uint32_t fromOutputIndex = 0;
    NodeId toNodeId;
    uint32_t toInputIndex = 0;
};

/// Graph snapshot (matches Pulse GraphSnapshot)
struct GraphSnapshot {
    std::string id;
    std::vector<NodeDesc> nodes;
    std::vector<ConnectionDesc> connections;

    /// Parse GraphSnapshot from JSON payload
    /// @param j JSON payload from Pulse
    /// @return Parsed GraphSnapshot or nullopt on error
    static std::optional<GraphSnapshot> fromJson(const nlohmann::json& j);
};
