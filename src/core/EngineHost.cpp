#include "core/EngineHost.hpp"
#include "core/AudioThread.hpp"
#include "core/MeteringService.hpp"
#include "core/MixerService.hpp"
#include "core/AutomationService.hpp"
#include "core/ClipScheduler.hpp"
#include <iostream>
#include <memory>
#include <cstdint>
#include <chrono>
#include <cmath>
#include <cstring>
#include <unordered_set>
#include <unordered_map>

EngineHost::EngineHost()
    : _state(State::Stopped)
    , _lastError(std::nullopt)
    , _shuttingDown(false)
    , _playheadSamples(0)
{
    _audioThread = std::make_unique<AudioThread>();
    _meteringService = std::make_unique<MeteringService>();
    _mixerService = std::make_unique<MixerService>();
    _automationService = std::make_unique<AutomationService>();
    _clipScheduler = std::make_unique<ClipScheduler>();
    setupAudioCallback();
    std::cout << "[EngineHost] Created" << std::endl;
}

EngineHost::~EngineHost() {
    if (_state == State::Running || _state == State::Starting) {
        stop();
    }
    std::cout << "[EngineHost] Destroyed" << std::endl;
}

void EngineHost::start() {
    if (_shuttingDown) {
        std::cout << "[EngineHost] Cannot start: shutting down" << std::endl;
        return;
    }

    if (_state == State::Running) {
        std::cout << "[EngineHost] Already running" << std::endl;
        return;
    }

    if (_state == State::Error) {
        std::cout << "[EngineHost] Cannot start: in error state" << std::endl;
        return;
    }

    _state = State::Starting;
    clearError();

    // Wire up metering service to audio thread
    _audioThread->setMeteringService(_meteringService.get());

    _audioThread->start();

    // After audio thread starts successfully, transition to running
    _state = State::Running;
    std::cout << "[EngineHost] Started" << std::endl;
}

void EngineHost::stop() {
    if (_state == State::Stopped) {
        std::cout << "[EngineHost] Already stopped" << std::endl;
        return;
    }

    _state = State::Stopped;
    _audioThread->stop();
    std::cout << "[EngineHost] Stopped" << std::endl;
}

void EngineHost::reset() {
    stop();
    clearError();
    _transportState = TransportState();
    _playheadSamples.store(0, std::memory_order_release);
    _clipScheduler->clearSchedule();
    std::cout << "[EngineHost] Reset" << std::endl;
}

void EngineHost::shutdown() {
    if (_shuttingDown) {
        return;
    }

    _shuttingDown = true;
    stop();
    std::cout << "[EngineHost] Shutdown complete" << std::endl;
}

EngineHost::State EngineHost::state() const noexcept {
    return _state;
}

std::optional<std::string> EngineHost::lastError() const noexcept {
    return _lastError;
}

void EngineHost::setError(const std::string& error) {
    _state = State::Error;
    _lastError = error;
    std::cout << "[EngineHost] Error: " << error << std::endl;
}

void EngineHost::clearError() {
    if (_state == State::Error) {
        _state = State::Stopped;
    }
    _lastError = std::nullopt;
}

TransportState& EngineHost::transport() {
    return _transportState;
}

const TransportState& EngineHost::transport() const {
    return _transportState;
}

double EngineHost::getCpuLoad() const {
    // Stub implementation - return 0.0 for now
    return 0.0;
}

uint64_t EngineHost::getXruns() const {
    // Stub implementation - return 0 for now
    return 0;
}

double EngineHost::getSampleRate() const {
    return SAMPLE_RATE;
}

size_t EngineHost::getBlockSize() const {
    return BLOCK_SIZE;
}

MeteringService& EngineHost::metering() {
    return *_meteringService;
}

const MeteringService& EngineHost::metering() const {
    return *_meteringService;
}

MixerService& EngineHost::mixer() {
    return *_mixerService;
}

