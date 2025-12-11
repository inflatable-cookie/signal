#include "core/MixerService.hpp"
#include "core/AudioBus.hpp"
#include "core/NodeBuffers.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <memory>
#include <sstream>
#include <cmath>
#include <shared_mutex>

MixerService::MixerService() {
    LOG_INFO({"MixerService"}, "Initialised");
}

MixerService::~MixerService() = default;

void MixerService::registerChannel(const std::string& channelId) {
    std::unique_lock<std::shared_mutex> lock(_mutex);
    if (_channels.find(channelId) == _channels.end()) {
        _channels[channelId] = std::make_unique<ChannelMixerState>(channelId);
        LOG_DEBUG({"MixerService"}, std::string("Registered channel: ") + channelId);
    }
}

void MixerService::unregisterChannel(const std::string& channelId) {
    std::unique_lock<std::shared_mutex> lock(_mutex);
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
    std::unique_lock<std::shared_mutex> lock(_mutex);
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
    std::shared_lock<std::shared_mutex> lock(_mutex);
    auto it = _channels.find(channelId);
    if (it != _channels.end()) {
        return it->second.get();
    }
    return nullptr;
}

float MixerService::getEffectiveGain(const std::string& channelId) const {
    std::shared_lock<std::shared_mutex> lock(_mutex);
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

void MixerService::finalMix(
    const AudioBuffer& deviceNodeOutput,
    AudioBus& output,
    const std::string& channelId
) const {
    // Real-time safe: lock-free reads of atomic mixer state
    // Channel-aware: supports mono, stereo, and multi-channel layouts
    std::shared_lock<std::shared_mutex> lock(_mutex);
    auto it = _channels.find(channelId);

    float gain = 1.0f;
    float pan = 0.0f;
    bool effectiveMuted = false;

    if (it != _channels.end()) {
        const ChannelMixerState* state = it->second.get();
        gain = state->gain.load(std::memory_order_acquire);
        pan = state->pan.load(std::memory_order_acquire);
        effectiveMuted = state->effectiveMuted.load(std::memory_order_acquire);
    }

    lock.unlock();

    // Apply mute
    if (effectiveMuted) {
        output.clear();
        return;
    }

    // Copy and apply gain/pan from device node output to interleaved output bus
    // Channel-aware: pan only applies to stereo (2 channels), gain applies to all channels
    const int inputChannels = deviceNodeOutput.numChannels();
    const int outputChannels = output.numChannels();
    const int numChannels = std::min(inputChannels, outputChannels);
    const int numFrames = std::min(deviceNodeOutput.numFrames(), output.numFrames());

    float* outData = output.data();
    if (!outData) {
        return;
    }

    // Channel-aware processing: pan only for stereo, gain for all channels
    if (numChannels == 1) {
        // Mono: Apply gain only (no panning)
        const float* src = deviceNodeOutput.getChannelData(0);
        if (outputChannels >= 1) {
            for (int frame = 0; frame < numFrames; ++frame) {
                outData[frame * outputChannels + 0] = src[frame] * gain;
            }
        }
        // If output has more channels, fill with silence (or duplicate - handled by DeviceNode)
    } else if (numChannels == 2) {
        // Stereo: Apply gain and pan
        // Linear pan law: pan in [-1, 1] where -1 = full left, 0 = centre, +1 = full right
        // leftGain = gain * (1 - pan), rightGain = gain * (1 + pan)
        // This matches FaderNode's pan implementation
        float leftGain = (1.0f - pan) * gain;
        float rightGain = (1.0f + pan) * gain;

        const float* srcLeft = deviceNodeOutput.getChannelData(0);
        const float* srcRight = deviceNodeOutput.getChannelData(1);

        if (outputChannels >= 2) {
            for (int frame = 0; frame < numFrames; ++frame) {
                outData[frame * outputChannels + 0] = srcLeft[frame] * leftGain;
                outData[frame * outputChannels + 1] = srcRight[frame] * rightGain;
            }
        } else if (outputChannels == 1) {
            // Stereo input to mono output: sum with pan
            for (int frame = 0; frame < numFrames; ++frame) {
                float left = srcLeft[frame] * leftGain;
                float right = srcRight[frame] * rightGain;
                outData[frame] = (left + right) * 0.5f; // Average for mono
            }
        }
    } else {
        // Multi-channel (3+ channels): Apply gain uniformly (no panning)
        // Pan is a stereo-only concept; multi-channel layouts don't use pan
        for (int ch = 0; ch < numChannels; ++ch) {
            const float* src = deviceNodeOutput.getChannelData(ch);
            for (int frame = 0; frame < numFrames; ++frame) {
                outData[frame * outputChannels + ch] = src[frame] * gain;
            }
        }
    }
}
