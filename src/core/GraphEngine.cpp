#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "core/StreamScheduler.hpp"
#include "core/NodeProcessContext.hpp"
#include "core/NodeAudioConfig.hpp"
#include "logging/Logging.hpp"
#include <queue>
#include <algorithm>
#include <cmath>
#include <sstream>

GraphEngine::GraphEngine() : _pluginHost(nullptr), _hasLiveInputsOrMonitors(false) {
    LOG_DEBUG({"GraphEngine", "Lifecycle"}, "Created");
}

GraphEngine::~GraphEngine() {
    clear();
    LOG_DEBUG({"GraphEngine", "Lifecycle"}, "Destroyed");
}

void GraphEngine::loadGraphSnapshot(const GraphSnapshot& snapshot, PluginHost* pluginHost, EngineHost* engineHost) {
    // Clear existing graph
    clear();

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

void GraphEngine::prepareGraph(int sampleRate, int maxBlockSize) {
    std::ostringstream prepareMsg;
    prepareMsg << "Preparing graph (sampleRate=" << sampleRate
        << ", maxBlockSize=" << maxBlockSize << ")";
    LOG_INFO({"GraphEngine", "Graph"}, prepareMsg.str());

    // Call prepare() on all nodes in execution order
    for (GraphNode* node : _executionOrder) {
        if (node) {
            node->prepare(sampleRate, static_cast<int>(maxBlockSize));
        }
    }

    LOG_INFO({"GraphEngine", "Graph"}, "Graph prepared");
}

const std::vector<GraphNode*>& GraphEngine::getExecutionOrder() const noexcept {
    return _executionOrder;
}

const std::vector<StreamInputBinding>& GraphEngine::getStreamBindings() const noexcept {
    return _streamBindings;
}

GraphNode* GraphEngine::findNode(const NodeId& id) noexcept {
    auto it = _nodes.find(id);
    return (it != _nodes.end()) ? it->second.get() : nullptr;
}

const GraphNode* GraphEngine::findNode(const NodeId& id) const noexcept {
    auto it = _nodes.find(id);
    return (it != _nodes.end()) ? it->second.get() : nullptr;
}

bool GraphEngine::hasGraph() const noexcept {
    return !_nodes.empty();
}

std::unordered_map<NodeId, std::vector<std::uint8_t>> GraphEngine::capturePluginStateChunks() const {
    std::unordered_map<NodeId, std::vector<std::uint8_t>> result;

    for (const auto& entry : _nodes) {
        const auto* pluginNode = dynamic_cast<const PluginNode*>(entry.second.get());
        if (!pluginNode) {
            continue;
        }

        auto chunk = pluginNode->getStateChunk();
        if (chunk.empty()) {
            continue;
        }

        result.emplace(entry.first, std::move(chunk));
    }

    return result;
}

void GraphEngine::clear() {
    _nodes.clear();
    _connections.clear();
    _streamBindings.clear();
    _executionOrder.clear();
    _adjacencyList.clear();
    _inDegree.clear();
    _incomingConnections.clear();
    setLiveInputsOrMonitorsActive(false);
}

int GraphEngine::getTotalLatencyInSamples() const noexcept {
    // Stub implementation: simple sum of all node latencies
    // TODO: Future implementation should:
    //   - Only consider nodes on the actual signal path (from sources to HardwareAudioOutputNode)
    //   - Account for parallel paths and take maximum latency per path
    //   - Consider that latency accumulates along the signal path
    int totalLatency = 0;
    for (const auto& pair : _nodes) {
        totalLatency += pair.second->getLatencyInSamples();
    }
    return totalLatency;
}

int GraphEngine::getMaxTailInSamples() const noexcept {
    // Stub implementation: simple maximum of all node tails
    // TODO: Future implementation should:
    //   - Only consider nodes on the actual signal path
    //   - Account for parallel paths and take maximum tail per path
    //   - Consider tail propagation through the graph (tail may extend through downstream nodes)
    int maxTail = 0;
    for (const auto& pair : _nodes) {
        maxTail = std::max(maxTail, pair.second->getTailInSamples());
    }
    return maxTail;
}

bool GraphEngine::hasActiveTails() const noexcept {
    // Stub implementation: always returns false for now
    // TODO: Future implementation should track actual tail state:
    //   - Monitor plugin tail outputs after playback stops
    //   - Return true if any plugin is still producing non-silent output
    //   - Return false once all tails have decayed to silence
    // This is a placeholder for future tail rendering logic
    return false;
}

bool GraphEngine::hasLiveInputsOrMonitors() const noexcept {
    return _hasLiveInputsOrMonitors.load(std::memory_order_acquire);
}

void GraphEngine::setLiveInputsOrMonitorsActive(bool active) noexcept {
    _hasLiveInputsOrMonitors.store(active, std::memory_order_release);
}

NodeAudioConfig GraphEngine::createAudioConfigFromDesc(const NodeDesc& desc, GraphNode* node) {
    // Start with node's current config (may have been set by constructor)
    NodeAudioConfig config = node->getAudioConfig();

    // Prefer explicit audio.numInputs/numOutputs metadata over legacy numAudioInputs/numAudioOutputs
    if (desc.audio.has_value()) {
        // Use separate input/output channel counts from audio metadata
        if (desc.audio->numInputs > 0) {
            config.numInputChannels = static_cast<int>(desc.audio->numInputs);
        }
        if (desc.audio->numOutputs > 0) {
            config.numOutputChannels = static_cast<int>(desc.audio->numOutputs);
        }
    } else {
        // Fall back to legacy fields for backwards compatibility
        if (desc.numAudioInputs.has_value()) {
            config.numInputChannels = static_cast<int>(desc.numAudioInputs.value());
        }
        if (desc.numAudioOutputs.has_value()) {
            config.numOutputChannels = static_cast<int>(desc.numAudioOutputs.value());
        }
    }

    // Apply node-type-specific defaults and rules
    switch (desc.kind) {
        case NodeKind::MidiLane:
            // MIDI lanes have no audio I/O
            config.numInputChannels = 0;
            config.numOutputChannels = 0;
            config.layout = ChannelLayout::Mono; // Not meaningful, but set for consistency
            break;

        case NodeKind::AudioLane:
            // Audio lanes: output channels from asset/schedule, default to stereo
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 2; // Default stereo
                config.layout = ChannelLayout::Stereo;
            } else {
                // Determine layout from channel count
                if (config.numOutputChannels == 1) {
                    config.layout = ChannelLayout::Mono;
                } else {
                    config.layout = ChannelLayout::Stereo; // Default for 2+ channels
                }
            }
            config.numInputChannels = 0; // No inputs (reads from schedule)
            break;

        case NodeKind::MidiFx:
            // MIDI FX: typically no audio, but may have passthrough
            // Use values from NodeDesc or defaults
            if (!desc.numAudioInputs.has_value()) {
                config.numInputChannels = 0;
            }
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 0;
            }
            config.layout = (config.numOutputChannels > 0) ? ChannelLayout::Stereo : ChannelLayout::Mono;
            break;

        case NodeKind::Instrument:
            // Instruments: MIDI in, audio out (typically stereo)
            config.numInputChannels = 0; // No audio input
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 2; // Default stereo
                config.layout = ChannelLayout::Stereo;
            } else {
                config.layout = (config.numOutputChannels == 1) ? ChannelLayout::Mono : ChannelLayout::Stereo;
            }
            break;

        case NodeKind::AudioFx:
            // Audio FX: audio in/out (typically stereo)
            if (!desc.numAudioInputs.has_value()) {
                config.numInputChannels = 2; // Default stereo
            }
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 2; // Default stereo
            }
            // Ensure input matches output for FX (unless explicitly configured otherwise)
            if (!desc.numAudioInputs.has_value() && !desc.numAudioOutputs.has_value()) {
                config.numInputChannels = config.numOutputChannels;
            }
            config.layout = (config.numOutputChannels == 1) ? ChannelLayout::Mono : ChannelLayout::Stereo;
            break;

        case NodeKind::Send:
        case NodeKind::Receive:
            // Send/Receive: pass through channels from connections
            // Default to stereo if not specified
            if (!desc.numAudioInputs.has_value()) {
                config.numInputChannels = 2;
            }
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 2;
            }
            // Ensure input matches output for routing nodes
            if (!desc.numAudioInputs.has_value() && !desc.numAudioOutputs.has_value()) {
                config.numInputChannels = config.numOutputChannels;
            }
            config.layout = (config.numOutputChannels == 1) ? ChannelLayout::Mono : ChannelLayout::Stereo;
            break;

        case NodeKind::Fader:
            // Fader: typically stereo output
            if (!desc.numAudioInputs.has_value()) {
                config.numInputChannels = 2; // Default stereo
            }
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 2; // Default stereo
            }
            config.layout = (config.numOutputChannels == 1) ? ChannelLayout::Mono : ChannelLayout::Stereo;
            break;

        case NodeKind::HardwareAudioOutput:
            // Hardware audio output: channel count will be set from actual device in prepare()
            // Use default stereo for now (will be updated when HardwareAudioOutputNode::prepare() is called)
            if (!desc.numAudioInputs.has_value()) {
                config.numInputChannels = 2; // Default stereo (will be overridden in prepare())
            }
            if (!desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = 2; // Default stereo (will be overridden in prepare())
            }
            config.layout = (config.numOutputChannels == 1) ? ChannelLayout::Mono : ChannelLayout::Stereo;
            break;

        case NodeKind::HardwareAudioInput:
            // AudioInput: already set in constructor (mono by default)
            // But allow override from NodeDesc
            if (desc.numAudioOutputs.has_value()) {
                config.numOutputChannels = static_cast<int>(desc.numAudioOutputs.value());
                config.layout = (config.numOutputChannels == 1) ? ChannelLayout::Mono : ChannelLayout::Stereo;
            }
            break;

        case NodeKind::HardwareMidiInput:
            // MIDI input: no audio
            config.numInputChannels = 0;
            config.numOutputChannels = 0;
            config.layout = ChannelLayout::Mono; // Not meaningful
            break;

        default:
            // Unknown node type: use defaults
            break;
    }

    // Validate channel counts (must be non-negative)
    if (config.numInputChannels < 0) {
        LOG_WARN({"GraphEngine", "ChannelConfig"}, std::string("Invalid input channel count for node: ") + desc.nodeId);
        config.numInputChannels = 0;
    }
    if (config.numOutputChannels < 0) {
        LOG_WARN({"GraphEngine", "ChannelConfig"}, std::string("Invalid output channel count for node: ") + desc.nodeId);
        config.numOutputChannels = 0;
    }

    return config;
}

