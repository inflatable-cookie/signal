#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "logging/Logging.hpp"
#include <sstream>

void GraphEngine::loadGraphSnapshot(const GraphSnapshot& snapshot, PluginHost* pluginHost, EngineHost* engineHost) {
    // Clear existing graph
    clear();
    _unavailablePluginNodes.clear();

    _pluginHost = pluginHost;

    std::ostringstream loadMsg;
    loadMsg << "Loading graph snapshot: " << snapshot.id
        << " (" << snapshot.nodes.size() << " nodes, "
        << snapshot.connections.size() << " connections)";
    LOG_INFO({"GraphEngine", "Graph"}, loadMsg.str());

    // Create nodes and assign channel configurations
    for (const auto& desc : snapshot.nodes) {
        // Validate channel metadata for audio-processing nodes
        bool requiresAudioChannels = (
            desc.kind == NodeKind::AudioLane ||
            desc.kind == NodeKind::Instrument ||
            desc.kind == NodeKind::AudioFx ||
            desc.kind == NodeKind::Fader ||
            desc.kind == NodeKind::Receive ||
            desc.kind == NodeKind::HardwareAudioOutput ||
            desc.kind == NodeKind::HardwareAudioInput
        );

        if (requiresAudioChannels) {
            // Check if audio.numOutputs is missing (prefer explicit metadata)
            // Note: numInputs can be 0 for source nodes (e.g., AudioLane), so we check numOutputs
            if (!desc.audio.has_value() || desc.audio->numOutputs == 0) {
                // Fall back to legacy fields
                bool hasLegacyChannels = desc.numAudioOutputs.has_value() && desc.numAudioOutputs.value() > 0;
                if (!hasLegacyChannels) {
                    std::ostringstream msg;
                    msg << "Missing audio channel metadata for node " << desc.nodeId
                        << " (kind: " << nodeKindToString(desc.kind) << ") - defaulting to stereo";
                    LOG_DEBUG({"GraphEngine", "Snapshot", "Channels"}, msg.str());
                }
            }
        }

        auto node = createNode(desc, pluginHost);
        if (node) {
            const bool isPluginNode =
                desc.kind == NodeKind::MidiFx ||
                desc.kind == NodeKind::Instrument ||
                desc.kind == NodeKind::AudioFx;
            if (isPluginNode) {
                const auto* pluginNode = dynamic_cast<const PluginNode*>(node.get());
                if (pluginNode &&
                    desc.pluginId.has_value() &&
                    !desc.pluginId->empty() &&
                    pluginNode->getPlugin() == nullptr) {
                    UnavailablePluginNode unavailable;
                    unavailable.nodeId = desc.nodeId;
                    unavailable.pluginFormat = desc.pluginFormat;
                    unavailable.pluginId = desc.pluginId.value_or("");
                    unavailable.reason = "instance_create_failed";
                    _unavailablePluginNodes.push_back(std::move(unavailable));
                }
            }

            // For HardwareAudioOutputNode, set EngineHost reference before assigning config
            // (HardwareAudioOutputNode needs it to query device channel count)
            if (desc.kind == NodeKind::HardwareAudioOutput) {
                auto* outputNode = dynamic_cast<HardwareAudioOutputNode*>(node.get());
                if (outputNode && engineHost) {
                    outputNode->setEngineHost(engineHost);
                }
            }

            // Assign channel configuration from NodeDesc
            // For HardwareAudioOutputNode, this will be overridden in prepare() with actual device channel count
            NodeAudioConfig config = createAudioConfigFromDesc(desc, node.get());
            node->setAudioConfig(config);
            _nodes[desc.nodeId] = std::move(node);
        } else {
            LOG_WARN({"GraphEngine", "Graph"}, std::string("Warning: Failed to create node: ") + desc.nodeId);
        }
    }

    // Process connections and validate channel compatibility
    for (const auto& connDesc : snapshot.connections) {
        if (connDesc.fromStreamId.has_value()) {
            // Stream input binding (validated during Source/Input Pass)
            StreamInputBinding binding;
            binding.streamId = connDesc.fromStreamId.value();
            binding.targetNodeId = connDesc.toNodeId;
            binding.targetInputIndex = connDesc.toInputIndex;
            _streamBindings.push_back(binding);
        } else if (connDesc.fromNodeId.has_value()) {
            // Add connection (validation happens later in validateRouting())
            GraphConnection conn;
            conn.fromNodeId = connDesc.fromNodeId.value();
            conn.fromOutputIndex = connDesc.fromOutputIndex;
            conn.toNodeId = connDesc.toNodeId;
            conn.toInputIndex = connDesc.toInputIndex;
            conn.isValid = true; // Will be validated in validateRouting()
            _connections.push_back(conn);
        }
    }

    // Build adjacency list and compute execution order
    buildAdjacencyList();
    computeExecutionOrder();
    buildIncomingConnections();

    // Validate routing rules (channel compatibility, layout rules)
    validateRouting();

    // Log channel validation summary
    int nodesWithAudioMetadata = 0;
    int nodesWithLegacyMetadata = 0;
    int nodesMissingMetadata = 0;
    for (const auto& desc : snapshot.nodes) {
        bool requiresAudioChannels = (
            desc.kind == NodeKind::AudioLane ||
            desc.kind == NodeKind::Instrument ||
            desc.kind == NodeKind::AudioFx ||
            desc.kind == NodeKind::Fader ||
            desc.kind == NodeKind::Receive ||
            desc.kind == NodeKind::HardwareAudioOutput ||
            desc.kind == NodeKind::HardwareAudioInput
        );
        if (requiresAudioChannels) {
            // Check for explicit audio metadata (numOutputs > 0 indicates valid metadata)
            if (desc.audio.has_value() && desc.audio->numOutputs > 0) {
                nodesWithAudioMetadata++;
            } else if (desc.numAudioOutputs.has_value() && desc.numAudioOutputs.value() > 0) {
                nodesWithLegacyMetadata++;
            } else {
                nodesMissingMetadata++;
            }
        }
    }
    if (nodesWithAudioMetadata > 0 || nodesWithLegacyMetadata > 0 || nodesMissingMetadata > 0) {
        std::ostringstream msg;
        msg << "Channel metadata: " << nodesWithAudioMetadata << " nodes with explicit audio.channels, "
            << nodesWithLegacyMetadata << " nodes with legacy numAudioOutputs, "
            << nodesMissingMetadata << " nodes missing metadata";
        LOG_DEBUG({"GraphEngine", "Snapshot", "Channels"}, msg.str());
    }

    // Log channel configuration summary
    std::ostringstream channelSummary;
    int monoNodes = 0;
    int stereoNodes = 0;
    int multiChannelNodes = 0;
    for (const auto& pair : _nodes) {
        const auto& config = pair.second->getAudioConfig();
        if (config.numOutputChannels == 1) {
            monoNodes++;
        } else if (config.numOutputChannels == 2) {
            stereoNodes++;
        } else if (config.numOutputChannels > 2) {
            multiChannelNodes++;
        }
    }
    if (monoNodes > 0 || stereoNodes > 0 || multiChannelNodes > 0) {
        channelSummary << "Channel config: " << monoNodes << " mono, " << stereoNodes << " stereo, " << multiChannelNodes << " multi-channel";
        LOG_DEBUG({"GraphEngine", "ChannelConfig"}, channelSummary.str());
    }

    // Validate runtime config matches snapshot (where applicable)
    for (const auto& desc : snapshot.nodes) {
        auto* node = findNode(desc.nodeId);
        if (node) {
            const auto& config = node->getAudioConfig();
            // Check if snapshot specified channel counts (prefer audio.numOutputs, fall back to legacy)
            int snapshotInputs = 0;
            int snapshotOutputs = 0;
            if (desc.audio.has_value()) {
                snapshotInputs = static_cast<int>(desc.audio->numInputs);
                snapshotOutputs = static_cast<int>(desc.audio->numOutputs);
            } else {
                if (desc.numAudioInputs.has_value()) {
                    snapshotInputs = static_cast<int>(desc.numAudioInputs.value());
                }
                if (desc.numAudioOutputs.has_value()) {
                    snapshotOutputs = static_cast<int>(desc.numAudioOutputs.value());
                }
            }

            // Check if snapshot specified channel counts that differ from runtime config
            if (snapshotInputs > 0 && snapshotInputs != config.numInputChannels) {
                std::ostringstream msg;
                msg << "Input channel count mismatch for node " << desc.nodeId
                    << ": snapshot=" << snapshotInputs
                    << ", runtime=" << config.numInputChannels;
                LOG_WARN({"GraphEngine", "Snapshot", "Channels"}, msg.str());
            }
            if (snapshotOutputs > 0 && snapshotOutputs != config.numOutputChannels) {
                std::ostringstream msg;
                msg << "Output channel count mismatch for node " << desc.nodeId
                    << ": snapshot=" << snapshotOutputs
                    << ", runtime=" << config.numOutputChannels;
                LOG_WARN({"GraphEngine", "Snapshot", "Channels"}, msg.str());
            }

            // Legacy check for numAudioInputs (for backwards compatibility)
            if (desc.numAudioInputs.has_value()) {
                int snapshotInputs = static_cast<int>(desc.numAudioInputs.value());
                if (snapshotInputs != config.numInputChannels) {
                    std::ostringstream msg;
                    msg << "Input channel mismatch for node " << desc.nodeId
                        << ": snapshot=" << snapshotInputs
                        << ", runtime=" << config.numInputChannels;
                    LOG_WARN({"GraphEngine", "ChannelConfig"}, msg.str());
                }
            }
            if (desc.numAudioOutputs.has_value()) {
                int snapshotOutputs = static_cast<int>(desc.numAudioOutputs.value());
                if (snapshotOutputs != config.numOutputChannels) {
                    std::ostringstream msg;
                    msg << "Output channel mismatch for node " << desc.nodeId
                        << ": snapshot=" << snapshotOutputs
                        << ", runtime=" << config.numOutputChannels;
                    LOG_WARN({"GraphEngine", "ChannelConfig"}, msg.str());
                }
            }
        }
    }

    // Update live inputs/monitors flag based on graph structure
    // Check for hardware input nodes, or instrument nodes that might receive live MIDI
    bool hasLiveInputs = false;
    for (const auto& pair : _nodes) {
        const GraphNode* node = pair.second.get();
        if (!node) continue;

        NodeKind kind = node->getKind();
        if (kind == NodeKind::HardwareAudioInput || kind == NodeKind::HardwareMidiInput) {
            hasLiveInputs = true;
            break;
        }

        // Instrument nodes that receive MIDI from HardwareMidiInput nodes also need live processing
        // We check if there's a path from HardwareMidiInput to Instrument
        if (kind == NodeKind::Instrument) {
            // Check if this instrument has incoming connections from HardwareMidiInput nodes
            auto it = _incomingConnections.find(node->getId());
            if (it != _incomingConnections.end()) {
                for (const auto& conn : it->second) {
                    GraphNode* fromNode = findNode(conn.fromNodeId);
                    if (fromNode && fromNode->getKind() == NodeKind::HardwareMidiInput) {
                        hasLiveInputs = true;
                        break;
                    }
                }
            }
            if (hasLiveInputs) break;
        }
    }
    setLiveInputsOrMonitorsActive(hasLiveInputs);

    std::ostringstream msg;
    msg << "Graph loaded: " << _nodes.size() << " nodes, "
        << _connections.size() << " connections, "
        << _streamBindings.size() << " stream bindings, "
        << _executionOrder.size() << " nodes in execution order";
    LOG_INFO({"GraphEngine", "Graph"}, msg.str());
}

