#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/StreamScheduler.hpp"
#include "core/ScheduleData.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <cmath>
#include <unordered_set>
#include <sstream>
#include <iostream>

EngineDomain::EngineDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void EngineDomain::handleStart() {
    _engineHost->start();
}

void EngineDomain::handleStop() {
    _engineHost->stop();
}

void EngineDomain::handleReset() {
    _engineHost->reset();
}

void EngineDomain::handleShutdown() {
    LOG_INFO({"EngineDomain"}, "Shutdown requested");
    _engineHost->shutdown();
}

void EngineDomain::handleScheduleSession(const nlohmann::json& payload) {
    // Handle stream-based schedule from Pulse
    // Architecture: Pulse sends PlaybackScheduleSnapshot with streams, audioSegments, midiEvents
    // Signal converts to compiled format and applies to StreamScheduler
    try {

            // Diagnostic: Log raw JSON structure
            std::cout << "[EngineDomain][Schedule][Signal] Received playback schedule snapshot envelope" << std::endl;
            std::cout << "[EngineDomain][Schedule][Signal] Top-level keys: ";
            if (payload.is_object()) {
                for (auto it = payload.begin(); it != payload.end(); ++it) {
                    std::cout << it.key() << " ";
                }
            }
            std::cout << std::endl;
            if (payload.contains("streams")) {
                std::cout << "[EngineDomain][Schedule][Signal] 'streams' type: " << payload["streams"].type_name()
                          << ", is_array: " << payload["streams"].is_array() << std::endl;
                if (payload["streams"].is_array()) {
                    std::cout << "[EngineDomain][Schedule][Signal] 'streams' array size: " << payload["streams"].size() << std::endl;
                    if (payload["streams"].size() > 0) {
                        const auto& firstStream = payload["streams"][0];
                        std::cout << "[EngineDomain][Schedule][Signal] First stream JSON keys: ";
                        if (firstStream.is_object()) {
                            for (auto it = firstStream.begin(); it != firstStream.end(); ++it) {
                                std::cout << it.key() << " ";
                            }
                        }
                        std::cout << std::endl;
                    }
                } else {
                    std::cerr << "[EngineDomain][Schedule][Signal] ERROR: 'streams' field exists but is not an array!" << std::endl;
                }
            } else {
                std::cerr << "[EngineDomain][Schedule][Signal] ERROR: 'streams' field not found in payload!" << std::endl;
            }
            if (payload.contains("audioSegments")) {
                std::cout << "[EngineDomain][Schedule][Signal] 'audioSegments' type: " << payload["audioSegments"].type_name()
                          << ", is_array: " << payload["audioSegments"].is_array() << std::endl;
                if (payload["audioSegments"].is_array()) {
                    std::cout << "[EngineDomain][Schedule][Signal] 'audioSegments' array size: " << payload["audioSegments"].size() << std::endl;
                    if (payload["audioSegments"].size() > 0) {
                        const auto& firstSegment = payload["audioSegments"][0];
                        std::cout << "[EngineDomain][Schedule][Signal] First segment JSON keys: ";
                        if (firstSegment.is_object()) {
                            for (auto it = firstSegment.begin(); it != firstSegment.end(); ++it) {
                                std::cout << it.key() << " ";
                            }
                        }
                        std::cout << std::endl;
                    }
                } else {
                    std::cerr << "[EngineDomain][Schedule][Signal] ERROR: 'audioSegments' field exists but is not an array!" << std::endl;
                }
            } else {
                std::cerr << "[EngineDomain][Schedule][Signal] ERROR: 'audioSegments' field not found in payload!" << std::endl;
            }

            double sampleRate = _engineHost->getSampleRate();

            // Parse streams
            std::vector<StreamDescriptor> streams;
            if (payload.contains("streams") && payload["streams"].is_array()) {
                for (size_t idx = 0; idx < payload["streams"].size(); ++idx) {
                    const auto& streamJson = payload["streams"][idx];
                    StreamDescriptor stream;
                    stream.streamId = streamJson.value("streamId", "");
                    stream.trackId = streamJson.value("trackId", "");
                    stream.laneId = streamJson.value("laneId", "");
                    std::string streamTypeStr = streamJson.value("streamType", "");
                    // Map string to StreamType enum
                    if (streamTypeStr == "audio" || streamTypeStr == "Audio") {
                        stream.streamType = "Audio";
                    } else if (streamTypeStr == "midi" || streamTypeStr == "Midi") {
                        stream.streamType = "Midi";
                    } else {
                        stream.streamType = streamTypeStr; // Keep as-is if unknown
                    }
                    streams.push_back(stream);
                    if (idx < 3) { // Log first 3 streams
                        std::cout << "[EngineDomain][Schedule][Signal] Parsed Stream " << idx
                                  << ": streamId='" << stream.streamId
                                  << "', trackId='" << stream.trackId
                                  << "', laneId='" << stream.laneId
                                  << "', type=" << stream.streamType << std::endl;
                    }
                }
            }

            // Parse audio segments (beats-based, convert to samples)
            std::vector<AudioSegmentCompiled> audioSegments;
            TempoMap tempoMap;

            // Parse tempo map (simplified - full implementation in future)
            if (payload.contains("tempoMap")) {
                const auto& tempoMapJson = payload["tempoMap"];
                tempoMap.defaultTempo = tempoMapJson.value("defaultTempo", 120.0);
                if (tempoMapJson.contains("entries") && tempoMapJson["entries"].is_array()) {
                    for (const auto& entryJson : tempoMapJson["entries"]) {
                        TempoMapEntry entry;
                        entry.timeBeats = entryJson.value("timeBeats", 0.0);
                        entry.tempo = entryJson.value("tempo", 120.0);
                        tempoMap.entries.push_back(entry);
                    }
                }
            } else {
                // Fallback to transport tempo
                tempoMap.defaultTempo = _engineHost->transport().tempo;
            }

            // Convert beats to samples helper (simplified - uses default tempo)
            auto beatsToSamples = [&](double beats) -> uint64_t {
                return static_cast<uint64_t>((beats / tempoMap.defaultTempo) * 60.0 * sampleRate);
            };

            if (payload.contains("audioSegments") && payload["audioSegments"].is_array()) {
                for (size_t idx = 0; idx < payload["audioSegments"].size(); ++idx) {
                    const auto& segmentJson = payload["audioSegments"][idx];
                    AudioSegmentCompiled segment;
                    segment.streamId = segmentJson.value("streamId", "");
                    segment.assetId = segmentJson.value("assetId", "");

                    // Convert beats to samples
                    double startBeats = segmentJson.value("startBeats", 0.0);
                    double endBeats = segmentJson.value("endBeats", 0.0);
                    double assetStartBeats = segmentJson.value("assetStartBeats", 0.0);

                    segment.startSamples = beatsToSamples(startBeats);
                    segment.endSamples = beatsToSamples(endBeats);
                    segment.assetStartSamples = beatsToSamples(assetStartBeats);

                    // Phase 12b: Parse gain (if present)
                    segment.gainDb = segmentJson.value("gainDb", 0.0);

                    // Phase 12b: Parse fade metadata (if present)
                    if (segmentJson.contains("fadeInBeats")) {
                        double fadeInBeats = segmentJson.value("fadeInBeats", 0.0);
                        segment.fadeInSamples = beatsToSamples(fadeInBeats);
                        segment.fadeInCurve = segmentJson.value("fadeInCurve", "linear");
                    } else {
                        segment.fadeInSamples = 0;
                        segment.fadeInCurve = "linear";
                    }

                    if (segmentJson.contains("fadeOutBeats")) {
                        double fadeOutBeats = segmentJson.value("fadeOutBeats", 0.0);
                        segment.fadeOutSamples = beatsToSamples(fadeOutBeats);
                        segment.fadeOutCurve = segmentJson.value("fadeOutCurve", "linear");
                    } else {
                        segment.fadeOutSamples = 0;
                        segment.fadeOutCurve = "linear";
                    }

                    // Phase 9: Parse stretch metadata (if present)
                    if (segmentJson.contains("stretch") && segmentJson["stretch"].is_object()) {
                        const auto& stretchJson = segmentJson["stretch"];
                        segment.stretch.mode = stretchJson.value("mode", "none");
                        segment.stretch.ratio = stretchJson.value("ratio", 1.0);
                    } else {
                        segment.stretch.mode = "none";
                        segment.stretch.ratio = 1.0;
                    }

                    audioSegments.push_back(segment);
                    if (idx < 3) { // Log first 3 segments
                        std::cout << "[EngineDomain][Schedule][Signal] Parsed Segment " << idx
                                  << ": streamId='" << segment.streamId
                                  << "', assetId='" << segment.assetId
                                  << "', startBeats=" << startBeats
                                  << ", endBeats=" << endBeats
                                  << ", assetStartBeats=" << assetStartBeats << std::endl;
                    }
                }
            }

            // Parse MIDI events (beats-based, convert to samples)
            std::vector<MidiEventCompiled> midiEvents;
            if (payload.contains("midiEvents") && payload["midiEvents"].is_array()) {
                for (const auto& eventJson : payload["midiEvents"]) {
                    MidiEventCompiled event;
                    event.streamId = eventJson.value("streamId", "");

                    // Convert beats to samples
                    double timeBeats = eventJson.value("timeBeats", 0.0);
                    event.timeSamples = beatsToSamples(timeBeats);

                    event.status = eventJson.value("status", 0);
                    event.data1 = eventJson.value("data1", 0);
                    event.data2 = eventJson.value("data2", 0);
                    event.channel = eventJson.value("channel", 0);

                    midiEvents.push_back(event);
                }
            }

            // Diagnostic logging: schedule reception
            std::cout << "[EngineDomain][Schedule] Received playback schedule snapshot" << std::endl;
            std::cout << "[EngineDomain][Schedule] Parsed " << streams.size() << " streams, "
                      << audioSegments.size() << " audio segments, "
                      << midiEvents.size() << " MIDI events" << std::endl;

            // Log stream details
            for (const auto& stream : streams) {
                std::cout << "[EngineDomain][Schedule] Stream: id='" << stream.streamId
                          << "', track='" << stream.trackId
                          << "', lane='" << stream.laneId
                          << "', type='" << stream.streamType << "'" << std::endl;
            }

            // Log first few segments
            for (size_t idx = 0; idx < audioSegments.size() && idx < 5; ++idx) {
                const auto& seg = audioSegments[idx];
                std::cout << "[EngineDomain][Schedule] Segment " << idx << ": stream='" << seg.streamId
                          << "', asset='" << seg.assetId
                          << "', start=" << seg.startSamples << " samples, end=" << seg.endSamples << " samples"
                          << ", assetStart=" << seg.assetStartSamples << " samples" << std::endl;
            }
            if (audioSegments.size() > 5) {
                std::cout << "[EngineDomain][Schedule] ... and " << (audioSegments.size() - 5) << " more segments" << std::endl;
            }

            if (audioSegments.empty()) {
                std::cerr << "[EngineDomain][Schedule] ✗ WARNING: No audio segments in schedule!" << std::endl;
            }

            // Apply schedule to StreamScheduler
            _engineHost->streamScheduler().setSchedule(
                streams,
                audioSegments,
                midiEvents,
                tempoMap,
                sampleRate
            );

        std::cout << "[EngineDomain][Schedule] Schedule applied to StreamScheduler" << std::endl;
    } catch (const std::exception& e) {
        LOG_ERROR({"EngineDomain"}, std::string("Failed to parse schedule payload: ") + e.what());
    }
}