std::unique_ptr<GraphNode> GraphEngine::createNode(const NodeDesc& desc, PluginHost* pluginHost) {
    std::string trackId = desc.trackId.value_or("");
    std::string laneId = desc.laneId.value_or("");
    std::string pluginId = desc.pluginId.value_or("");

    switch (desc.kind) {
        case NodeKind::MidiLane:
            return std::make_unique<MidiLaneNode>(desc.nodeId, trackId, laneId);

        case NodeKind::AudioLane:
            return std::make_unique<AudioLaneNode>(desc.nodeId, trackId, laneId);

        case NodeKind::MidiFx:
            return std::make_unique<PluginNode>(PluginNodeKind::MidiFx, desc.nodeId, trackId, desc, pluginHost);

        case NodeKind::Instrument:
            return std::make_unique<PluginNode>(PluginNodeKind::Instrument, desc.nodeId, trackId, desc, pluginHost);

        case NodeKind::AudioFx:
            return std::make_unique<PluginNode>(PluginNodeKind::AudioFx, desc.nodeId, trackId, desc, pluginHost);

        case NodeKind::Send:
            return std::make_unique<SendNode>(desc.nodeId, trackId, pluginId); // Using pluginId as busId for now

        case NodeKind::Fader: {
            auto node = std::make_unique<FaderNode>(desc.nodeId, trackId);

            // Initialise mix state from snapshot metadata, if present.
            if (desc.mix.has_value()) {
                const auto& mix = desc.mix.value();

                // Apply gain. Mute is handled via `node.setParameter` (`muted`).
                if (mix.gain.has_value()) {
                    node->setGain(mix.gain.value());
                }
                // Solo semantics and effective mute remain coordinated by Pulse; Signal only
                // consumes the computed effective mute state.
            }

            if (desc.spatial.has_value()) {
                const auto& spatial = desc.spatial.value();
                const bool enabled = spatial.enabled.value_or(false);
                if (enabled && spatial.adapter.has_value()) {
                    const auto& adapter = spatial.adapter.value();
                    if (adapter == "perChannelGain") {
                        node->setSpatialAdapter(FaderNode::SpatialAdapter::PerChannelGain);
                    } else if (adapter == "balance") {
                        node->setSpatialAdapter(FaderNode::SpatialAdapter::Balance);
                    }
                }
            }

            return node;
        }

        case NodeKind::Receive:
            return std::make_unique<ReceiveNode>(desc.nodeId, pluginId); // Using pluginId as receiveName for now

        case NodeKind::HardwareAudioOutput: {
            auto outputNode = std::make_unique<HardwareAudioOutputNode>(desc.nodeId);
            // HardwareAudioOutputNode needs EngineHost reference to query device channel count
            // This will be set after node creation if EngineHost is available
            return outputNode;
        }

        case NodeKind::HardwareAudioInput:
            return std::make_unique<AudioInputNode>(desc.nodeId, desc.deviceId.value_or(""), desc.inputChannelIndex.value_or(0));

        case NodeKind::HardwareMidiInput:
            return std::make_unique<MidiInputNode>(desc.nodeId, desc.portId.value_or(""));

        default:
            std::ostringstream msg;
            msg << "Warning: Unknown node kind for node: " << desc.nodeId;
            LOG_WARN({"GraphEngine"}, msg.str());
            return nullptr;
    }
}

