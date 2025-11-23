#include "core/ClipScheduler.hpp"
#include <algorithm>
#include <iostream>
#include <cmath>

ClipScheduler::ClipScheduler()
    : _tempo(120.0)
    , _sampleRate(44100.0)
{
    std::cout << "[ClipScheduler] Initialised" << std::endl;
}

ClipScheduler::~ClipScheduler() = default;

uint64_t ClipScheduler::beatsToSamples(double beats, double tempo, double sampleRate) {
    return static_cast<uint64_t>((beats / tempo) * 60.0 * sampleRate);
}

void ClipScheduler::setSchedule(
    const std::vector<ScheduledClip>& clips,
    double tempo,
    double sampleRate
) {
    std::lock_guard<std::mutex> lock(_mutex);

    _tempo = tempo;
    _sampleRate = sampleRate;

    // Clear existing clips
    _clips.clear();
    _clipStartSamples.clear();

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

        // Store in map
        _clips[clip.clipId] = std::move(state);
        _clipStartSamples[clip.clipId] = startSamples;
    }

    std::cout << "[ClipScheduler] Set schedule: " << clips.size() << " clips, tempo=" << tempo << ", sampleRate=" << sampleRate << std::endl;
}

void ClipScheduler::clearSchedule() {
    std::lock_guard<std::mutex> lock(_mutex);
    _clips.clear();
    _clipStartSamples.clear();
    std::cout << "[ClipScheduler] Cleared schedule" << std::endl;
}

std::vector<ClipPlaybackState*> ClipScheduler::getActiveClips(
    const std::string& channelId,
    uint64_t samplePosition
) const {
    std::vector<ClipPlaybackState*> active;

    // Note: We need to lock to iterate, but the returned pointers are safe
    // because the states themselves are not deleted while playback is active
    std::lock_guard<std::mutex> lock(_mutex);

    for (const auto& pair : _clips) {
        ClipPlaybackState* state = pair.second.get();
        if (state->channelId != channelId) {
            continue;
        }

        auto it = _clipStartSamples.find(pair.first);
        if (it == _clipStartSamples.end()) {
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
    std::lock_guard<std::mutex> lock(_mutex);

    for (const auto& pair : _clips) {
        ClipPlaybackState* state = pair.second.get();

        auto it = _clipStartSamples.find(pair.first);
        if (it == _clipStartSamples.end()) {
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

