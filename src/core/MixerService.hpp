#pragma once

/// MixerService - Manages per-channel mixer state (gain, mute, solo)
///
/// Thread: Control thread (main thread)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (MixerDomain handlers)
///   - Read by audio thread via lock-free atomic operations
///   - Provides snapshot API for IPC updates

#include <atomic>
#include <unordered_map>
#include <mutex>
#include <string>

/// Per-channel mixer state (atomic for lock-free audio thread access)
struct ChannelMixerState {
    std::string channelId;
    std::atomic<float> gain;           // Linear gain (0.0 to 4.0)
    std::atomic<bool> isMuted;         // Explicit mute state
    std::atomic<bool> isSoloed;        // Solo state
    std::atomic<bool> effectiveMuted;  // Computed effective mute (considering solo)

    ChannelMixerState()
        : channelId("")
        , gain(1.0f)
        , isMuted(false)
        , isSoloed(false)
        , effectiveMuted(false)
    {
    }

    ChannelMixerState(const std::string& id)
        : channelId(id)
        , gain(1.0f)
        , isMuted(false)
        , isSoloed(false)
        , effectiveMuted(false)
    {
    }
};

class MixerService {
public:
    MixerService();
    ~MixerService();

    /// Register a channel for mixer control (called when channel is created)
    void registerChannel(const std::string& channelId);

    /// Unregister a channel (called when channel is removed)
    void unregisterChannel(const std::string& channelId);

    /// Update mixer state for a channel (called from IPC thread)
    void updateChannel(
        const std::string& channelId,
        float gain,
        bool isMuted,
        bool isSoloed,
        bool effectiveMuted
    );

    /// Get atomic mixer state for audio thread (returns nullptr if not found)
    ChannelMixerState* getChannelState(const std::string& channelId);

    /// Get effective gain for a channel (gain * mute factor)
    /// Returns 0.0 if effectiveMuted is true, otherwise returns gain
    float getEffectiveGain(const std::string& channelId) const;

private:
    /// Recompute effective mute for all channels (called after solo changes)
    void recomputeEffectiveMutes();

    mutable std::mutex _mutex; // Protects _channels map structure
    std::unordered_map<std::string, std::unique_ptr<ChannelMixerState>> _channels;
};

