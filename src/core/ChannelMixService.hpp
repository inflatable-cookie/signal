#pragma once

/// ChannelMixService - Manages per-channel channel‑mix state (gain, pan, mute, solo)
///
/// Thread: Control thread (main thread)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (ChannelMixDomain handlers)
///   - Read by audio thread via lock-free atomic operations
///   - Provides snapshot API for IPC updates
///
/// Phase 9 note:
///   - Per-channel gain/pan live on Fader nodes and are driven via the Node
///     domain (`node.setParameter`) and automation.
///   - ChannelMixService currently applies a final gain/mute stage when
///     mixing a hardware output node into the host output bus. This is a
///     temporary output path while the graph-level routing/mapping matures.
///
/// Architecture note: Channels in Signal are processing paths (not tracks).
/// A channel represents a processing path with nodes (lane → fx → fader → output).
/// Channel IDs come from Pulse's graph model and represent processing paths,
/// not track-level concepts.

#include <atomic>
#include <unordered_map>
#include <mutex>
#include <shared_mutex>
#include <string>

// Forward declarations
class AudioBuffer;
class AudioBus;

/// Per-channel channel‑mix state (atomic for lock-free audio thread access)
struct ChannelMixerState {
    std::string channelId;
    std::atomic<float> gain;           // Linear gain (0.0 to 4.0)
    std::atomic<float> pan;            // Pan position: -1.0 = Left, 0.0 = Centre, +1.0 = Right
    std::atomic<bool> isMuted;         // Explicit mute state
    std::atomic<bool> isSoloed;        // Solo state
    std::atomic<bool> effectiveMuted;  // Computed effective mute (considering solo)

    ChannelMixerState()
        : channelId()
        , gain(1.0f)
        , pan(0.0f)
        , isMuted(false)
        , isSoloed(false)
        , effectiveMuted(false)
    {
    }
};

class ChannelMixService {
public:
    ChannelMixService();
    ~ChannelMixService();

    /// Register a channel for channel‑mix control (called when channel is created)
    void registerChannel(const std::string& channelId);

    /// Unregister a channel (called when channel is removed)
    void unregisterChannel(const std::string& channelId);

    /// Update channel‑mix state for a channel (called from IPC thread)
    void updateChannel(
        const std::string& channelId,
        float gain,
        float pan,
        bool isMuted,
        bool isSoloed,
        bool effectiveMuted
    );

    /// Get atomic channel‑mix state for audio thread (returns nullptr if not found)
    ChannelMixerState* getChannelState(const std::string& channelId);

    /// Get effective gain for a channel (gain * mute factor)
    /// Returns 0.0 if effectiveMuted is true, otherwise returns gain
    float getEffectiveGain(const std::string& channelId) const;

    /// Apply channel-mix to a node output, writing into the host output bus (audio thread)
    /// Reads from node AudioBuffer, applies gain/mute/solo, writes to AudioBus.
    void applyChannelMixToBus(
        const class AudioBuffer& nodeOutput,
        class AudioBus& output,
        const std::string& channelId,
        bool applyGain
    ) const;

private:
    /// Recompute effective mute for all channels (called after solo changes)
    void recomputeEffectiveMutes();

    mutable std::shared_mutex _mutex; // Protects _channels map structure (allows concurrent reads)
    std::unordered_map<std::string, std::unique_ptr<ChannelMixerState>> _channels;
};
