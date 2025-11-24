#pragma once

/// ClipScheduler - Manages scheduled clip playback
///
/// Thread: Control thread (main thread) for updates, audio thread for reads
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (EngineDomain handlers) - builds new ScheduleData
///   - Read by audio thread via lock-free atomic pointer (no locks)
///   - Provides playback state for audio thread
///
/// Thread Safety:
///   - Control thread: builds new ScheduleData in build buffer, atomically swaps pointer
///   - Audio thread: reads atomic pointer once per renderBlock, uses snapshot for entire block
///   - No locks in audio thread path

#include "core/ScheduleData.hpp"
#include <atomic>
#include <memory>
#include <string>
#include <vector>
#include <cstdint>

/// Scheduled clip information (input from IPC)
struct ScheduledClip {
    std::string clipId;
    std::string channelId;
    double startBeats;
    double durationBeats;
    float gainDb;
    bool muted;
};

class ClipScheduler {
public:
    ClipScheduler();
    ~ClipScheduler();

    /// Set schedule from engine schedule payload
    /// tempo: BPM, sampleRate: Hz
    void setSchedule(
        const std::vector<ScheduledClip>& clips,
        double tempo,
        double sampleRate
    );

    /// Clear all scheduled clips
    void clearSchedule();

    /// Get active clips for a channel at a given sample position
    /// Thread-safe: reads atomic pointer (lock-free)
    /// @param channelId Channel to query
    /// @param samplePosition Current sample position
    /// @return Vector of active clip playback states (pointers valid for current render block)
    std::vector<ClipPlaybackState*> getActiveClips(
        const std::string& channelId,
        uint64_t samplePosition
    ) const;

    /// Update playback state based on current sample position
    /// Thread-safe: reads atomic pointer (lock-free)
    /// @param samplePosition Current sample position
    void updatePlayback(uint64_t samplePosition);

    /// Convert beats to samples
    static uint64_t beatsToSamples(double beats, double tempo, double sampleRate);

private:
    // Active schedule snapshot (read by audio thread, swapped by control thread)
    // Using raw pointer with shared_ptr for lifetime management
    std::atomic<const ScheduleData*> _activeSchedule;

    // Empty schedule (used when clearing)
    std::shared_ptr<ScheduleData> _emptySchedule;

    // Current schedule (control thread only, used for building new schedules)
    std::shared_ptr<ScheduleData> _currentSchedule;

    // Keep previous schedule alive until next swap (ensures audio thread safety)
    std::shared_ptr<ScheduleData> _previousSchedule;
};