void GraphEngine::buildAdjacencyList() {
    _adjacencyList.clear();
    _inDegree.clear();

    // Initialize in-degree for all nodes
    for (const auto& pair : _nodes) {
        _inDegree[pair.first] = 0;
        _adjacencyList[pair.first] = std::vector<NodeId>();
    }

    // Build adjacency list from connections
    for (const auto& conn : _connections) {
        // Verify both nodes exist
        if (_nodes.find(conn.fromNodeId) == _nodes.end() ||
            _nodes.find(conn.toNodeId) == _nodes.end()) {
            std::ostringstream msg;
            msg << "Warning: Connection references non-existent node: "
                << conn.fromNodeId << " -> " << conn.toNodeId;
            LOG_WARN({"GraphEngine"}, msg.str());
            continue;
        }

        // Add edge: fromNodeId -> toNodeId
        _adjacencyList[conn.fromNodeId].push_back(conn.toNodeId);
        _inDegree[conn.toNodeId]++;
    }
}

void GraphEngine::computeExecutionOrder() {
    _executionOrder.clear();

    // Kahn's algorithm for topological sort
    std::queue<NodeId> queue;

    // Find all nodes with in-degree 0 (sources)
    for (const auto& pair : _inDegree) {
        if (pair.second == 0) {
            queue.push(pair.first);
        }
    }

    // Process nodes in topological order
    while (!queue.empty()) {
        NodeId current = queue.front();
        queue.pop();

        // Add to execution order
        auto it = _nodes.find(current);
        if (it != _nodes.end()) {
            _executionOrder.push_back(it->second.get());
        }

        // Process all outgoing edges
        auto adjIt = _adjacencyList.find(current);
        if (adjIt != _adjacencyList.end()) {
            for (const NodeId& neighbor : adjIt->second) {
                _inDegree[neighbor]--;
                if (_inDegree[neighbor] == 0) {
                    queue.push(neighbor);
                }
            }
        }
    }

    // Check for cycles (if there are nodes not in execution order, there's a cycle)
    if (_executionOrder.size() < _nodes.size()) {
        std::ostringstream msg;
        msg << "Warning: Graph contains cycles or disconnected nodes. "
            << "Execution order has " << _executionOrder.size()
            << " nodes, but graph has " << _nodes.size() << " nodes.";
        LOG_WARN({"GraphEngine"}, msg.str());

        // Add remaining nodes in arbitrary order (fallback)
        for (const auto& pair : _nodes) {
            // Check if node is already in execution order
            bool found = false;
            for (GraphNode* node : _executionOrder) {
                if (node->getId() == pair.first) {
                    found = true;
                    break;
                }
            }
            if (!found) {
                _executionOrder.push_back(pair.second.get());
            }
        }
    }
}

