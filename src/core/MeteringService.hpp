#pragma once

/// MeteringService - Manages per-channel metering data
///
/// Thread: Control thread (main thread)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by audio thread via lock-free atomic operations
///   - Read by control thread for IPC publishing
///   - Provides snapshot API for periodic IPC updates

#include "core/ChannelMetering.hpp"
#include <unordered_map>
#include <mutex>
#include <vector>
#include <string>
#include <optional>
#include <memory>

class MeteringService {
public:
    MeteringService();
    ~MeteringService();

    /// Register a channel for metering (called when channel is created)
    void registerChannel(const std::string& channelId);

    /// Unregister a channel (called when channel is removed)
    void unregisterChannel(const std::string& channelId);

    /// Get atomic metering handle for audio thread updates
    /// Returns nullptr if channel not registered
    AtomicChannelMetering* getAtomicMetering(const std::string& channelId);

    /// Get snapshot of all current metering data (for IPC publishing)
    std::vector<ChannelMetering> snapshotAll() const;

    /// Get snapshot for a specific channel
    std::optional<ChannelMetering> snapshotChannel(const std::string& channelId) const;

private:
    mutable std::mutex _mutex; // Protects _metering map structure
    std::unordered_map<std::string, std::unique_ptr<AtomicChannelMetering>> _metering;
};

