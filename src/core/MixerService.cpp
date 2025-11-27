#include "core/MixerService.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <memory>
#include <sstream>

MixerService::MixerService() {
    LOG_INFO({"MixerService"}, "Initialised");
}

MixerService::~MixerService() = default;

void MixerService::registerChannel(const std::string& channelId) {
    std::lock_guard<std::mutex> lock(_mutex);
    if (_channels.find(channelId) == _channels.end()) {
        _channels[channelId] = std::make_unique<ChannelMixerState>(channelId);
        LOG_DEBUG({"MixerService"}, std::string("Registered channel: ") + channelId);
    }
}

void MixerService::unregisterChannel(const std::string& channelId) {
    std::lock_guard<std::mutex> lock(_mutex);
    _channels.erase(channelId);
        LOG_DEBUG({"MixerService"}, std::string("Unregistered channel: ") + channelId);
}

void MixerService::updateChannel(
    const std::string& channelId,
    float gain,
    float pan,
    bool isMuted,
    bool isSoloed,
    bool effectiveMuted
) {
    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _channels.find(channelId);
    if (it != _channels.end()) {
        ChannelMixerState* state = it->second.get();
        state->gain.store(gain, std::memory_order_release);
        // Clamp pan to -1.0 to +1.0
        float clampedPan = std::max(-1.0f, std::min(1.0f, pan));
        state->pan.store(clampedPan, std::memory_order_release);
        state->isMuted.store(isMuted, std::memory_order_release);
        state->isSoloed.store(isSoloed, std::memory_order_release);
        state->effectiveMuted.store(effectiveMuted, std::memory_order_release);
    } else {
        // Auto-register if channel doesn't exist
        _channels[channelId] = std::make_unique<ChannelMixerState>(channelId);
        ChannelMixerState* state = _channels[channelId].get();
        state->gain.store(gain, std::memory_order_release);
        // Clamp pan to -1.0 to +1.0
        float clampedPan = std::max(-1.0f, std::min(1.0f, pan));
        state->pan.store(clampedPan, std::memory_order_release);
        state->isMuted.store(isMuted, std::memory_order_release);
        state->isSoloed.store(isSoloed, std::memory_order_release);
        state->effectiveMuted.store(effectiveMuted, std::memory_order_release);
        LOG_DEBUG({"MixerService"}, std::string("Auto-registered and updated channel: ") + channelId);
    }

    // If solo state changed, recompute effective mutes for all channels
    if (isSoloed || std::any_of(_channels.begin(), _channels.end(),
        [](const auto& pair) {
            return pair.second->isSoloed.load(std::memory_order_acquire);
        })) {
        recomputeEffectiveMutes();
    }
}

ChannelMixerState* MixerService::getChannelState(const std::string& channelId) {
    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _channels.find(channelId);
    if (it != _channels.end()) {
        return it->second.get();
    }
    return nullptr;
}

float MixerService::getEffectiveGain(const std::string& channelId) const {
    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _channels.find(channelId);
    if (it != _channels.end()) {
        const ChannelMixerState* state = it->second.get();
        bool effectiveMuted = state->effectiveMuted.load(std::memory_order_acquire);
        if (effectiveMuted) {
            return 0.0f;
        }
        return state->gain.load(std::memory_order_acquire);
    }
    return 1.0f; // Default unity gain if channel not found
}

void MixerService::recomputeEffectiveMutes() {
    // Check if any channel is soloed
    bool hasAnySolo = std::any_of(_channels.begin(), _channels.end(),
        [](const auto& pair) {
            return pair.second->isSoloed.load(std::memory_order_acquire);
        });

    // Update effective mute for all channels
    for (auto& pair : _channels) {
        ChannelMixerState* state = pair.second.get();
        bool isMuted = state->isMuted.load(std::memory_order_acquire);
        bool isSoloed = state->isSoloed.load(std::memory_order_acquire);

        bool effectiveMuted;
        if (!hasAnySolo) {
            effectiveMuted = isMuted;
        } else {
            effectiveMuted = !isSoloed || isMuted;
        }

        state->effectiveMuted.store(effectiveMuted, std::memory_order_release);
    }
}

