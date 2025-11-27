#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/StreamScheduler.hpp"
#include "core/ScheduleData.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>
#include <cmath>
#include <unordered_set>

EngineDomain::EngineDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void EngineDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        std::cout << "[EngineDomain] Ignoring non-command: " << env.kind << std::endl;
        return;
    }

    if (!_engineHost) {
        std::cerr << "[EngineDomain] EngineHost is null" << std::endl;
        return;
    }

    if (env.name == "start") {
        _engineHost->start();
    } else if (env.name == "stop") {
        _engineHost->stop();
    } else if (env.name == "reset") {
        _engineHost->reset();
    } else if (env.name == "shutdown") {
        std::cout << "[EngineDomain] Shutdown requested" << std::endl;
        _engineHost->shutdown();
    } else if (env.name == "heartbeat") {
        // Heartbeat command received - handled by DomainDispatcher to emit event
        std::cout << "[EngineDomain] Heartbeat command received" << std::endl;
    } else if (env.name == "scheduleSession" || env.name == "playbackScheduleSnapshot") {
        // Handle stream-based schedule from Pulse
        // Architecture: Pulse sends PlaybackScheduleSnapshot with streams, audioSegments, midiEvents
        // Signal converts to compiled format and applies to StreamScheduler
        try {
            nlohmann::json payload = env.payload;
            double sampleRate = _engineHost->getSampleRate();

            // Parse streams
            std::vector<StreamDescriptor> streams;
            if (payload.contains("streams") && payload["streams"].is_array()) {
                for (const auto& streamJson : payload["streams"]) {
                    StreamDescriptor stream;
                    stream.streamId = streamJson.value("streamId", "");
                    stream.trackId = streamJson.value("trackId", "");
                    stream.laneId = streamJson.value("laneId", "");
                    stream.streamType = streamJson.value("streamType", "");
                    streams.push_back(stream);
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
                for (const auto& segmentJson : payload["audioSegments"]) {
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
            std::cerr << "[EngineDomain] Failed to parse schedule payload: " << e.what() << std::endl;
        }
    } else if (env.name == "graphSnapshot" || env.name == "applyGraphSnapshot") {
        // Handle GraphSnapshot from Pulse
        // Architecture: Pulse sends GraphSnapshot with nodes and connections
        // Signal builds runtime node graph from snapshot
        try {
            nlohmann::json payload = env.payload;

            // Parse graph snapshot ID
            std::string snapshotId = payload.value("id", "unknown");

            // Parse nodes
            std::vector<NodeDesc> nodes;
            if (payload.contains("nodes") && payload["nodes"].is_array()) {
                for (const auto& nodeJson : payload["nodes"]) {
                    NodeDesc node;
                    node.nodeId = nodeJson.value("nodeId", "");
                    if (node.nodeId.empty()) {
                        std::cerr << "[EngineDomain] GraphSnapshot node missing nodeId" << std::endl;
                        continue;
                    }

                    // Parse optional track/lane IDs
                    if (nodeJson.contains("trackId") && nodeJson["trackId"].is_string()) {
                        node.trackId = nodeJson["trackId"].get<std::string>();
                    }
                    if (nodeJson.contains("laneId") && nodeJson["laneId"].is_string()) {
                        node.laneId = nodeJson["laneId"].get<std::string>();
                    }

                    // Parse node kind
                    std::string kindStr = nodeJson.value("kind", "");
                    auto kindOpt = nodeKindFromString(kindStr);
                    if (!kindOpt.has_value()) {
                        std::cerr << "[EngineDomain] GraphSnapshot node " << node.nodeId
                                  << " has invalid kind: " << kindStr << std::endl;
                        continue;
                    }
                    node.kind = kindOpt.value();

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
                    conn.fromOutputIndex = connJson.value("fromOutputIndex", 0u);
                    conn.toInputIndex = connJson.value("toInputIndex", 0u);

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
            for (const auto& node : nodes) {
                if (node.kind == NodeKind::Device) {
                    hasDeviceNode = true;
                    break;
                }
            }
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
            std::cerr << "[EngineDomain] Failed to parse graph snapshot payload: " << e.what() << std::endl;
            // Don't replace current graph - keep previous or silence
        }
    } else {
        std::cout << "[EngineDomain] Unknown command: " << env.name << std::endl;
    }
}