void EngineDomain::handleGraphSnapshot(const nlohmann::json& payload) {
    // Handle GraphSnapshot from Pulse
    // Architecture: Pulse sends GraphSnapshot with nodes and connections
    // Signal builds runtime node graph from snapshot
    try {

            // Diagnostic: Log raw JSON structure (truncated)
            std::cout << "[EngineDomain][Graph] Received graph snapshot envelope" << std::endl;
            if (payload.contains("id")) {
                std::cout << "[EngineDomain][Graph] Top-level keys: id, nodes, connections" << std::endl;
            }
            if (payload.contains("nodes") && payload["nodes"].is_array() && !payload["nodes"].empty()) {
                const auto& firstNode = payload["nodes"][0];
                std::cout << "[EngineDomain][Graph] First node keys: ";
                if (firstNode.is_object()) {
                    for (auto it = firstNode.begin(); it != firstNode.end(); ++it) {
                        std::cout << it.key() << " ";
                    }
                }
                std::cout << std::endl;
            }

            // Parse graph snapshot ID
            std::string snapshotId = "unknown";
            if (payload.contains("id") && payload["id"].is_string()) {
                snapshotId = payload["id"].get<std::string>();
            } else if (payload.contains("id")) {
                std::cerr << "[EngineDomain][Graph] WARNING: 'id' field is not a string, type: " << payload["id"].type_name() << std::endl;
            }

            // Parse nodes
            std::vector<NodeDesc> nodes;
            if (payload.contains("nodes") && payload["nodes"].is_array()) {
                for (const auto& nodeJson : payload["nodes"]) {
                    NodeDesc node;
                    node.nodeId = nodeJson.value("nodeId", "");
                    if (node.nodeId.empty()) {
                        std::cerr << "[EngineDomain][Graph][Signal] Node missing nodeId" << std::endl;
                        continue;
                    }

                    // Parse optional track/lane IDs
                    if (nodeJson.contains("trackId") && nodeJson["trackId"].is_string()) {
                        node.trackId = nodeJson["trackId"].get<std::string>();
                    }
                    if (nodeJson.contains("laneId") && nodeJson["laneId"].is_string()) {
                        node.laneId = nodeJson["laneId"].get<std::string>();
                    }

                    // Parse node kind with diagnostic logging
                    std::string kindStr = "";
                    if (nodeJson.contains("kind")) {
                        if (nodeJson["kind"].is_string()) {
                            kindStr = nodeJson["kind"].get<std::string>();
                        } else {
                            std::cerr << "[EngineDomain][Graph][Signal] Node " << node.nodeId
                                      << " has 'kind' field but it's not a string, type: " << nodeJson["kind"].type_name() << std::endl;
                        }
                    }
                    auto kindOpt = nodeKindFromString(kindStr);
                    if (!kindOpt.has_value()) {
                        std::cerr << "[EngineDomain][Graph][Signal] Node " << node.nodeId
                                  << " has invalid kind: \"" << kindStr << "\" (raw JSON value)" << std::endl;
                        continue;
                    }
                    node.kind = kindOpt.value();
                    // Log node kind mapping with readable name
                    std::string kindName = "Unknown";
                    switch (node.kind) {
                        case NodeKind::MidiLane: kindName = "MidiLane"; break;
                        case NodeKind::AudioLane: kindName = "AudioLane"; break;
                        case NodeKind::MidiFx: kindName = "MidiFx"; break;
                        case NodeKind::Instrument: kindName = "Instrument"; break;
                        case NodeKind::AudioFx: kindName = "AudioFx"; break;
                        case NodeKind::Send: kindName = "Send"; break;
                        case NodeKind::MixerChannel: kindName = "MixerChannel"; break;
                        case NodeKind::Receive: kindName = "Receive"; break;
                        case NodeKind::Device: kindName = "Device"; break;
                        case NodeKind::AudioInput: kindName = "AudioInput"; break;
                        case NodeKind::MidiInput: kindName = "MidiInput"; break;
                        default: kindName = "Unknown"; break;
                    }
                    std::cout << "[EngineDomain][Graph][Signal] Raw node: nodeId='" << node.nodeId
                              << "', kind=\"" << kindStr << "\" → NodeKind::" << kindName << std::endl;

                    // Parse plugin metadata (if present)
                    if (nodeJson.contains("pluginFormat") && nodeJson["pluginFormat"].is_string()) {
                        std::string formatStr = nodeJson["pluginFormat"].get<std::string>();
                        if (formatStr == "clap") node.pluginFormat = PluginFormat::Clap;
                        else if (formatStr == "vst3") node.pluginFormat = PluginFormat::Vst3;
                        else if (formatStr == "au") node.pluginFormat = PluginFormat::Au;
                        else if (formatStr == "lv2") node.pluginFormat = PluginFormat::Lv2;
                        else if (formatStr == "native") node.pluginFormat = PluginFormat::Native;
                    }
                    if (nodeJson.contains("pluginId") && nodeJson["pluginId"].is_string()) {
                        node.pluginId = nodeJson["pluginId"].get<std::string>();
                    }

                    // Parse audio/MIDI channel counts
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

                    // Parse input node fields (Phase 7)
                    if (nodeJson.contains("deviceId") && nodeJson["deviceId"].is_string()) {
                        node.deviceId = nodeJson["deviceId"].get<std::string>();
                    }
                    if (nodeJson.contains("inputChannelIndex") && nodeJson["inputChannelIndex"].is_number_integer()) {
                        node.inputChannelIndex = nodeJson["inputChannelIndex"].get<int>();
                    }
                    if (nodeJson.contains("portId") && nodeJson["portId"].is_string()) {
                        node.portId = nodeJson["portId"].get<std::string>();
                    }

                    nodes.push_back(node);
                }
            }

            // Parse connections
            std::vector<ConnectionDesc> connections;
            if (payload.contains("connections") && payload["connections"].is_array()) {
                for (const auto& connJson : payload["connections"]) {
                    ConnectionDesc conn;

                    // Parse source (either fromStreamId or fromNodeId)
                    if (connJson.contains("fromStreamId") && connJson["fromStreamId"].is_string()) {
                        conn.fromStreamId = connJson["fromStreamId"].get<std::string>();
                    } else if (connJson.contains("fromNodeId") && connJson["fromNodeId"].is_string()) {
                        conn.fromNodeId = connJson["fromNodeId"].get<std::string>();
                    }

                    // Parse output/input indices (default to 0)
                    // Use safe parsing that checks type first
                    if (connJson.contains("fromOutputIndex")) {
                        if (connJson["fromOutputIndex"].is_number_unsigned()) {
                            conn.fromOutputIndex = connJson["fromOutputIndex"].get<uint32_t>();
                        } else {
                            std::cerr << "[EngineDomain][Graph] WARNING: fromOutputIndex is not a number, type: " << connJson["fromOutputIndex"].type_name() << std::endl;
                            conn.fromOutputIndex = 0;
                        }
                    } else {
                        conn.fromOutputIndex = 0;
                    }
                    if (connJson.contains("toInputIndex")) {
                        if (connJson["toInputIndex"].is_number_unsigned()) {
                            conn.toInputIndex = connJson["toInputIndex"].get<uint32_t>();
                        } else {
                            std::cerr << "[EngineDomain][Graph] WARNING: toInputIndex is not a number, type: " << connJson["toInputIndex"].type_name() << std::endl;
                            conn.toInputIndex = 0;
                        }
                    } else {
                        conn.toInputIndex = 0;
                    }

                    // Parse destination (required)
                    if (connJson.contains("toNodeId") && connJson["toNodeId"].is_string()) {
                        conn.toNodeId = connJson["toNodeId"].get<std::string>();
                    } else {
                        std::cerr << "[EngineDomain] GraphSnapshot connection missing toNodeId" << std::endl;
                        continue;
                    }

                    connections.push_back(conn);
                }
            }

            // Build GraphSnapshot structure
            GraphSnapshot snapshot;
            snapshot.id = snapshotId;
            snapshot.nodes = nodes;
            snapshot.connections = connections;

            // Validate snapshot
            bool isValid = true;
            std::string validationError;

            // Check: All referenced node IDs in connections must exist
            std::unordered_set<std::string> nodeIds;
            for (const auto& node : nodes) {
                nodeIds.insert(node.nodeId);
            }

            for (const auto& conn : connections) {
                if (conn.fromNodeId.has_value() && nodeIds.find(conn.fromNodeId.value()) == nodeIds.end()) {
                    validationError = "Connection references non-existent fromNodeId: " + conn.fromNodeId.value();
                    isValid = false;
                    break;
                }
                if (nodeIds.find(conn.toNodeId) == nodeIds.end()) {
                    validationError = "Connection references non-existent toNodeId: " + conn.toNodeId;
                    isValid = false;
                    break;
                }
            }

            // Check: At least one DeviceNode must exist as a sink
            bool hasDeviceNode = false;
            int deviceNodeCount = 0;
            int masterNodeCount = 0;
            for (const auto& node : nodes) {
                if (node.kind == NodeKind::Device) {
                    hasDeviceNode = true;
                    deviceNodeCount++;
                }
                // Note: Master is deprecated but check for it anyway
                // (though nodeKindFromString maps "master" to Device, so this shouldn't be needed)
            }
            std::cout << "[EngineDomain][Graph][Signal] Before validation: " << deviceNodeCount
                      << " Device nodes, " << masterNodeCount << " Master nodes" << std::endl;
            if (!hasDeviceNode) {
                validationError = "GraphSnapshot must contain at least one DeviceNode";
                isValid = false;
            }

            if (!isValid) {
                std::cerr << "[EngineDomain] GraphSnapshot validation failed: " << validationError << std::endl;
                // Don't replace current graph - keep previous or silence
                return;
            }

            // Diagnostic logging: graph snapshot reception
            std::cout << "[EngineDomain][Graph] Received graph snapshot: id='" << snapshotId << "'" << std::endl;
            std::cout << "[EngineDomain][Graph] Parsed " << nodes.size() << " nodes, " << connections.size() << " connections" << std::endl;

            // Count AudioLane nodes and check for fromStreamId connections (reuse hasDeviceNode from validation above)
            std::vector<std::string> laneNodeIds;
            for (const auto& node : nodes) {
                if (node.kind == NodeKind::AudioLane) {
                    laneNodeIds.push_back(node.nodeId);
                }
            }
            if (hasDeviceNode) {
                std::cout << "[EngineDomain][Graph] ✓ DeviceNode found" << std::endl;
            } else {
                std::cerr << "[EngineDomain][Graph] ✗ WARNING: No DeviceNode found in graph!" << std::endl;
            }
            if (!laneNodeIds.empty()) {
                std::cout << "[EngineDomain][Graph] AudioLane nodes: ";
                for (size_t i = 0; i < laneNodeIds.size(); ++i) {
                    if (i > 0) std::cout << ", ";
                    std::cout << laneNodeIds[i];
                }
                std::cout << std::endl;
            }

            // Check for fromStreamId connections
            int streamBindingCount = 0;
            for (const auto& conn : connections) {
                if (conn.fromStreamId.has_value()) {
                    streamBindingCount++;
                    std::cout << "[EngineDomain][Graph] Stream binding: '" << conn.fromStreamId.value()
                              << "' -> node '" << conn.toNodeId << "'" << std::endl;
                }
            }
            if (streamBindingCount == 0) {
                std::cerr << "[EngineDomain][Graph] ✗ WARNING: No fromStreamId connections found!" << std::endl;
            } else {
                std::cout << "[EngineDomain][Graph] ✓ Found " << streamBindingCount << " stream binding(s)" << std::endl;
            }

            // Load snapshot into EngineHost
            _engineHost->loadGraphSnapshot(snapshot);

            // Prepare graph if engine is already running
            if (_engineHost->state() == EngineHost::State::Running) {
                _engineHost->prepareEngine(
                    static_cast<int>(_engineHost->getSampleRate()),
                    static_cast<size_t>(_engineHost->getBlockSize())
                );
            }

        std::cout << "[EngineDomain][Graph] Graph snapshot loaded and prepared" << std::endl;
    } catch (const std::exception& e) {
        LOG_ERROR({"EngineDomain"}, std::string("Failed to parse graph snapshot payload: ") + e.what());
        // Don't replace current graph - keep previous or silence
    }
}

void EngineDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "engine") {
        LOG_DEBUG({"EngineDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"EngineDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"EngineDomain"}, "EngineHost is null");
        return;
    }

    // Handle commands directly
    if (env.name == "start") {
        handleStart();
    } else if (env.name == "stop") {
        handleStop();
    } else if (env.name == "reset") {
        handleReset();
    } else if (env.name == "shutdown") {
        handleShutdown();
    } else if (env.name == "heartbeat") {
        // Heartbeat command - just emit response
        emitHeartbeatEvent(env, session);
        return;
    } else if (env.name == "scheduleSession" || env.name == "playbackScheduleSnapshot") {
        handleScheduleSession(env.payload);
    } else if (env.name == "graphSnapshot" || env.name == "applyGraphSnapshot") {
        handleGraphSnapshot(env.payload);
    } else {
        LOG_WARN({"EngineDomain"}, std::string("Unknown command: ") + env.name);
    }

    // Emit state events after processing commands (except heartbeat which already emitted)
    if (env.name != "heartbeat") {
        emitEngineStateEvent(env, session);
    }
}

void EngineDomain::emitEngineStateEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    IpcEnvelope stateEvent;
    stateEvent.version = 1;
    stateEvent.id = "engine-state-" + commandEnv.id;
    stateEvent.correlationId = commandEnv.id;
    stateEvent.timestamp = currentTimestamp();
    stateEvent.origin = IpcOrigin::Signal;

    // Convert origin to target for event
    switch (commandEnv.origin) {
    case IpcOrigin::Aura:
        stateEvent.target = IpcTarget::Aura;
        break;
    case IpcOrigin::Pulse:
        stateEvent.target = IpcTarget::Pulse;
        break;
    case IpcOrigin::Signal:
        stateEvent.target = IpcTarget::Signal;
        break;
    case IpcOrigin::Composer:
        stateEvent.target = IpcTarget::Composer;
        break;
    }

    stateEvent.domain = "engine";
    stateEvent.kind = IpcKind::Event;
    stateEvent.name = "state";
    stateEvent.priority = commandEnv.priority;

    // Get current engine state and create payload
    std::string lifecycle = "stopped";
    std::optional<std::string> lastError;
    if (_engineHost) {
        switch (_engineHost->state()) {
        case EngineHost::State::Stopped:
            lifecycle = "stopped";
            break;
        case EngineHost::State::Starting:
            lifecycle = "starting";
            break;
        case EngineHost::State::Running:
            lifecycle = "running";
            break;
        case EngineHost::State::Error:
            lifecycle = "error";
            lastError = _engineHost->lastError();
            break;
        }
    }

    nlohmann::json payload;
    payload["lifecycle"] = lifecycle;
    if (lastError.has_value()) {
        payload["lastError"] = lastError.value();
    } else {
        payload["lastError"] = nullptr;
    }
    // Include runtime configuration in state event
    if (_engineHost) {
        payload["sampleRate"] = _engineHost->getSampleRate();
        payload["blockSize"] = _engineHost->getBlockSize();
        payload["outputDeviceName"] = _engineHost->getOutputDeviceName();
        payload["numOutputChannels"] = _engineHost->getNumOutputChannels();
    }

    stateEvent.payload = payload;

    session->send(stateEvent);
}