GraphEngine::ChannelCompatibility GraphEngine::checkChannelCompatibility(
    const GraphNode& source,
    const GraphNode& dest
) {
    ChannelCompatibility result;
    const auto& sourceConfig = source.getAudioConfig();
    const auto& destConfig = dest.getAudioConfig();

    result.sourceChannels = sourceConfig.numOutputChannels;
    result.destChannels = destConfig.numInputChannels;

    // Compatible if:
    // - Both have audio channels (non-zero)
    // - Channel counts match exactly
    result.isCompatible = (
        result.sourceChannels > 0 &&
        result.destChannels > 0 &&
        result.sourceChannels == result.destChannels
    );

    // Mismatch if:
    // - Both have audio channels but counts don't match
    result.isMismatch = (
        result.sourceChannels > 0 &&
        result.destChannels > 0 &&
        result.sourceChannels != result.destChannels
    );

    return result;
}

void GraphEngine::buildIncomingConnections() {
    // Pre-compute incoming connections per node for efficient routing
    _incomingConnections.clear();

    for (const auto& conn : _connections) {
        // Only include valid connections in routing
        if (conn.isValid) {
            _incomingConnections[conn.toNodeId].push_back(conn);
        }
    }
}

void GraphEngine::validateRouting() {
    // Validate all node-to-node connections according to routing rules
    // Rules:
    // 1. Channel count equality: fromNode.outputChannels == toNode.inputChannels
    // 2. Audio-only nodes cannot connect to MIDI-only nodes (and vice versa)
    // 3. HardwareAudioOutputNode must accept layouts matching hardware (for now, stereo only)
    // 4. MidiLaneNode has 0 audio channels - audio connections are invalid

    int invalidConnections = 0;

    for (auto& conn : _connections) {
        // Skip if already marked invalid
        if (!conn.isValid) {
            continue;
        }

        GraphNode* fromNode = findNode(conn.fromNodeId);
        GraphNode* toNode = findNode(conn.toNodeId);

        if (!fromNode || !toNode) {
            // Node not found - mark as invalid
            conn.isValid = false;
            invalidConnections++;
            std::ostringstream msg;
            msg << "Invalid connection: node not found - "
                << (fromNode ? conn.toNodeId : conn.fromNodeId);
            LOG_ERROR({"GraphEngine", "Routing"}, msg.str());
            continue;
        }

        const auto& fromConfig = fromNode->getAudioConfig();
        const auto& toConfig = toNode->getAudioConfig();

        // Rule 1: Channel count equality for audio connections
        // Exception: HardwareAudioOutputNode can handle mismatches (handled in Rule 3)
        ChannelCompatibility compat = checkChannelCompatibility(*fromNode, *toNode);
        bool isHardwareOutputNode = (toNode->getKind() == NodeKind::HardwareAudioOutput);

        if (compat.isMismatch && !isHardwareOutputNode) {
            // Channel mismatch - mark connection as invalid (unless it's a hardware output node)
            conn.isValid = false;
            invalidConnections++;
            std::ostringstream msg;
            msg << "Channel mismatch in connection: " << conn.fromNodeId
                << " (" << compat.sourceChannels << " ch) -> "
                << conn.toNodeId << " (" << compat.destChannels << " ch)";
            // Add node kind information for better diagnostics
            std::string fromKind = nodeKindToString(fromNode->getKind());
            std::string toKind = nodeKindToString(toNode->getKind());
            msg << " (from " << fromKind << " to " << toKind << ")";
            LOG_ERROR({"GraphEngine", "Routing"}, msg.str());
            continue;
        } else if (compat.isCompatible) {
            // Log info for compatible connections (debug level)
            LOG_DEBUG({"GraphEngine", "Routing"},
                std::string("Compatible connection: ") + conn.fromNodeId +
                " (" + std::to_string(compat.sourceChannels) + " ch) -> " +
                conn.toNodeId + " (" + std::to_string(compat.destChannels) + " ch)");
        }

        // Rule 2: Audio-only nodes cannot connect to MIDI-only nodes
        // (MIDI routing is separate and always valid)
        // This is implicitly handled by channel count check above

        // Rule 3: HardwareAudioOutputNode validation
        // HardwareAudioOutputNode channel count comes from actual hardware device
        // HardwareAudioOutputNode can handle channel mismatches (expansion/truncation), so we log warnings but don't mark as invalid
        if (toNode->getKind() == NodeKind::HardwareAudioOutput) {
            // HardwareAudioOutputNode channel count is set from actual device in prepare()
            // HardwareAudioOutputNode will handle channel expansion/truncation in process()
            if (compat.isMismatch) {
                std::ostringstream msg;
                msg << "HardwareAudioOutputNode " << conn.toNodeId
                    << " channel mismatch: upstream has " << compat.sourceChannels
                    << " ch, device has " << compat.destChannels << " ch";
                if (compat.sourceChannels < compat.destChannels) {
                    msg << " (will expand: duplicate channels)";
                } else {
                    msg << " (will truncate: drop extra channels)";
                }
                LOG_WARN({"GraphEngine", "Routing", "HardwareAudioOutput"}, msg.str());
                // Connection is still valid - HardwareAudioOutputNode handles the mismatch
                conn.isValid = true;
            } else if (compat.isCompatible) {
                LOG_DEBUG({"GraphEngine", "Routing", "HardwareAudioOutput"},
                    std::string("HardwareAudioOutputNode ") + conn.toNodeId + " routing compatible: " +
                    std::to_string(compat.destChannels) + " channels");
            }
        }

        // Rule 4: MidiLaneNode has 0 audio channels
        if (fromNode->getKind() == NodeKind::MidiLane) {
            if (fromConfig.numOutputChannels > 0) {
                // MidiLaneNode should have 0 audio channels
                // This shouldn't happen if config is set correctly, but check anyway
                std::ostringstream msg;
                msg << "MidiLaneNode " << conn.fromNodeId
                    << " has audio channels (" << fromConfig.numOutputChannels
                    << ") - audio connection invalid";
                LOG_ERROR({"GraphEngine", "Routing"}, msg.str());
                conn.isValid = false;
                invalidConnections++;
                continue;
            }
        }

        // Rule 5: Mixer-related node validation (SendNode, ReceiveNode, FaderNode)
        // These nodes participate in routing and must have compatible channel counts
        bool isMixerNode = (
            fromNode->getKind() == NodeKind::Send ||
            toNode->getKind() == NodeKind::Receive ||
            fromNode->getKind() == NodeKind::Fader ||
            toNode->getKind() == NodeKind::Fader
        );

        if (isMixerNode && compat.isCompatible) {
            // Log routing compatibility at debug level
            LOG_DEBUG({"GraphEngine", "Routing", "Mixer"},
                std::string("Mixer routing compatible: ") + conn.fromNodeId +
                " (" + std::to_string(compat.sourceChannels) + " ch) -> " +
                conn.toNodeId + " (" + std::to_string(compat.destChannels) + " ch)");
        }
    }

    // Count compatible connections for summary
    int compatibleConnections = 0;
    for (const auto& conn : _connections) {
        if (conn.isValid) {
            GraphNode* fromNode = findNode(conn.fromNodeId);
            GraphNode* toNode = findNode(conn.toNodeId);
            if (fromNode && toNode) {
                ChannelCompatibility compat = checkChannelCompatibility(*fromNode, *toNode);
                if (compat.isCompatible) {
                    compatibleConnections++;
                }
            }
        }
    }

    // Log summary
    if (invalidConnections > 0) {
        std::ostringstream msg;
        msg << "Routing validation: " << compatibleConnections << " compatible, "
            << invalidConnections << " invalid connection(s) marked and will be skipped at render time";
        LOG_WARN({"GraphEngine", "Routing"}, msg.str());
    } else {
        std::ostringstream msg;
        msg << "Routing validation: " << compatibleConnections << " compatible connection(s)";
        LOG_DEBUG({"GraphEngine", "Routing"}, msg.str());
    }
}