const MixerService& EngineHost::mixer() const {
    return *_mixerService;
}

AutomationService& EngineHost::automation() {
    return *_automationService;
}

const AutomationService& EngineHost::automation() const {
    return *_automationService;
}

ClipScheduler& EngineHost::clipScheduler() {
    return *_clipScheduler;
}

const ClipScheduler& EngineHost::clipScheduler() const {
    return *_clipScheduler;
}

uint64_t EngineHost::getPlayheadSamples() const noexcept {
    return _playheadSamples.load(std::memory_order_acquire);
}

void EngineHost::setPlayheadSamples(uint64_t samples) noexcept {
    _playheadSamples.store(samples, std::memory_order_release);
}

void EngineHost::setupAudioCallback() {
    _audioThread->setCallback([this](float* buffer, size_t numFrames, int numChannels) {
        this->audioCallback(buffer, numFrames, numChannels);
    });
}

void EngineHost::audioCallback(float* buffer, size_t numFrames, int numChannels) {
    // Clear buffer
    std::memset(buffer, 0, numFrames * numChannels * sizeof(float));

    // Get current playhead position
    uint64_t currentPlayhead = _playheadSamples.load(std::memory_order_acquire);

    // Check transport state (lock-free read)
    const auto& transport = _transportState;
    if (!transport.isPlaying) {
        // Not playing - output silence, but still advance playhead for seek accuracy
        _playheadSamples.store(currentPlayhead + numFrames, std::memory_order_release);
        return;
    }

    // Handle loop wrapping
    uint64_t effectivePlayhead = currentPlayhead;
    bool wrapped = false;
    if (transport.loopEnabled && transport.loopRegion.has_value()) {
        const auto& loop = transport.loopRegion.value();
        uint64_t loopStartSamples = static_cast<uint64_t>(loop.startSeconds * SAMPLE_RATE);
        uint64_t loopEndSamples = static_cast<uint64_t>(loop.endSeconds * SAMPLE_RATE);

        if (loopEndSamples > loopStartSamples) {
            if (currentPlayhead >= loopEndSamples) {
                // Wrap to loop start
                uint64_t loopLength = loopEndSamples - loopStartSamples;
                effectivePlayhead = loopStartSamples + ((currentPlayhead - loopEndSamples) % loopLength);
                wrapped = true;
            } else if (currentPlayhead < loopStartSamples) {
                // Before loop start - clamp to loop start
                effectivePlayhead = loopStartSamples;
                wrapped = true;
            }
        }
    }

    if (wrapped) {
        _playheadSamples.store(effectivePlayhead, std::memory_order_release);
    }

    // Update clip scheduler playback state for the start of this block
    _clipScheduler->updatePlayback(effectivePlayhead);

    // Update automation current values once per block (for efficiency)
    _automationService->updateCurrentValues(effectivePlayhead);

    // Process audio block
    // For each frame in the block
    for (size_t frame = 0; frame < numFrames; ++frame) {
        uint64_t framePlayhead = effectivePlayhead + frame;

        // Handle loop wrapping within the block
        if (transport.loopEnabled && transport.loopRegion.has_value()) {
            const auto& loop = transport.loopRegion.value();
            uint64_t loopStartSamples = static_cast<uint64_t>(loop.startSeconds * SAMPLE_RATE);
            uint64_t loopEndSamples = static_cast<uint64_t>(loop.endSeconds * SAMPLE_RATE);
            if (loopEndSamples > loopStartSamples && framePlayhead >= loopEndSamples) {
                uint64_t loopLength = loopEndSamples - loopStartSamples;
                framePlayhead = loopStartSamples + ((framePlayhead - loopEndSamples) % loopLength);
            }
        }

        // For each output channel, mix all active clips
        for (int ch = 0; ch < numChannels; ++ch) {
            float channelSample = 0.0f;

            // TODO: In a real implementation, we would:
            // 1. Maintain a list of channels from the schedule
            // 2. For each channel, get active clips from ClipScheduler
            // 3. Read audio data from clip sources (audio buffers)
            // 4. Mix all active clips for this channel
            // 5. Apply clip-level gain (from ScheduledClip.gainDb)
            // 6. Apply automation (volume automation for this channel)
            // 7. Apply mixer gain (from MixerService)

            // For now, generate a simple test tone if there are any active clips
            // This demonstrates the integration but is not production-ready
            // In production, we would read actual audio data from clip sources

            // Check if there are any active clips (simplified - check first channel only)
            // In real implementation, we'd track all channels and process each separately
            std::string testChannelId = "channel-0"; // Placeholder - would come from schedule
            auto activeClips = _clipScheduler->getActiveClips(testChannelId, framePlayhead);

            if (!activeClips.empty()) {
                // Generate test tone (440 Hz sine wave) as placeholder for actual audio
                float time = static_cast<float>(framePlayhead) / static_cast<float>(SAMPLE_RATE);
                float frequency = 440.0f; // A4
                channelSample = 0.1f * std::sin(2.0f * M_PI * frequency * time);

                // Apply clip-level gain (from first active clip as example)
                // In real implementation, we'd mix all clips and apply each clip's gain
                if (!activeClips.empty()) {
                    float clipGainDb = activeClips[0]->gainDb.load(std::memory_order_acquire);
                    // Convert dB to linear gain
                    float clipGainLinear = (clipGainDb == 0.0f) ? 1.0f : std::pow(10.0f, clipGainDb / 20.0f);
                    channelSample *= clipGainLinear;
                }
            }

            // Apply automation (volume automation for channel)
            float automationGain = _automationService->evaluateAt(testChannelId, "gain", framePlayhead);
            channelSample *= automationGain;

            // Apply mixer gain
            float mixerGain = _mixerService->getEffectiveGain(testChannelId);
            channelSample *= mixerGain;

            // Apply panning (equal-power panning for stereo)
            // Pan: -1.0 = Left, 0.0 = Centre, +1.0 = Right
            float pan = 0.0f;
            auto* mixerState = _mixerService->getChannelState(testChannelId);
            if (mixerState) {
                pan = mixerState->pan.load(std::memory_order_acquire);
            }

            // Equal-power panning: left = cos((pan + 1) * PI/4), right = sin((pan + 1) * PI/4)
            // This ensures constant power across the stereo field
            float panAngle = (pan + 1.0f) * (M_PI / 4.0f);
            float leftGain = std::cos(panAngle);
            float rightGain = std::sin(panAngle);

            // Write to buffer with panning applied
            if (numChannels >= 2) {
                // Stereo output: apply panning
                buffer[frame * numChannels + 0] += channelSample * leftGain;  // Left
                buffer[frame * numChannels + 1] += channelSample * rightGain; // Right
            } else {
                // Mono output: write same sample to all channels
                for (int c = 0; c < numChannels; ++c) {
                    buffer[frame * numChannels + c] += channelSample;
                }
            }
        }
    }

    // Advance playhead
    uint64_t newPlayhead = effectivePlayhead + numFrames;

    // Handle loop wrapping at block boundary
    if (transport.loopEnabled && transport.loopRegion.has_value()) {
        const auto& loop = transport.loopRegion.value();
        uint64_t loopStartSamples = static_cast<uint64_t>(loop.startSeconds * SAMPLE_RATE);
        uint64_t loopEndSamples = static_cast<uint64_t>(loop.endSeconds * SAMPLE_RATE);
        if (loopEndSamples > loopStartSamples && newPlayhead >= loopEndSamples) {
            uint64_t loopLength = loopEndSamples - loopStartSamples;
            newPlayhead = loopStartSamples + ((newPlayhead - loopEndSamples) % loopLength);
        }
    }

    _playheadSamples.store(newPlayhead, std::memory_order_release);
}

