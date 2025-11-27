#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/StreamScheduler.hpp"
#include "core/NodeProcessContext.hpp"
#include <iostream>
#include <queue>
#include <algorithm>
#include <cmath>

GraphEngine::GraphEngine() : _pluginHost(nullptr) {
    std::cout << "[GraphEngine] Created" << std::endl;
}

GraphEngine::~GraphEngine() {
    clear();
    std::cout << "[GraphEngine] Destroyed" << std::endl;
}

void GraphEngine::loadGraphSnapshot(const GraphSnapshot& snapshot, PluginHost* pluginHost) {
    // Clear existing graph
    clear();

    _pluginHost = pluginHost;

    std::cout << "[GraphEngine] Loading graph snapshot: " << snapshot.id
              << " (" << snapshot.nodes.size() << " nodes, "
              << snapshot.connections.size() << " connections)" << std::endl;

    // Create nodes
    for (const auto& desc : snapshot.nodes) {
        auto node = createNode(desc, pluginHost);
        if (node) {
            _nodes[desc.nodeId] = std::move(node);
        } else {
            std::cerr << "[GraphEngine] Warning: Failed to create node: " << desc.nodeId << std::endl;
        }
    }

    // Process connections
    for (const auto& connDesc : snapshot.connections) {
        if (connDesc.fromStreamId.has_value()) {
            // Stream input binding
            StreamInputBinding binding;
            binding.streamId = connDesc.fromStreamId.value();
            binding.targetNodeId = connDesc.toNodeId;
            binding.targetInputIndex = connDesc.toInputIndex;
            _streamBindings.push_back(binding);
        } else if (connDesc.fromNodeId.has_value()) {
            // Node-to-node connection
            GraphConnection conn;
            conn.fromNodeId = connDesc.fromNodeId.value();
            conn.fromOutputIndex = connDesc.fromOutputIndex;
            conn.toNodeId = connDesc.toNodeId;
            conn.toInputIndex = connDesc.toInputIndex;
            _connections.push_back(conn);
        }
    }

    // Build adjacency list and compute execution order
    buildAdjacencyList();
    computeExecutionOrder();

    std::cout << "[GraphEngine] Graph loaded: " << _nodes.size() << " nodes, "
              << _connections.size() << " connections, "
              << _streamBindings.size() << " stream bindings, "
              << _executionOrder.size() << " nodes in execution order" << std::endl;
}

