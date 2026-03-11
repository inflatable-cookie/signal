#pragma once

/// ChannelMetering - Per-channel metering data
///
/// Thread: Audio thread (read-only snapshots) or control thread (updates)
/// Ownership: Managed by MeteringService
/// Communication:
///   - Updated by audio thread via lock-free atomic operations
///   - Read by control thread for IPC publishing

#include <atomic>
#include <string>
#include <cstdint>

struct ChannelMetering {
    std::string channelId;
    float peak;      // Linear peak level (0.0 to 1.0+)
    float rms;       // RMS level (0.0 to 1.0+)
    std::uint64_t timestamp; // Last update timestamp (microseconds since epoch)

    ChannelMetering()
        : channelId("")
        , peak(0.0f)
        , rms(0.0f)
        , timestamp(0)
    {
    }

    ChannelMetering(
        const std::string& id,
        float p,
        float r,
        std::uint64_t ts
    )
        : channelId(id)
        , peak(p)
        , rms(r)
        , timestamp(ts)
    {
    }
};

/// Atomic metering snapshot for lock-free updates from audio thread
struct AtomicChannelMetering {
    std::string channelId;
    std::atomic<float> peak;
    std::atomic<float> rms;
    std::atomic<std::uint64_t> timestamp;

    AtomicChannelMetering()
        : channelId("")
        , peak(0.0f)
        , rms(0.0f)
        , timestamp(0)
    {
    }

    AtomicChannelMetering(const std::string& id)
        : channelId(id)
        , peak(0.0f)
        , rms(0.0f)
        , timestamp(0)
    {
    }

    /// Read current values into a snapshot (non-atomic, safe for control thread)
    ChannelMetering snapshot() const {
        ChannelMetering result;
        result.channelId = channelId;
        result.peak = peak.load(std::memory_order_acquire);
        result.rms = rms.load(std::memory_order_acquire);
        result.timestamp = timestamp.load(std::memory_order_acquire);
        return result;
    }

    /// Update from audio thread (lock-free)
    void update(float p, float r, std::uint64_t ts) {
        peak.store(p, std::memory_order_release);
        rms.store(r, std::memory_order_release);
        timestamp.store(ts, std::memory_order_release);
    }
};


