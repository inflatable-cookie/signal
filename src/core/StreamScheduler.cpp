#include "core/StreamScheduler.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <memory>
#include <sstream>

StreamScheduler::StreamScheduler()
{
    // Create empty schedule as initial state
    _emptySchedule = std::make_shared<ScheduleData>(44100.0, 120.0);
    _activeSchedule.store(_emptySchedule.get(), std::memory_order_release);

    LOG_INFO({"StreamScheduler"}, "Initialised");
}

StreamScheduler::~StreamScheduler() {
    // Clear active schedule (shared_ptr will handle cleanup)
    _activeSchedule.store(nullptr, std::memory_order_release);
}

void StreamScheduler::setSchedule(const ScheduleData& schedule) {
    // Build new schedule (control thread only, no locks needed)
    auto newSchedule = std::make_shared<ScheduleData>(schedule.sampleRate, schedule.tempoMap.defaultTempo);

    // Copy all components
    newSchedule->streams = schedule.streams;
    newSchedule->audioSegments = schedule.audioSegments;
    newSchedule->midiEvents = schedule.midiEvents;
    newSchedule->tempoMap = schedule.tempoMap;

    // Build lookup maps for efficient audio thread access
    newSchedule->buildLookupMaps();

    // Keep previous schedule alive until next swap (ensures audio thread safety)
    _previousSchedule = _currentSchedule;

    // Atomically swap the active schedule pointer
    // Old schedule kept alive in _previousSchedule until next swap
    _activeSchedule.store(newSchedule.get(), std::memory_order_release);

    // Update our current schedule pointer
    _currentSchedule = newSchedule;

    std::ostringstream msg;
    msg << "Set schedule: " << schedule.streams.size() << " streams, "
        << schedule.audioSegments.size() << " audio segments, "
        << schedule.midiEvents.size() << " MIDI events";
    LOG_INFO({"StreamScheduler"}, msg.str());
}

void StreamScheduler::clearSchedule() {
    // Keep previous schedule alive until next swap
    _previousSchedule = _currentSchedule;

    // Swap to empty schedule atomically
    _activeSchedule.store(_emptySchedule.get(), std::memory_order_release);

    // Clear current schedule (will be recreated on next setSchedule)
    _currentSchedule.reset();

    LOG_INFO({"StreamScheduler"}, "Cleared schedule");
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

bool StreamScheduler::hasSchedule() const noexcept {
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);
    if (!schedule) {
        return false;
    }
    // Check if schedule has streams (not empty)
    return !schedule->streams.empty();
}

int StreamScheduler::getActiveStreamCount() const noexcept {
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);
    if (!schedule) {
        return 0;
    }
    return static_cast<int>(schedule->streams.size());
}

const ScheduleData* StreamScheduler::getSchedule() const {
    return _activeSchedule.load(std::memory_order_acquire);
}

bool StreamScheduler::hasActiveStreams(uint64_t samplePosition) const noexcept {
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);
    if (!schedule || schedule->streams.empty()) {
        return false;
    }

    // Check if any stream has active audio segments at this position
    for (const auto& stream : schedule->streams) {
        auto it = schedule->audioSegmentsByStream.find(stream.streamId);
        if (it != schedule->audioSegmentsByStream.end()) {
            for (const AudioSegmentCompiled* segment : it->second) {
                if (samplePosition >= segment->startSamples && samplePosition < segment->endSamples) {
                    return true;
                }
            }
        }

        // Check if any stream has MIDI events near this position (within current block)
        // For efficiency, check a small range around current position
        constexpr uint64_t MIDI_CHECK_RANGE = 512; // One block size
        auto midiIt = schedule->midiEventsByStream.find(stream.streamId);
        if (midiIt != schedule->midiEventsByStream.end()) {
            for (const MidiEventCompiled* event : midiIt->second) {
                if (event->timeSamples >= samplePosition && event->timeSamples < samplePosition + MIDI_CHECK_RANGE) {
                    return true;
                }
            }
        }
    }

    return false;
}

