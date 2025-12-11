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

/// Mixer configuration (matches Pulse NodeMixerConfig)
struct NodeMixerConfigDesc {
    std::optional<float> gain;
    std::optional<float> pan;
    std::optional<bool> muted;
    std::optional<bool> soloed;
};

/// Node descriptor (matches Pulse NodeDesc)
struct NodeDesc {
    NodeId nodeId;
    std::optional<std::string> trackId;
    std::optional<std::string> laneId;
    NodeKind kind;
    std::optional<PluginFormat> pluginFormat;
    std::optional<std::string> pluginId;
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
    /// Optional mixer configuration for channel-related nodes (e.g. Fader)
    std::optional<NodeMixerConfigDesc> mixer;
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

    /// Parse GraphSnapshot from JSON payload
    /// @param j JSON payload from Pulse
    /// @return Parsed GraphSnapshot or nullopt on error
    static std::optional<GraphSnapshot> fromJson(const nlohmann::json& j);
};