void GraphEngine::processGraph(EngineRenderContext& ctx) {
    // Real-time safe: no allocations, no logging, deterministic processing order
    // Note: Source/Input Pass must be called separately before this function

    // 1. Build NodeProcessContext for this block
    NodeProcessContext npc;
    npc.sampleRate = static_cast<int>(ctx.sampleRate);
    npc.blockSize = ctx.blockSize;
    npc.blockStartSample = ctx.playheadSamples;
    npc.tempo = ctx.tempo;
    npc.isPlaying = ctx.isPlaying;
    npc.loopEnabled = ctx.loopEnabled;
    npc.loopStartBeats = ctx.loopStartBeats;
    npc.loopEndBeats = ctx.loopEndBeats;

    // 2. Process nodes in execution order
    // For each node: route inputs, process
    // Note: Source/Input Pass already populated outputs for source/input nodes
    for (GraphNode* node : _executionOrder) {
        if (!node) continue;

        // Clear input buffers at the start of the block; they will be
        // repopulated via routing from upstream nodes.
        node->io.audioIn.clear();
        node->io.midiIn.clear();

        // Clear output buffers (will be filled by processing).
        // Exception: source/input nodes have outputs populated by
        // Source/Input Pass, so don't clear them here.
        NodeKind kind = node->getKind();
        if (
            kind != NodeKind::AudioLane &&
            kind != NodeKind::MidiLane &&
            kind != NodeKind::HardwareAudioInput &&
            kind != NodeKind::HardwareMidiInput
        ) {
            node->io.audioOut.clear();
            node->io.midiOut.clear();
        }

        // Route connections from upstream nodes to this node (fan-in)
        // Real-time safe: no allocations, uses pre-computed connection list
        // Only valid connections are included in _incomingConnections (filtered during buildIncomingConnections)
        auto it = _incomingConnections.find(node->getId());
        if (it != _incomingConnections.end()) {
            const auto& config = node->getAudioConfig();
            for (const auto& conn : it->second) {
                GraphNode* fromNode = findNode(conn.fromNodeId);
                if (fromNode) {
                    const auto& fromConfig = fromNode->getAudioConfig();

                    // Audio routing: sum outputs (channel-aware summing with upmix/downmix support)
                    // Note: GraphEngine validation ensures most connections are compatible, but
                    // sumFrom() can handle channel mismatches gracefully (upmix/downmix)
                    if (fromConfig.numOutputChannels > 0 && config.numInputChannels > 0) {
                        // Channel-aware summing: sumFrom() handles mismatches with upmix/downmix rules
                        node->io.audioIn.sumFrom(fromNode->io.audioOut);
                        // If channel counts don't match, sumFrom() will apply upmix/downmix rules:
                        // - Upmix: duplicate last source channel to fill target channels
                        // - Downmix: truncate extra source channels (sum first N channels)
                    }

                    // MIDI routing: append messages
                    node->io.midiIn.append(fromNode->io.midiOut);
                }
            }
        }

        // Process node with NodeProcessContext
        node->process(npc);
    }
}


