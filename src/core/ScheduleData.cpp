#include "core/ScheduleData.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <algorithm>
#include <sstream>

void ScheduleData::buildLookupMaps() {
    // Clear existing maps
    audioSegmentsByStream.clear();
    midiEventsByStream.clear();

    // Build audio segments lookup by streamId
    for (const auto& segment : audioSegments) {
        audioSegmentsByStream[segment.streamId].push_back(&segment);
    }

    // Sort segments by startSamples for each stream (for efficient range queries)
    for (auto& pair : audioSegmentsByStream) {
        std::sort(pair.second.begin(), pair.second.end(),
            [](const AudioSegmentCompiled* a, const AudioSegmentCompiled* b) {
                return a->startSamples < b->startSamples;
            });
    }

    // Build MIDI events lookup by streamId
    for (const auto& event : midiEvents) {
        midiEventsByStream[event.streamId].push_back(&event);
    }

    // Sort events by timeSamples for each stream (for efficient range queries)
    for (auto& pair : midiEventsByStream) {
        std::sort(pair.second.begin(), pair.second.end(),
            [](const MidiEventCompiled* a, const MidiEventCompiled* b) {
                return a->timeSamples < b->timeSamples;
            });
    }
}

std::optional<ScheduleData> ScheduleData::fromJson(
    const nlohmann::json& j,
    double sampleRate,
    double defaultTempo
) {
    if (!j.is_object()) {
        LOG_ERROR({"ScheduleData"}, "JSON payload is not an object");
        return std::nullopt;
    }

    ScheduleData schedule(sampleRate, defaultTempo);

    // Parse tempo map
    if (j.contains("tempoMap") && j["tempoMap"].is_object()) {
        const auto& tempoMapJson = j["tempoMap"];
        schedule.tempoMap.defaultTempo = tempoMapJson.value("defaultTempo", defaultTempo);
        if (tempoMapJson.contains("entries") && tempoMapJson["entries"].is_array()) {
            for (const auto& entryJson : tempoMapJson["entries"]) {
                TempoMapEntry entry;
                entry.timeBeats = entryJson.value("timeBeats", 0.0);
                entry.tempo = entryJson.value("tempo", defaultTempo);
                schedule.tempoMap.entries.push_back(entry);
            }
        }
    } else {
        schedule.tempoMap.defaultTempo = defaultTempo;
    }

    // Helper to convert beats to samples (simplified - uses default tempo)
    auto beatsToSamples = [&](double beats) -> uint64_t {
        return static_cast<uint64_t>((beats / schedule.tempoMap.defaultTempo) * 60.0 * sampleRate);
    };

    // Parse streams
    if (j.contains("streams") && j["streams"].is_array()) {
        for (const auto& streamJson : j["streams"]) {
            if (!streamJson.is_object()) {
                LOG_WARN({"ScheduleData"}, "Skipping invalid stream entry (not an object)");
                continue;
            }

            StreamDescriptor stream;
            stream.streamId = streamJson.value("streamId", "");
            stream.trackId = streamJson.value("trackId", "");
            stream.laneId = streamJson.value("laneId", "");

            std::string streamTypeStr = streamJson.value("streamType", "");
            // Map string to stream type (normalize case)
            if (streamTypeStr == "audio" || streamTypeStr == "Audio") {
                stream.streamType = "Audio";
            } else if (streamTypeStr == "midi" || streamTypeStr == "Midi") {
                stream.streamType = "Midi";
            } else {
                stream.streamType = streamTypeStr; // Keep as-is if unknown
            }

            if (stream.streamId.empty()) {
                LOG_WARN({"ScheduleData"}, "Skipping stream with empty streamId");
                continue;
            }

            schedule.streams.push_back(stream);
        }
    } else {
        LOG_WARN({"ScheduleData"}, "Missing or invalid 'streams' array in schedule payload");
    }

    // Parse audio segments
    if (j.contains("audioSegments") && j["audioSegments"].is_array()) {
        for (const auto& segmentJson : j["audioSegments"]) {
            if (!segmentJson.is_object()) {
                LOG_WARN({"ScheduleData"}, "Skipping invalid audio segment (not an object)");
                continue;
            }

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

            // Parse gain (Phase 12b)
            segment.gainDb = segmentJson.value("gainDb", 0.0);

            // Parse fade metadata (Phase 12b)
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

            // Parse stretch metadata (Phase 9)
            if (segmentJson.contains("stretch") && segmentJson["stretch"].is_object()) {
                const auto& stretchJson = segmentJson["stretch"];
                segment.stretch.mode = stretchJson.value("mode", "none");
                segment.stretch.ratio = stretchJson.value("ratio", 1.0);
            } else {
                segment.stretch.mode = "none";
                segment.stretch.ratio = 1.0;
            }

            if (segment.streamId.empty()) {
                LOG_WARN({"ScheduleData"}, "Skipping audio segment with empty streamId");
                continue;
            }

            schedule.audioSegments.push_back(segment);
        }
    }

    // Parse MIDI events
    if (j.contains("midiEvents") && j["midiEvents"].is_array()) {
        for (const auto& eventJson : j["midiEvents"]) {
            if (!eventJson.is_object()) {
                LOG_WARN({"ScheduleData"}, "Skipping invalid MIDI event (not an object)");
                continue;
            }

            MidiEventCompiled event;
            event.streamId = eventJson.value("streamId", "");

            // Convert beats to samples
            double timeBeats = eventJson.value("timeBeats", 0.0);
            event.timeSamples = beatsToSamples(timeBeats);

            event.status = static_cast<uint8_t>(eventJson.value("status", 0));
            event.data1 = static_cast<uint8_t>(eventJson.value("data1", 0));
            event.data2 = static_cast<uint8_t>(eventJson.value("data2", 0));
            event.channel = static_cast<uint8_t>(eventJson.value("channel", 0));

            if (event.streamId.empty()) {
                LOG_WARN({"ScheduleData"}, "Skipping MIDI event with empty streamId");
                continue;
            }

            schedule.midiEvents.push_back(event);
        }
    }

    // Build lookup maps
    schedule.buildLookupMaps();

    // Log summary
    std::ostringstream msg;
    msg << "Parsed schedule: " << schedule.streams.size() << " streams, "
        << schedule.audioSegments.size() << " audio segments, "
        << schedule.midiEvents.size() << " MIDI events";
    LOG_INFO({"ScheduleData"}, msg.str());

    return schedule;
}