void GraphEngine::prepareGraph(int sampleRate, int maxBlockSize) {
    std::cout << "[GraphEngine] Preparing graph (sampleRate=" << sampleRate
              << ", maxBlockSize=" << maxBlockSize << ")" << std::endl;

    // Call prepare() on all nodes in execution order
    for (GraphNode* node : _executionOrder) {
        if (node) {
            node->prepare(sampleRate, static_cast<int>(maxBlockSize));
        }
    }

    std::cout << "[GraphEngine] Graph prepared" << std::endl;
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

void GraphEngine::clear() {
    _nodes.clear();
    _connections.clear();
    _streamBindings.clear();
    _executionOrder.clear();
    _adjacencyList.clear();
    _inDegree.clear();
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
            return std::make_unique<MidiFxNode>(desc.nodeId, trackId, desc, pluginHost);

        case NodeKind::Instrument:
            return std::make_unique<InstrumentNode>(desc.nodeId, trackId, desc, pluginHost);

        case NodeKind::AudioFx:
            return std::make_unique<AudioFxNode>(desc.nodeId, trackId, desc, pluginHost);

        case NodeKind::Send:
            return std::make_unique<SendNode>(desc.nodeId, trackId, pluginId); // Using pluginId as busId for now

        case NodeKind::MixerChannel:
            return std::make_unique<MixerChannelNode>(desc.nodeId, trackId);

        case NodeKind::Receive:
            return std::make_unique<ReceiveNode>(desc.nodeId, pluginId); // Using pluginId as receiveName for now

        case NodeKind::Device:
            return std::make_unique<DeviceNode>(desc.nodeId);

        case NodeKind::AudioInput:
            return std::make_unique<AudioInputNode>(desc.nodeId, desc.deviceId.value_or(""), desc.inputChannelIndex.value_or(0));

        case NodeKind::MidiInput:
            return std::make_unique<MidiInputNode>(desc.nodeId, desc.portId.value_or(""));

        default:
            std::cerr << "[GraphEngine] Warning: Unknown node kind for node: " << desc.nodeId << std::endl;
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
            std::cerr << "[GraphEngine] Warning: Connection references non-existent node: "
                      << conn.fromNodeId << " -> " << conn.toNodeId << std::endl;
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
        std::cerr << "[GraphEngine] Warning: Graph contains cycles or disconnected nodes. "
                   << "Execution order has " << _executionOrder.size()
                   << " nodes, but graph has " << _nodes.size() << " nodes." << std::endl;

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

void GraphEngine::processGraph(EngineRenderContext& ctx, const StreamScheduler* scheduler, AudioAssetSource* assetSource) {
    // 1. Clear all node buffers
    clearAllBuffers();

    // 2. Inject stream data into lane nodes
    if (scheduler && assetSource) {
        injectStreamData(ctx, scheduler, assetSource);
    }

    // 3. Build NodeProcessContext for this block
    NodeProcessContext npc;
    npc.sampleRate = static_cast<int>(ctx.sampleRate);
    npc.blockSize = ctx.blockSize;
    npc.blockStartSample = ctx.playheadSamples;

    // Copy transport/tempo info from render context (Phase 8)
    npc.tempo = ctx.tempo;
    npc.isPlaying = ctx.isPlaying;
    npc.loopEnabled = ctx.loopEnabled;
    npc.loopStartBeats = ctx.loopStartBeats;
    npc.loopEndBeats = ctx.loopEndBeats;

    // 4. Process nodes in execution order
    // For each node, route connections from upstream nodes, then process
    for (GraphNode* node : _executionOrder) {
        if (!node) continue;

        // Route connections from upstream nodes to this node (fan-in)
        for (const auto& conn : _connections) {
            if (conn.toNodeId == node->getId()) {
                GraphNode* fromNode = findNode(conn.fromNodeId);
                if (fromNode) {
                    // Check if fromNode is a SendNode - apply send level
                    if (fromNode->getKind() == NodeKind::Send) {
                        auto* sendNode = dynamic_cast<SendNode*>(fromNode);
                        if (sendNode) {
                            float sendLevel = sendNode->getSendLevel();
                            // Apply send level when summing
                            // Create a temporary scaled buffer
                            AudioBuffer scaledBuffer;
                            scaledBuffer.resize(fromNode->io.audioOut.numChannels(), fromNode->io.audioOut.numFrames());
                            scaledBuffer.copyFrom(fromNode->io.audioOut);
                            // Scale by send level
                            for (int ch = 0; ch < scaledBuffer.numChannels(); ++ch) {
                                for (int frame = 0; frame < scaledBuffer.numFrames(); ++frame) {
                                    float sample = scaledBuffer.getSample(frame, ch) * sendLevel;
                                    scaledBuffer.setSample(frame, ch, sample);
                                }
                            }
                            // Sum scaled audio into destination
                            node->io.audioIn.sumFrom(scaledBuffer);
                        } else {
                            // Fallback: sum without scaling
                            node->io.audioIn.sumFrom(fromNode->io.audioOut);
                        }
                    } else {
                        // Normal connection: sum audio from upstream node
                        node->io.audioIn.sumFrom(fromNode->io.audioOut);
                    }
                    // Append MIDI from upstream node
                    node->io.midiIn.append(fromNode->io.midiOut);
                }
            }
        }

        // Process node with NodeProcessContext
        node->process(npc);
    }
}

void GraphEngine::clearAllBuffers() {
    for (auto& pair : _nodes) {
        if (pair.second) {
            // Clear input buffers (will be filled by routing/injection)
            pair.second->io.audioIn.clear();
            pair.second->io.midiIn.clear();
            // Clear output buffers (will be filled by processing)
            pair.second->io.audioOut.clear();
            pair.second->io.midiOut.clear();
        }
    }
}

void GraphEngine::injectStreamData(EngineRenderContext& ctx, const StreamScheduler* scheduler, AudioAssetSource* assetSource) {
    const ScheduleData* schedule = scheduler->getSchedule();
    if (!schedule || !assetSource) {
        #ifdef LOOPHOLE_ENABLE_AUDIO_DEBUG
        static int logCount = 0;
        if (logCount++ < 5) {
            if (!schedule) {
                std::cerr << "[GraphEngine] injectStreamData: schedule is null" << std::endl;
            }
            if (!assetSource) {
                std::cerr << "[GraphEngine] injectStreamData: assetSource is null" << std::endl;
            }
        }
        #endif
        return;
    }

    uint64_t blockStartSamples = ctx.playheadSamples;
    uint64_t blockEndSamples = blockStartSamples + ctx.blockSize;

    // Diagnostic: Log stream binding count
    #ifdef LOOPHOLE_ENABLE_AUDIO_DEBUG
    static int bindingLogCount = 0;
    if (bindingLogCount++ < 5) {
        std::cout << "[GraphEngine] injectStreamData: " << _streamBindings.size() << " stream binding(s)" << std::endl;
    }
    #endif

    // Inject data for each stream binding
    for (const auto& binding : _streamBindings) {
        GraphNode* targetNode = findNode(binding.targetNodeId);
        if (!targetNode) {
            #ifdef LOOPHOLE_ENABLE_AUDIO_DEBUG
            static int logCount = 0;
            if (logCount++ < 5) {
                std::cerr << "[GraphEngine] injectStreamData: target node not found: '"
                          << binding.targetNodeId << "'" << std::endl;
            }
            #endif
            continue;
        }

        // Determine node type and inject accordingly
        if (targetNode->getKind() == NodeKind::AudioLane) {
            auto* audioLane = dynamic_cast<AudioLaneNode*>(targetNode);
            if (audioLane) {
                audioLane->setStreamId(binding.streamId);

                // Get active audio segments for this stream
                auto segments = scheduler->getActiveAudioSegments(binding.streamId, blockStartSamples);

                #ifdef LOOPHOLE_ENABLE_AUDIO_DEBUG
                static int segmentLogCount = 0;
                if (segmentLogCount++ < 5) {
                    std::cout << "[GraphEngine] injectStreamData: stream '" << binding.streamId
                              << "' has " << segments.size() << " active segment(s)" << std::endl;
                }
                #endif

                for (const auto* segment : segments) {
                    if (segment && segment->startSamples < blockEndSamples && segment->endSamples > blockStartSamples) {
                        // Phase 3: Pass AudioAssetSource to injectAudioSegment
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

void GraphEngine::routeConnections() {
    // This method is no longer used - routing happens inline in processGraph()
    // Kept for potential future use or debugging
}

