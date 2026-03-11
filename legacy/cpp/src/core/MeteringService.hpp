#pragma once

/// MeteringService - Manages per-channel metering data
///
/// Thread: Control thread (registration, snapshots)
///         Audio thread (sample submission - lock-free)
/// Ownership: Owned by EngineHost
///
/// Real-time safety:
///   - Audio thread: submitSampleBlock() is lock-free (no allocations, no locks)
///   - Control thread: registerChannel(), snapshotAll() may use locks for map access
///
/// Usage:
///   1. Control thread: Register channels before audio starts (registerChannel)
///   2. Audio thread: Submit samples every block (submitSampleBlock)
///   3. Control thread: Get snapshots for IPC (snapshotAll)

#include "core/ChannelMetering.hpp"
#include <unordered_map>
#include <shared_mutex>
#include <vector>
#include <string>
#include <optional>
#include <memory>
#include <atomic>

// Forward declaration
class AudioBus;

/// Meter snapshot structure for IPC
struct MeterSnapshot {
    std::string id;  // Channel/meter ID (e.g. "master", "track-123", "bus-reverb")
    float peakL;     // Peak level (left channel or mono)
    float peakR;     // Peak level (right channel, 0.0f for mono)
    float rmsL;      // RMS level (left channel or mono)
    float rmsR;      // RMS level (right channel, 0.0f for mono)
    std::uint64_t timestamp; // Last update timestamp (microseconds since epoch)
};

class MeteringService {
public:
    MeteringService();
    ~MeteringService();

    /// Register a channel for metering (called on control thread before audio starts)
    /// Must be called before submitSampleBlock() is used for this channel
    void registerChannel(const std::string& channelId);

    /// Unregister a channel (called on control thread when channel is removed)
    void unregisterChannel(const std::string& channelId);

    /// Submit sample block for metering (called from audio thread - lock-free)
    /// Real-time safe: no allocations, no locks, deterministic
    /// @param meterId Meter/channel ID (e.g. "master", "track-123")
    /// @param interleavedData Audio samples (interleaved: L, R, L, R, ...)
    /// @param numChannels Number of channels (1 = mono, 2 = stereo, etc.)
    /// @param numFrames Number of frames (samples per channel)
    void submitSampleBlock(
        const std::string& meterId,
        const float* interleavedData,
        int numChannels,
        int numFrames
    ) const;

    /// Submit channel levels (alternative API - called from audio thread - lock-free)
    /// Real-time safe: no allocations, no locks
    /// @param meterId Meter/channel ID
    /// @param peakL Peak level (left or mono)
    /// @param peakR Peak level (right, 0.0f for mono)
    /// @param rmsL RMS level (left or mono)
    /// @param rmsR RMS level (right, 0.0f for mono)
    void submitChannelLevels(
        const std::string& meterId,
        float peakL,
        float peakR,
        float rmsL,
        float rmsR
    ) const;

    /// Get snapshot of all current metering data (called on control thread)
    /// Returns vector of MeterSnapshot for IPC publishing
    /// Applies decay/smoothing if needed (currently simple passthrough)
    std::vector<MeterSnapshot> getSnapshotAndDecay();

    /// Get snapshot for a specific channel (called on control thread)
    std::optional<MeterSnapshot> getSnapshotChannel(const std::string& channelId) const;

    /// Legacy: Capture levels from AudioBus (deprecated - use submitSampleBlock instead)
    /// @deprecated Use submitSampleBlock() for better real-time safety
    void captureLevels(
        const class AudioBus& output,
        const std::string& channelId = "master"
    ) const;

private:
    /// Internal metering entry with atomic state
    struct MeterEntry {
        std::string id;
        std::atomic<float> peakL;
        std::atomic<float> peakR;
        std::atomic<float> rmsL;
        std::atomic<float> rmsR;
        std::atomic<std::uint64_t> timestamp;

        MeterEntry(const std::string& meterId)
            : id(meterId)
            , peakL(0.0f)
            , peakR(0.0f)
            , rmsL(0.0f)
            , rmsR(0.0f)
            , timestamp(0)
        {
        }
    };

    /// Get meter entry pointer (lock-free read after registration)
    /// Returns nullptr if not registered (safe to call from audio thread)
    MeterEntry* getMeterEntry(const std::string& meterId) const;

    /// Protects _metering map structure
    /// Uses shared_mutex to allow concurrent reads (audio thread) while protecting writes (control thread)
    mutable std::shared_mutex _mutex;
    std::unordered_map<std::string, std::unique_ptr<MeterEntry>> _metering;
};

