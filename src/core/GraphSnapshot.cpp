#include "core/GraphSnapshot.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <unordered_set>
#include <sstream>

std::optional<GraphSnapshot> GraphSnapshot::fromJson(const nlohmann::json& j) {
    if (!j.is_object()) {
        LOG_ERROR({"GraphSnapshot"}, "JSON payload is not an object");
        return std::nullopt;
    }

    GraphSnapshot snapshot;

    // Parse snapshot ID
    if (j.contains("id") && j["id"].is_string()) {
        snapshot.id = j["id"].get<std::string>();
    } else {
        snapshot.id = "unknown";
        LOG_WARN({"GraphSnapshot"}, "Missing or invalid 'id' field, using 'unknown'");
    }

    // Parse nodes
    if (!j.contains("nodes") || !j["nodes"].is_array()) {
        LOG_ERROR({"GraphSnapshot"}, "Missing or invalid 'nodes' array in graph snapshot");
        return std::nullopt;
    }

    for (const auto& nodeJson : j["nodes"]) {
        if (!nodeJson.is_object()) {
            LOG_WARN({"GraphSnapshot"}, "Skipping invalid node entry (not an object)");
            continue;
        }

        NodeDesc node;

        // Parse nodeId (required)
        if (nodeJson.contains("nodeId") && nodeJson["nodeId"].is_string()) {
            node.nodeId = nodeJson["nodeId"].get<std::string>();
        } else {
            LOG_WARN({"GraphSnapshot"}, "Skipping node with missing or invalid nodeId");
            continue;
        }

        // Parse optional trackId and laneId
        if (nodeJson.contains("trackId") && nodeJson["trackId"].is_string()) {
            node.trackId = nodeJson["trackId"].get<std::string>();
        }
        if (nodeJson.contains("laneId") && nodeJson["laneId"].is_string()) {
            node.laneId = nodeJson["laneId"].get<std::string>();
        }

        // Parse optional Channel metadata (Pulse emits `channel: { channelId, ... }`)
        if (nodeJson.contains("channel") && nodeJson["channel"].is_object()) {
            const auto& channelJson = nodeJson["channel"];
            if (channelJson.contains("channelId") && channelJson["channelId"].is_string()) {
                node.channelId = channelJson["channelId"].get<std::string>();
            }
        } else if (nodeJson.contains("channelId") && nodeJson["channelId"].is_string()) {
            // Legacy/alternate shape (kept for compatibility).
            node.channelId = nodeJson["channelId"].get<std::string>();
        }

        // Parse node kind (required)
        std::string kindStr = "";
        if (nodeJson.contains("kind") && nodeJson["kind"].is_string()) {
            kindStr = nodeJson["kind"].get<std::string>();
        } else {
            LOG_WARN({"GraphSnapshot"}, std::string("Node ") + node.nodeId + " missing 'kind' field");
            continue;
        }

        auto kindOpt = nodeKindFromString(kindStr);
        if (!kindOpt.has_value()) {
            LOG_WARN({"GraphSnapshot"}, std::string("Node ") + node.nodeId + " has invalid kind: \"" + kindStr + "\"");
            continue;
        }
        node.kind = kindOpt.value();

        // Parse plugin metadata (optional)
        if (nodeJson.contains("pluginFormat") && nodeJson["pluginFormat"].is_string()) {
            std::string formatStr = nodeJson["pluginFormat"].get<std::string>();
            if (formatStr == "clap") {
                node.pluginFormat = PluginFormat::Clap;
            } else if (formatStr == "vst3") {
                node.pluginFormat = PluginFormat::Vst3;
            } else if (formatStr == "au") {
                node.pluginFormat = PluginFormat::Au;
            } else if (formatStr == "lv2") {
                node.pluginFormat = PluginFormat::Lv2;
            } else if (formatStr == "native") {
                node.pluginFormat = PluginFormat::Native;
            }
        }
        if (nodeJson.contains("pluginId") && nodeJson["pluginId"].is_string()) {
            node.pluginId = nodeJson["pluginId"].get<std::string>();
        }

        // Parse audio channel configuration (preferred, explicit metadata)
        if (nodeJson.contains("audio") && nodeJson["audio"].is_object()) {
            const auto& audioJson = nodeJson["audio"];
            NodeAudioConfigDesc audioConfig;
            // Parse separate input/output channel counts
            if (audioJson.contains("inputs") && audioJson["inputs"].is_number_unsigned()) {
                audioConfig.numInputs = audioJson["inputs"].get<uint16_t>();
            }
            if (audioJson.contains("outputs") && audioJson["outputs"].is_number_unsigned()) {
                audioConfig.numOutputs = audioJson["outputs"].get<uint16_t>();
            }
            // Backwards compatibility: if old "channels" field exists, use it for both input and output
            if (audioJson.contains("channels") && audioJson["channels"].is_number_unsigned()) {
                uint16_t channels = audioJson["channels"].get<uint16_t>();
                if (audioConfig.numInputs == 0) {
                    audioConfig.numInputs = channels;
                }
                if (audioConfig.numOutputs == 0) {
                    audioConfig.numOutputs = channels;
                }
            }
            if (audioJson.contains("layout") && audioJson["layout"].is_string()) {
                audioConfig.layout = audioJson["layout"].get<std::string>();
            }
            // Set audio config if we have at least output channels (input can be 0 for source nodes)
            if (audioConfig.numOutputs > 0 || audioConfig.numInputs > 0) {
                node.audio = audioConfig;
            }
        }

        // Parse legacy audio/MIDI channel counts (optional, for backwards compatibility)
        if (nodeJson.contains("numAudioInputs") && nodeJson["numAudioInputs"].is_number_unsigned()) {
            node.numAudioInputs = nodeJson["numAudioInputs"].get<uint32_t>();
        }
        if (nodeJson.contains("numAudioOutputs") && nodeJson["numAudioOutputs"].is_number_unsigned()) {
            node.numAudioOutputs = nodeJson["numAudioOutputs"].get<uint32_t>();
        }
        if (nodeJson.contains("numMidiInputs") && nodeJson["numMidiInputs"].is_number_unsigned()) {
            node.numMidiInputs = nodeJson["numMidiInputs"].get<uint32_t>();
        }
        if (nodeJson.contains("numMidiOutputs") && nodeJson["numMidiOutputs"].is_number_unsigned()) {
            node.numMidiOutputs = nodeJson["numMidiOutputs"].get<uint32_t>();
        }

        // Parse optional latency/tail hints (for future use)
        if (nodeJson.contains("latencySamples") && nodeJson["latencySamples"].is_number_unsigned()) {
            node.latencySamples = nodeJson["latencySamples"].get<uint32_t>();
        }
        if (nodeJson.contains("tailSamples") && nodeJson["tailSamples"].is_number_unsigned()) {
            node.tailSamples = nodeJson["tailSamples"].get<uint32_t>();
        }

        // Parse hardware I/O fields (Phase 7+, optional).
        // Pulse emits `device: { deviceId, isDefault }` for hardware nodes.
        if (nodeJson.contains("device") && nodeJson["device"].is_object()) {
            const auto& deviceJson = nodeJson["device"];
            if (deviceJson.contains("deviceId") && deviceJson["deviceId"].is_string()) {
                node.deviceId = deviceJson["deviceId"].get<std::string>();
            }
            if (deviceJson.contains("isDefault") && deviceJson["isDefault"].is_boolean()) {
                node.deviceIsDefault = deviceJson["isDefault"].get<bool>();
            }
        }

        // Legacy/alternate device fields (kept for compatibility).
        if (!node.deviceId.has_value() && nodeJson.contains("deviceId") && nodeJson["deviceId"].is_string()) {
            node.deviceId = nodeJson["deviceId"].get<std::string>();
        }
        if (!node.deviceIsDefault.has_value() && nodeJson.contains("deviceIsDefault") && nodeJson["deviceIsDefault"].is_boolean()) {
            node.deviceIsDefault = nodeJson["deviceIsDefault"].get<bool>();
        }

        if (nodeJson.contains("inputChannelIndex") && nodeJson["inputChannelIndex"].is_number_integer()) {
            node.inputChannelIndex = nodeJson["inputChannelIndex"].get<int>();
        }
        if (nodeJson.contains("portId") && nodeJson["portId"].is_string()) {
            node.portId = nodeJson["portId"].get<std::string>();
        }

        // Parse channel mix configuration (optional, for fader nodes)
        if (nodeJson.contains("channelMix") && nodeJson["channelMix"].is_object()) {
            const auto& channelMixJson = nodeJson["channelMix"];
            NodeChannelMixConfigDesc channelMixConfig;
            if (channelMixJson.contains("gain") && channelMixJson["gain"].is_number()) {
                channelMixConfig.gain = channelMixJson["gain"].get<float>();
            }
            if (channelMixJson.contains("pan") && channelMixJson["pan"].is_number()) {
                channelMixConfig.pan = channelMixJson["pan"].get<float>();
            }
            if (channelMixJson.contains("muted") && channelMixJson["muted"].is_boolean()) {
                channelMixConfig.muted = channelMixJson["muted"].get<bool>();
            }
            if (channelMixJson.contains("soloed") && channelMixJson["soloed"].is_boolean()) {
                channelMixConfig.soloed = channelMixJson["soloed"].get<bool>();
            }
            node.channelMix = channelMixConfig;
        }

        snapshot.nodes.push_back(node);
    }

    // Parse connections
    if (!j.contains("connections") || !j["connections"].is_array()) {
        LOG_ERROR({"GraphSnapshot"}, "Missing or invalid 'connections' array in graph snapshot");
        return std::nullopt;
    }

    for (const auto& connJson : j["connections"]) {
        if (!connJson.is_object()) {
            LOG_WARN({"GraphSnapshot"}, "Skipping invalid connection entry (not an object)");
            continue;
        }

        ConnectionDesc conn;

        // Parse source (either fromStreamId or fromNodeId, mutually exclusive)
        if (connJson.contains("fromStreamId") && connJson["fromStreamId"].is_string()) {
            conn.fromStreamId = connJson["fromStreamId"].get<std::string>();
        } else if (connJson.contains("fromNodeId") && connJson["fromNodeId"].is_string()) {
            conn.fromNodeId = connJson["fromNodeId"].get<std::string>();
        }

        // Parse output/input indices (default to 0)
        if (connJson.contains("fromOutputIndex") && connJson["fromOutputIndex"].is_number_unsigned()) {
            conn.fromOutputIndex = connJson["fromOutputIndex"].get<uint32_t>();
        } else {
            conn.fromOutputIndex = 0;
        }
        if (connJson.contains("toInputIndex") && connJson["toInputIndex"].is_number_unsigned()) {
            conn.toInputIndex = connJson["toInputIndex"].get<uint32_t>();
        } else {
            conn.toInputIndex = 0;
        }

        // Parse destination (required)
        if (connJson.contains("toNodeId") && connJson["toNodeId"].is_string()) {
            conn.toNodeId = connJson["toNodeId"].get<std::string>();
        } else {
            LOG_WARN({"GraphSnapshot"}, "Skipping connection with missing or invalid toNodeId");
            continue;
        }

        snapshot.connections.push_back(conn);
    }

    // Validate snapshot
    // Build set of node IDs for validation
    std::unordered_set<std::string> nodeIds;
    for (const auto& node : snapshot.nodes) {
        nodeIds.insert(node.nodeId);
    }

    // Validate: All referenced node IDs in connections must exist
    for (const auto& conn : snapshot.connections) {
        if (conn.fromNodeId.has_value() && nodeIds.find(conn.fromNodeId.value()) == nodeIds.end()) {
            std::ostringstream msg;
            msg << "Connection references non-existent fromNodeId: " << conn.fromNodeId.value();
            LOG_ERROR({"GraphSnapshot"}, msg.str());
            return std::nullopt;
        }
        if (nodeIds.find(conn.toNodeId) == nodeIds.end()) {
            std::ostringstream msg;
            msg << "Connection references non-existent toNodeId: " << conn.toNodeId;
            LOG_ERROR({"GraphSnapshot"}, msg.str());
            return std::nullopt;
        }
    }

    // Validate: At least one hardware audio output node must exist
    bool hasOutputNode = false;
    for (const auto& node : snapshot.nodes) {
        if (node.kind == NodeKind::HardwareAudioOutput) {
            hasOutputNode = true;
            break;
        }
    }

    if (!hasOutputNode) {
        LOG_ERROR({"GraphSnapshot"}, "GraphSnapshot must contain at least one hardware audio output node");
        return std::nullopt;
    }

    // Log summary
    std::ostringstream msg;
    msg << "Parsed graph snapshot: id='" << snapshot.id << "', "
        << snapshot.nodes.size() << " nodes, "
        << snapshot.connections.size() << " connections";
    LOG_INFO({"GraphSnapshot"}, msg.str());

    return snapshot;
}
