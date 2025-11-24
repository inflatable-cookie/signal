#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/StreamScheduler.hpp"
#include "core/ScheduleData.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>
#include <cmath>

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

            // Apply schedule to StreamScheduler
            _engineHost->streamScheduler().setSchedule(
                streams,
                audioSegments,
                midiEvents,
                tempoMap,
                sampleRate
            );

            std::cout << "[EngineDomain] Applied stream-based schedule: " << streams.size() << " streams, "
                      << audioSegments.size() << " audio segments, "
                      << midiEvents.size() << " MIDI events" << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[EngineDomain] Failed to parse schedule payload: " << e.what() << std::endl;
        }
    } else if (env.name == "graphSnapshot" || env.name == "applyGraphSnapshot") {
        // TODO: Handle GraphSnapshot from Pulse
        // Architecture: Pulse sends GraphSnapshot with nodes and connections
        // Signal builds runtime node graph from snapshot
        // This includes:
        // - Lane nodes (audio-lane, midi-lane) - one per stream
        // - Processing nodes (audio-fx, midi-fx, instrument)
        // - Connections: stream → node, node → node, node → output
        // - Cohort assignments for scheduling
        //
        // Current implementation: Stub - will be implemented in future prompt
        std::cout << "[EngineDomain] GraphSnapshot command received (not yet implemented)" << std::endl;
    } else {
        std::cout << "[EngineDomain] Unknown command: " << env.name << std::endl;
    }
}

