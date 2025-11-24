#include "core/ClipScheduler.hpp"
#include <algorithm>
#include <iostream>
#include <cmath>
#include <memory>

ClipScheduler::ClipScheduler()
{
    // Create empty schedule as initial state
    _emptySchedule = std::make_shared<ScheduleData>(120.0, 44100.0);
    _activeSchedule.store(_emptySchedule.get(), std::memory_order_release);

    std::cout << "[ClipScheduler] Initialised" << std::endl;
}

ClipScheduler::~ClipScheduler() {
    // Clear active schedule (shared_ptr will handle cleanup)
    _activeSchedule.store(nullptr, std::memory_order_release);
}

uint64_t ClipScheduler::beatsToSamples(double beats, double tempo, double sampleRate) {
    return static_cast<uint64_t>((beats / tempo) * 60.0 * sampleRate);
}

void ClipScheduler::setSchedule(
    const std::vector<ScheduledClip>& clips,
    double tempo,
    double sampleRate
) {
    // Build new schedule (control thread only, no locks needed)
    auto newSchedule = std::make_shared<ScheduleData>(tempo, sampleRate);

    // Create playback states for each clip
    for (const auto& clip : clips) {
        auto state = std::make_unique<ClipPlaybackState>();
        state->clipId = clip.clipId;
        state->channelId = clip.channelId;

        uint64_t startSamples = beatsToSamples(clip.startBeats, tempo, sampleRate);
        uint64_t durationSamples = beatsToSamples(clip.durationBeats, tempo, sampleRate);
        uint64_t endSamples = startSamples + durationSamples;

        state->isActive.store(false, std::memory_order_release);
        state->currentSample.store(0, std::memory_order_release);
        state->endSample.store(endSamples, std::memory_order_release);
        state->gainDb.store(clip.gainDb, std::memory_order_release);
        state->muted.store(clip.muted, std::memory_order_release);

        // Store in schedule
        std::string clipId = clip.clipId;
        newSchedule->clips[clipId] = std::move(state);
        newSchedule->clipStartSamples[clipId] = startSamples;
    }

    // Keep previous schedule alive until next swap (ensures audio thread safety)
    _previousSchedule = _currentSchedule;

    // Atomically swap the active schedule pointer
    // Old schedule kept alive in _previousSchedule until next swap
    _activeSchedule.store(newSchedule.get(), std::memory_order_release);

    // Update our current schedule pointer
    _currentSchedule = newSchedule;

    std::cout << "[ClipScheduler] Set schedule: " << clips.size() << " clips, tempo=" << tempo << ", sampleRate=" << sampleRate << std::endl;
}

void ClipScheduler::clearSchedule() {
    // Keep previous schedule alive until next swap
    _previousSchedule = _currentSchedule;

    // Swap to empty schedule atomically
    _activeSchedule.store(_emptySchedule.get(), std::memory_order_release);

    // Clear current schedule (will be recreated on next setSchedule)
    _currentSchedule.reset();

    std::cout << "[ClipScheduler] Cleared schedule" << std::endl;
}

std::vector<ClipPlaybackState*> ClipScheduler::getActiveClips(
    const std::string& channelId,
    uint64_t samplePosition
) const {
    std::vector<ClipPlaybackState*> active;

    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous schedule kept alive in _previousSchedule)
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);

    if (!schedule) {
        return active;  // No schedule available
    }

    // Use the snapshot for the entire query (pointer remains valid)
    for (const auto& pair : schedule->clips) {
        ClipPlaybackState* state = pair.second.get();
        if (state->channelId != channelId) {
            continue;
        }

        auto it = schedule->clipStartSamples.find(pair.first);
        if (it == schedule->clipStartSamples.end()) {
            continue;
        }

        uint64_t start = it->second;
        uint64_t end = state->endSample.load(std::memory_order_acquire);

        if (samplePosition >= start && samplePosition < end) {
            if (!state->muted.load(std::memory_order_acquire)) {
                active.push_back(state);
            }
        }
    }

    return active;
}

void ClipScheduler::updatePlayback(uint64_t samplePosition) {
    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous schedule kept alive in _previousSchedule)
    const ScheduleData* schedule = _activeSchedule.load(std::memory_order_acquire);

    if (!schedule) {
        return;  // No schedule available
    }

    // Use the snapshot for the entire update (pointer remains valid)
    for (const auto& pair : schedule->clips) {
        ClipPlaybackState* state = pair.second.get();

        auto it = schedule->clipStartSamples.find(pair.first);
        if (it == schedule->clipStartSamples.end()) {
            continue;
        }

        uint64_t start = it->second;
        uint64_t end = state->endSample.load(std::memory_order_acquire);

        bool shouldBeActive = (samplePosition >= start && samplePosition < end);
        bool isActive = state->isActive.load(std::memory_order_acquire);

        if (shouldBeActive != isActive) {
            state->isActive.store(shouldBeActive, std::memory_order_release);
            if (shouldBeActive) {
                state->currentSample.store(samplePosition - start, std::memory_order_release);
            }
        } else if (shouldBeActive) {
            state->currentSample.store(samplePosition - start, std::memory_order_release);
        }
    }
}
