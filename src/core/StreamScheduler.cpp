#include "core/StreamScheduler.hpp"
#include <algorithm>
#include <iostream>
#include <memory>

StreamScheduler::StreamScheduler()
{
    // Create empty schedule as initial state
    _emptySchedule = std::make_shared<ScheduleData>(44100.0, 120.0);
    _activeSchedule.store(_emptySchedule.get(), std::memory_order_release);

    std::cout << "[StreamScheduler] Initialised" << std::endl;
}

StreamScheduler::~StreamScheduler() {
    // Clear active schedule (shared_ptr will handle cleanup)
    _activeSchedule.store(nullptr, std::memory_order_release);
}

void StreamScheduler::setSchedule(
    const std::vector<StreamDescriptor>& streams,
    const std::vector<AudioSegmentCompiled>& audioSegments,
    const std::vector<MidiEventCompiled>& midiEvents,
    const TempoMap& tempoMap,
    double sampleRate
) {
    // Build new schedule (control thread only, no locks needed)
    auto newSchedule = std::make_shared<ScheduleData>(sampleRate, tempoMap.defaultTempo);

    // Copy streams
    newSchedule->streams = streams;

    // Copy audio segments
    newSchedule->audioSegments = audioSegments;

    // Copy MIDI events
    newSchedule->midiEvents = midiEvents;

    // Copy tempo map
    newSchedule->tempoMap = tempoMap;

    // Build lookup maps for efficient audio thread access
    newSchedule->buildLookupMaps();

    // Keep previous schedule alive until next swap (ensures audio thread safety)
    _previousSchedule = _currentSchedule;

    // Atomically swap the active schedule pointer
    // Old schedule kept alive in _previousSchedule until next swap
    _activeSchedule.store(newSchedule.get(), std::memory_order_release);

    // Update our current schedule pointer
    _currentSchedule = newSchedule;

    std::cout << "[StreamScheduler] Set schedule: " << streams.size() << " streams, "
              << audioSegments.size() << " audio segments, "
              << midiEvents.size() << " MIDI events" << std::endl;
}

void StreamScheduler::clearSchedule() {
    // Keep previous schedule alive until next swap
    _previousSchedule = _currentSchedule;

    // Swap to empty schedule atomically
    _activeSchedule.store(_emptySchedule.get(), std::memory_order_release);

    // Clear current schedule (will be recreated on next setSchedule)
    _currentSchedule.reset();

    std::cout << "[StreamScheduler] Cleared schedule" << std::endl;
}

std::vector<const AudioSegmentCompiled*> StreamScheduler::getActiveAudioSegments(
    const std::string& streamId,
    uint64_t samplePosition
) const {
    std::vector<const AudioSegmentCompiled*> active;

    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous schedule kept alive in _previousSchedule)
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);

    if (!schedule) {
        return active;  // No schedule available
    }

    // Use the snapshot for the entire query (pointer remains valid)
    auto it = schedule->audioSegmentsByStream.find(streamId);
    if (it == schedule->audioSegmentsByStream.end()) {
        return active;  // No segments for this stream
    }

    // Find active segments (overlapping with samplePosition)
    for (const AudioSegmentCompiled* segment : it->second) {
        if (samplePosition >= segment->startSamples && samplePosition < segment->endSamples) {
            active.push_back(segment);
        }
    }

    return active;
}

std::vector<const MidiEventCompiled*> StreamScheduler::getMidiEventsInRange(
    const std::string& streamId,
    uint64_t startSample,
    uint64_t endSample
) const {
    std::vector<const MidiEventCompiled*> events;

    // Read atomic pointer once (lock-free)
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);

    if (!schedule) {
        return events;  // No schedule available
    }

    // Use the snapshot for the entire query (pointer remains valid)
    auto it = schedule->midiEventsByStream.find(streamId);
    if (it == schedule->midiEventsByStream.end()) {
        return events;  // No events for this stream
    }

    // Find events in range
    for (const MidiEventCompiled* event : it->second) {
        if (event->timeSamples >= startSample && event->timeSamples < endSample) {
            events.push_back(event);
        }
    }

    return events;
}

const ScheduleData* StreamScheduler::getSchedule() const {
    return _activeSchedule.load(std::memory_order_acquire);
}