/// Source/Input Pass - Unified injection of schedule data and hardware input
///
/// This pass runs once per render block, before the main node processing loop.
/// It populates source and input node outputs with:
/// - Schedule data (audio segments, MIDI events) for lane nodes
/// - Hardware input (audio, MIDI) for input nodes
///
/// Real-time safety:
/// - No allocations (uses pre-allocated buffers and vectors)
/// - No locks (read-only access to scheduler, asset source)
/// - No logging (silent operation)
/// - Deterministic execution order
void GraphEngine::runSourceInputPass(
    const EngineRenderContext& ctx,
    const StreamScheduler* scheduler,
    AudioAssetSource* assetSource,
    const float* hardwareAudioInput,
    int hardwareAudioChannels,
    int hardwareAudioFrames,
    const std::vector<MidiMessage>& hardwareMidiInput
) {
    // Real-time safe: no allocations, no logging, deterministic

    uint64_t blockStartSamples = ctx.playheadSamples;
    uint64_t blockEndSamples = blockStartSamples + ctx.blockSize;

    // Part 1: Inject schedule data into lane nodes (only when playing)
    if (ctx.isPlaying && scheduler && assetSource) {
        const ScheduleData* schedule = scheduler->getSchedule();
        if (schedule) {
            // Inject data for each stream binding
            for (const auto& binding : _streamBindings) {
                GraphNode* targetNode = findNode(binding.targetNodeId);
                if (!targetNode) {
                    continue;
                }

                // Determine node type and inject accordingly
                if (targetNode->getKind() == NodeKind::AudioLane) {
                    auto* audioLane = dynamic_cast<AudioLaneNode*>(targetNode);
                    if (audioLane) {
                        audioLane->setStreamId(binding.streamId);

                        // Clear lane output for this block; it will be repopulated
                        // from any active audio segments (or remain silent if none).
                        audioLane->io.audioOut.clear();

                        // Get active audio segments for this stream
                        auto segments = scheduler->getActiveAudioSegments(binding.streamId, blockStartSamples);

                        for (const auto* segment : segments) {
                            if (segment && segment->startSamples < blockEndSamples && segment->endSamples > blockStartSamples) {
                                audioLane->injectAudioSegment(segment, blockStartSamples, ctx.blockSize, assetSource);
                            }
                        }
                    }
                } else if (targetNode->getKind() == NodeKind::MidiLane) {
                    auto* midiLane = dynamic_cast<MidiLaneNode*>(targetNode);
                    if (midiLane) {
                        midiLane->setStreamId(binding.streamId);

                        // Get MIDI events for this stream in block range
                        auto events = scheduler->getMidiEventsInRange(binding.streamId, blockStartSamples, blockEndSamples);
                        midiLane->injectMidiEvents(events, blockStartSamples);
                    }
                }
            }
        }
    }

    // Part 2: Inject hardware input into input nodes
    // Iterate through execution order to find input nodes
    for (GraphNode* node : _executionOrder) {
        if (!node) {
            continue;
        }

        if (node->getKind() == NodeKind::HardwareAudioInput) {
            auto* inputNode = dynamic_cast<AudioInputNode*>(node);
            if (inputNode && hardwareAudioInput && hardwareAudioChannels > 0 && hardwareAudioFrames > 0) {
                // Extract channel from interleaved input buffer
                int channelIndex = inputNode->getInputChannelIndex();
                if (channelIndex < hardwareAudioChannels) {
                    inputNode->injectInputAudio(
                        hardwareAudioInput,
                        hardwareAudioChannels,
                        hardwareAudioFrames,
                        channelIndex
                    );
                }
            }
        } else if (node->getKind() == NodeKind::HardwareMidiInput) {
            auto* midiInputNode = dynamic_cast<MidiInputNode*>(node);
            if (midiInputNode) {
                // Inject MIDI from hardware input (or empty if no backend)
                midiInputNode->injectInputMidi(hardwareMidiInput);
            }
        }
    }
}