void EngineDomain::emitHeartbeatEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    IpcEnvelope heartbeatEvent;
    heartbeatEvent.version = 1;
    heartbeatEvent.id = "engine-heartbeat-" + commandEnv.id;
    heartbeatEvent.correlationId = commandEnv.id;
    heartbeatEvent.timestamp = currentTimestamp();
    heartbeatEvent.origin = IpcOrigin::Signal;

    switch (commandEnv.origin) {
    case IpcOrigin::Aura:
        heartbeatEvent.target = IpcTarget::Aura;
        break;
    case IpcOrigin::Pulse:
        heartbeatEvent.target = IpcTarget::Pulse;
        break;
    case IpcOrigin::Signal:
        heartbeatEvent.target = IpcTarget::Signal;
        break;
    case IpcOrigin::Composer:
        heartbeatEvent.target = IpcTarget::Composer;
        break;
    }

    heartbeatEvent.domain = "engine";
    heartbeatEvent.kind = IpcKind::Event;
    heartbeatEvent.name = "heartbeat";
    heartbeatEvent.priority = commandEnv.priority;

    std::string lifecycle = "stopped";
    if (_engineHost) {
        switch (_engineHost->state()) {
        case EngineHost::State::Stopped:
            lifecycle = "stopped";
            break;
        case EngineHost::State::Starting:
            lifecycle = "starting";
            break;
        case EngineHost::State::Running:
            lifecycle = "running";
            break;
        case EngineHost::State::Error:
            lifecycle = "error";
            break;
        }
    }

    nlohmann::json payload;
    payload["lifecycle"] = lifecycle;
    heartbeatEvent.payload = payload;

    session->send(heartbeatEvent);
}

