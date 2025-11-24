#include "core/ScheduleData.hpp"
#include <algorithm>

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

