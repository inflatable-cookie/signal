#pragma once

/// ScheduleData - Immutable schedule snapshot for audio thread
///
/// Thread: Audio thread (read-only)
/// Ownership: Owned by ClipScheduler, swapped atomically
///
/// This structure contains a complete, self-contained snapshot of the schedule
/// that can be safely read from the audio thread without locking.

#include <string>
#include <unordered_map>
#include <vector>
#include <cstdint>
#include <memory>

/// Clip playback state (for audio thread)
struct ClipPlaybackState {
    std::string clipId;
    std::string channelId;
    std::atomic<bool> isActive;
    std::atomic<uint64_t> currentSample;
    std::atomic<uint64_t> endSample;
    std::atomic<float> gainDb;
    std::atomic<bool> muted;

    ClipPlaybackState()
        : isActive(false)
        , currentSample(0)
        , endSample(0)
        , gainDb(0.0f)
        , muted(false)
    {
    }
};

/// Immutable schedule snapshot
///
/// This structure is fully self-contained and can be safely read from the
/// audio thread. Once created, it is never modified - new schedules create
/// new ScheduleData instances.
struct ScheduleData {
    // Map from clip ID to playback state
    std::unordered_map<std::string, std::unique_ptr<ClipPlaybackState>> clips;

    // Map from clip ID to start sample position (for efficient lookup)
    std::unordered_map<std::string, uint64_t> clipStartSamples;

    // Tempo and sample rate (immutable for this snapshot)
    double tempo;
    double sampleRate;

    ScheduleData(double tempo, double sampleRate)
        : tempo(tempo)
        , sampleRate(sampleRate)
    {
    }

    // Non-copyable (we use unique_ptr for ownership)
    ScheduleData(const ScheduleData&) = delete;
    ScheduleData& operator=(const ScheduleData&) = delete;

    // Movable
    ScheduleData(ScheduleData&&) = default;
    ScheduleData& operator=(ScheduleData&&) = default;
};

