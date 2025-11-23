#pragma once

/// ClipScheduler - Manages scheduled clip playback
///
/// Thread: Control thread (main thread)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (EngineDomain handlers)
///   - Read by audio thread via lock-free snapshot
///   - Provides playback state for audio thread

#include <atomic>
#include <unordered_map>
#include <mutex>
#include <string>
#include <vector>
#include <cstdint>

/// Scheduled clip information
struct ScheduledClip {
    std::string clipId;
    std::string channelId;
    double startBeats;
    double durationBeats;
    float gainDb;
    bool muted;
    uint64_t startSamples;  // Converted from beats
    uint64_t endSamples;    // startSamples + durationSamples
    uint64_t durationSamples;
};

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
    std::vector<ClipPlaybackState*> getActiveClips(
        const std::string& channelId,
        uint64_t samplePosition
    ) const;

    /// Update playback state based on current sample position
    void updatePlayback(uint64_t samplePosition);

    /// Convert beats to samples
    static uint64_t beatsToSamples(double beats, double tempo, double sampleRate);

private:
    mutable std::mutex _mutex;
    std::unordered_map<std::string, std::unique_ptr<ClipPlaybackState>> _clips;
    // Store start samples for each clip (for efficient lookup)
    std::unordered_map<std::string, uint64_t> _clipStartSamples;
    double _tempo;
    double _sampleRate;
};

