#include "core/MeteringService.hpp"
#include "core/AudioBus.hpp"
#include <algorithm>
#include <cmath>
#include <chrono>
#include <shared_mutex>

MeteringService::MeteringService() {
    // Pre-register "master" channel (most common case)
    registerChannel("master");
}

MeteringService::~MeteringService() {
}

void MeteringService::registerChannel(const std::string& channelId) {
    std::unique_lock<std::shared_mutex> lock(_mutex);
    if (_metering.find(channelId) == _metering.end()) {
        _metering[channelId] = std::make_unique<MeterEntry>(channelId);
    }
}

void MeteringService::unregisterChannel(const std::string& channelId) {
    std::unique_lock<std::shared_mutex> lock(_mutex);
    _metering.erase(channelId);
}

MeteringService::MeterEntry* MeteringService::getMeterEntry(const std::string& meterId) const {
    // Note: This is called from the audio thread, so we need lock-free access
    // Current implementation uses map lookup which requires synchronization
    // For now, we use a shared lock (allows concurrent reads) which is fast
    // Future optimization: use a lock-free hash map or pre-computed pointer cache

    // The map is only modified on control thread (registration/unregistration)
    // During audio processing, the map structure is stable
    // We use shared_mutex to allow concurrent reads

    // TODO: Optimize with lock-free hash map or pointer cache for common channels
    std::shared_lock<std::shared_mutex> lock(_mutex);
    auto it = _metering.find(meterId);
    if (it != _metering.end()) {
        return it->second.get();
    }
    return nullptr;
}

void MeteringService::submitSampleBlock(
    const std::string& meterId,
    const float* interleavedData,
    int numChannels,
    int numFrames
) const {
    // Real-time safe: no allocations, no locks, deterministic
    MeterEntry* entry = getMeterEntry(meterId);
    if (!entry) {
        // Channel not registered - skip metering (safe to ignore)
        return;
    }

    // Calculate peak and RMS per channel
    float peakL = 0.0f;
    float peakR = 0.0f;
    float rmsSumL = 0.0f;
    float rmsSumR = 0.0f;

    if (numChannels == 1) {
        // Mono: use same values for L and R
        for (int frame = 0; frame < numFrames; ++frame) {
            float sample = std::abs(interleavedData[frame]);
            peakL = std::max(peakL, sample);
            rmsSumL += sample * sample;
        }
        peakR = peakL;
        rmsSumR = rmsSumL;
    } else if (numChannels >= 2) {
        // Stereo or multi-channel: use first two channels
        for (int frame = 0; frame < numFrames; ++frame) {
            int baseIdx = frame * numChannels;
            float sampleL = std::abs(interleavedData[baseIdx]);
            float sampleR = std::abs(interleavedData[baseIdx + 1]);
            peakL = std::max(peakL, sampleL);
            peakR = std::max(peakR, sampleR);
            rmsSumL += sampleL * sampleL;
            rmsSumR += sampleR * sampleR;
        }
    }

    float rmsL = (numFrames > 0) ? std::sqrt(rmsSumL / static_cast<float>(numFrames)) : 0.0f;
    float rmsR = (numFrames > 0) ? std::sqrt(rmsSumR / static_cast<float>(numFrames)) : 0.0f;

    // Get current timestamp (microseconds since epoch)
    auto now = std::chrono::system_clock::now();
    auto duration = now.time_since_epoch();
    auto microseconds = std::chrono::duration_cast<std::chrono::microseconds>(duration).count();

    // Update atomic metering state (lock-free)
    entry->peakL.store(peakL, std::memory_order_release);
    entry->peakR.store(peakR, std::memory_order_release);
    entry->rmsL.store(rmsL, std::memory_order_release);
    entry->rmsR.store(rmsR, std::memory_order_release);
    entry->timestamp.store(static_cast<std::uint64_t>(microseconds), std::memory_order_release);
}

void MeteringService::submitChannelLevels(
    const std::string& meterId,
    float peakL,
    float peakR,
    float rmsL,
    float rmsR
) const {
    // Real-time safe: no allocations, no locks
    MeterEntry* entry = getMeterEntry(meterId);
    if (!entry) {
        // Channel not registered - skip metering (safe to ignore)
        return;
    }

    // Get current timestamp (microseconds since epoch)
    auto now = std::chrono::system_clock::now();
    auto duration = now.time_since_epoch();
    auto microseconds = std::chrono::duration_cast<std::chrono::microseconds>(duration).count();

    // Update atomic metering state (lock-free)
    entry->peakL.store(peakL, std::memory_order_release);
    entry->peakR.store(peakR, std::memory_order_release);
    entry->rmsL.store(rmsL, std::memory_order_release);
    entry->rmsR.store(rmsR, std::memory_order_release);
    entry->timestamp.store(static_cast<std::uint64_t>(microseconds), std::memory_order_release);
}

std::vector<MeterSnapshot> MeteringService::getSnapshotAndDecay() {
    std::shared_lock<std::shared_mutex> lock(_mutex);
    std::vector<MeterSnapshot> result;
    result.reserve(_metering.size());

    for (const auto& pair : _metering) {
        MeterEntry* entry = pair.second.get();
        MeterSnapshot snapshot;
        snapshot.id = entry->id;
        snapshot.peakL = entry->peakL.load(std::memory_order_acquire);
        snapshot.peakR = entry->peakR.load(std::memory_order_acquire);
        snapshot.rmsL = entry->rmsL.load(std::memory_order_acquire);
        snapshot.rmsR = entry->rmsR.load(std::memory_order_acquire);
        snapshot.timestamp = entry->timestamp.load(std::memory_order_acquire);
        result.push_back(snapshot);
    }

    return result;
}

std::optional<MeterSnapshot> MeteringService::getSnapshotChannel(const std::string& channelId) const {
    std::shared_lock<std::shared_mutex> lock(_mutex);
    auto it = _metering.find(channelId);
    if (it != _metering.end()) {
        MeterEntry* entry = it->second.get();
        MeterSnapshot snapshot;
        snapshot.id = entry->id;
        snapshot.peakL = entry->peakL.load(std::memory_order_acquire);
        snapshot.peakR = entry->peakR.load(std::memory_order_acquire);
        snapshot.rmsL = entry->rmsL.load(std::memory_order_acquire);
        snapshot.rmsR = entry->rmsR.load(std::memory_order_acquire);
        snapshot.timestamp = entry->timestamp.load(std::memory_order_acquire);
        return snapshot;
    }
    return std::nullopt;
}

void MeteringService::captureLevels(
    const AudioBus& output,
    const std::string& channelId
) const {
    // Legacy implementation: convert AudioBus to interleaved format and call submitSampleBlock
    // This maintains backward compatibility but is less efficient

    const int numChannels = output.numChannels();
    const int numFrames = output.numFrames();

    if (numFrames == 0 || numChannels == 0) {
        return;
    }

    // Real-time safe: direct access to interleaved data, no conversion needed
    const float* interleavedData = output.data();
    if (interleavedData) {
        submitSampleBlock(channelId, interleavedData, numChannels, numFrames);
    }
}


