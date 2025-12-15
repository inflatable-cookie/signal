#include "core/ChannelMixService.hpp"

#include "core/NodeBuffers.hpp"
#include "core/AudioBus.hpp"
#include "logging/Logging.hpp"

ChannelMixService::ChannelMixService() {
    LOG_INFO({"ChannelMixService"}, "Initialised");
}

ChannelMixService::~ChannelMixService() = default;

void ChannelMixService::registerChannel(const std::string& channelId) {
    std::unique_lock lock(_mutex);
    if (_channels.find(channelId) == _channels.end()) {
        auto state = std::make_unique<ChannelMixerState>();
        state->channelId = channelId;
        _channels.emplace(channelId, std::move(state));
        LOG_DEBUG({"ChannelMixService"}, std::string("Registered channel: ") + channelId);
    }
}

void ChannelMixService::unregisterChannel(const std::string& channelId) {
    std::unique_lock lock(_mutex);
    _channels.erase(channelId);
    LOG_DEBUG({"ChannelMixService"}, std::string("Unregistered channel: ") + channelId);
}

void ChannelMixService::updateChannel(
    const std::string& channelId,
    float gain,
    float pan,
    bool isMuted,
    bool isSoloed,
    bool effectiveMuted
) {
    std::unique_lock lock(_mutex);
    auto it = _channels.find(channelId);
    if (it == _channels.end()) {
        auto state = std::make_unique<ChannelMixerState>();
        state->channelId = channelId;
        state->gain.store(gain);
        state->pan.store(pan);
        state->isMuted.store(isMuted);
        state->isSoloed.store(isSoloed);
        state->effectiveMuted.store(effectiveMuted);
        _channels.emplace(channelId, std::move(state));
        LOG_DEBUG({"ChannelMixService"}, std::string("Auto-registered and updated channel: ") + channelId);
        return;
    }

    auto* state = it->second.get();
    state->gain.store(gain);
    state->pan.store(pan);
    state->isMuted.store(isMuted);
    state->isSoloed.store(isSoloed);
    state->effectiveMuted.store(effectiveMuted);
}

ChannelMixerState* ChannelMixService::getChannelState(const std::string& channelId) {
    std::shared_lock lock(_mutex);
    auto it = _channels.find(channelId);
    if (it == _channels.end()) {
        return nullptr;
    }
    return it->second.get();
}

float ChannelMixService::getEffectiveGain(const std::string& channelId) const {
    std::shared_lock lock(_mutex);
    auto it = _channels.find(channelId);
    if (it == _channels.end()) {
        return 1.0f;
    }
    const auto* state = it->second.get();
    if (state->effectiveMuted.load()) {
        return 0.0f;
    }
    return state->gain.load();
}

void ChannelMixService::recomputeEffectiveMutes() {
    std::shared_lock lock(_mutex);
    bool anySolo = false;
    for (const auto& [_, statePtr] : _channels) {
        if (statePtr->isSoloed.load()) {
            anySolo = true;
            break;
        }
    }

    for (const auto& [_, statePtr] : _channels) {
        const bool soloed = statePtr->isSoloed.load();
        const bool muted = statePtr->isMuted.load();
        if (!anySolo) {
            statePtr->effectiveMuted.store(muted);
        } else {
            statePtr->effectiveMuted.store(!soloed || muted);
        }
    }
}

void ChannelMixService::applyChannelMixToBus(
    const AudioBuffer& nodeOutput,
    AudioBus& output,
    const std::string& channelId,
    bool applyGain
) const {
    float gain = 1.0f;

    {
        std::shared_lock lock(_mutex);
        auto it = _channels.find(channelId);
        if (it != _channels.end()) {
            const auto* state = it->second.get();
            if (state->effectiveMuted.load()) {
                gain = 0.0f;
            } else if (applyGain) {
                gain = state->gain.load();
            }
        }
    }

    const int numChannels = output.numChannels();
    const int numFrames = output.numFrames();

    // Convert from deinterleaved AudioBuffer to interleaved AudioBus
    for (int frame = 0; frame < numFrames; ++frame) {
        for (int ch = 0; ch < numChannels; ++ch) {
            const float* inChannel = nodeOutput.getChannelData(ch);
            float sample = inChannel[frame] * gain;
            output.setSample(frame, ch, sample);
        }
    }
}
