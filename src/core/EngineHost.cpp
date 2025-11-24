#include "core/EngineHost.hpp"
#include "core/AudioThread.hpp"
#include "backend/AudioBackend.hpp"
#include "backend/MiniaudioBackend.hpp"
#include "backend/AudioBackendConfig.hpp"
#include "core/MeteringService.hpp"
#include "core/MixerService.hpp"
#include "core/AutomationService.hpp"
#include "core/ClipScheduler.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
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

    // Initialize transport state with default values
    _transportState = std::make_shared<TransportState>();
    _activeTransport.store(_transportState.get(), std::memory_order_release);

    setupAudioCallback();  // Legacy - for backward compatibility
    setupAudioBackend();   // New backend-based approach
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

    // Start audio backend (preferred method)
    if (_audioBackend) {
        if (!_audioBackend->start()) {
            setError("Failed to start audio backend");
            return;
        }
    } else {
        // Fallback to legacy AudioThread
        _audioThread->setMeteringService(_meteringService.get());
        _audioThread->start();
    }

    // After audio starts successfully, transition to running
    _state = State::Running;
    std::cout << "[EngineHost] Started" << std::endl;
}

void EngineHost::stop() {
    if (_state == State::Stopped) {
        std::cout << "[EngineHost] Already stopped" << std::endl;
        return;
    }

    _state = State::Stopped;

    if (_audioBackend) {
        _audioBackend->stop();
    } else {
        _audioThread->stop();
    }

    std::cout << "[EngineHost] Stopped" << std::endl;
}

void EngineHost::reset() {
    stop();
    clearError();

    // Reset transport state (create new snapshot)
    _transportState = std::make_shared<TransportState>();
    _activeTransport.store(_transportState.get(), std::memory_order_release);
    _previousTransport.reset();

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
    // Return mutable reference for control thread updates
    // Caller should call commitTransportUpdate() after making changes
    return *_transportState;
}

const TransportState& EngineHost::transport() const {
    return *_transportState;
}

const TransportState* EngineHost::getTransportSnapshot() const {
    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous snapshot kept alive in _previousTransport)
    return _activeTransport.load(std::memory_order_acquire);
}

// Helper method to commit transport updates (called after modifying transport())
void EngineHost::commitTransportUpdate() {
    // Create a new snapshot from current state (copy constructor)
    // At this point, _transportState points to the object that was just modified
    auto newSnapshot = std::make_shared<TransportState>(*_transportState);

    // Keep previous snapshot alive until next swap (ensures audio thread safety)
    _previousTransport = _transportState;

    // Atomically swap pointer (old snapshot kept alive in _previousTransport)
    _activeTransport.store(newSnapshot.get(), std::memory_order_release);

    // Update our mutable state pointer (now points to the new snapshot)
    // This ensures future calls to transport() return the new snapshot
    _transportState = newSnapshot;
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
    if (_audioBackend) {
        return _audioBackend->getSampleRate();
    }
    return SAMPLE_RATE;
}

size_t EngineHost::getBlockSize() const {
    if (_audioBackend) {
        return static_cast<size_t>(_audioBackend->getBufferSize());
    }
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

void EngineHost::setupAudioBackend() {
    // Create MiniaudioBackend (placeholder implementation)
    _audioBackend = std::make_unique<MiniaudioBackend>();

    // Configure backend
    AudioBackendConfig config;
    config.preferredSampleRate = SAMPLE_RATE;
    config.preferredBufferSize = static_cast<int>(BLOCK_SIZE);
    config.numInputChannels = 0;   // No input for now
    config.numOutputChannels = 2;  // Stereo output

    if (!_audioBackend->initialise(config)) {
        std::cerr << "[EngineHost] Failed to initialise audio backend" << std::endl;
        _audioBackend.reset();
        return;
    }

    // Set render callback to call renderBlock
    _audioBackend->setRenderCallback([this](
        EngineRenderContext& ctx,
        AudioBus& input,
        AudioBus& output
    ) {
        this->renderBlock(ctx, input, output);
    });

    std::cout << "[EngineHost] Audio backend configured" << std::endl;
}

void EngineHost::audioCallback(float* buffer, size_t numFrames, int numChannels) {
    // Clear buffer
    std::memset(buffer, 0, numFrames * numChannels * sizeof(float));

    // Get current playhead position
    uint64_t currentPlayhead = _playheadSamples.load(std::memory_order_acquire);

    // Check transport state (lock-free read via snapshot)
    const TransportState* transport = getTransportSnapshot();
    if (!transport || !transport->isPlaying) {
        // Not playing - output silence, but still advance playhead for seek accuracy
        _playheadSamples.store(currentPlayhead + numFrames, std::memory_order_release);
        return;
    }

    // Handle loop wrapping
    uint64_t effectivePlayhead = currentPlayhead;
    bool wrapped = false;
    if (transport && transport->loopEnabled && transport->loopRegion.has_value()) {
        const auto& loop = transport->loopRegion.value();
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
        if (transport && transport->loopEnabled && transport->loopRegion.has_value()) {
            const auto& loop = transport->loopRegion.value();
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
            // Check for pan automation first (automation overrides static pan)
            float pan = _automationService->evaluateAt(testChannelId, "pan", framePlayhead);

            // If no pan automation exists, evaluateAt returns 0.0 (default centre pan)
            // We need to check if automation actually exists. For now, we'll use a simple approach:
            // If pan is exactly 0.0, it could be either "no automation" or "pan at centre".
            // To distinguish, we check if there's a pan curve. But that requires more API.
            // For now, use a heuristic: if pan is 0.0 and we have static pan that's not 0.0,
            // assume no automation and use static. Otherwise use automation value.
            // This is not perfect but works for most cases.
            auto* mixerState = _mixerService->getChannelState(testChannelId);
            if (mixerState && pan == 0.0f) {
                float staticPan = mixerState->pan.load(std::memory_order_acquire);
                // If static pan is not centre, and automation pan is centre,
                // it's likely no automation exists, so use static
                // This heuristic works: if user set static pan to non-zero,
                // and automation returns 0.0, it's probably no automation
                if (staticPan != 0.0f) {
                    pan = staticPan;
                }
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
    if (transport && transport->loopEnabled && transport->loopRegion.has_value()) {
        const auto& loop = transport->loopRegion.value();
        uint64_t loopStartSamples = static_cast<uint64_t>(loop.startSeconds * SAMPLE_RATE);
        uint64_t loopEndSamples = static_cast<uint64_t>(loop.endSeconds * SAMPLE_RATE);
        if (loopEndSamples > loopStartSamples && newPlayhead >= loopEndSamples) {
            uint64_t loopLength = loopEndSamples - loopStartSamples;
            newPlayhead = loopStartSamples + ((newPlayhead - loopEndSamples) % loopLength);
        }
    }

    _playheadSamples.store(newPlayhead, std::memory_order_release);
}

void EngineHost::renderBlock(
    EngineRenderContext& ctx,
    AudioBus& input,
    AudioBus& output
) {
    // Real-time safety: No allocations, locks, or I/O in this function

    // Read transport state snapshot once (lock-free)
    // Pointer remains valid for the entire renderBlock (previous snapshot kept alive)
    const TransportState* transport = getTransportSnapshot();

    // Update context with current playhead
    ctx.playheadSamples = _playheadSamples.load(std::memory_order_acquire);

    // Clear output buffer
    output.clear();

    // TODO (Phase B-E): Implement full audio pipeline:
    // - Schedule → clips (Phase B)
    // - Mixer gain/mute/solo (Phase C)
    // - Automation (volume & pan) (Phase D)
    // - Loop handling (Phase E)
    // - Metering (Phase E)

    // For now, produce silence or a simple test tone
    // Uncomment the test tone code below to verify audio output:
    /*
    const float testToneFreq = 440.0f; // A4
    const float amplitude = 0.1f;
    float* outData = output.data();
    if (outData && output.numChannels() > 0) {
        for (int frame = 0; frame < output.numFrames(); ++frame) {
            float time = static_cast<float>(ctx.playheadSamples + frame) / static_cast<float>(ctx.sampleRate);
            float sample = amplitude * std::sin(2.0f * M_PI * testToneFreq * time);
            for (int ch = 0; ch < output.numChannels(); ++ch) {
                output.setSample(frame, ch, sample);
            }
        }
    }
    */

    // Update playhead for next block
    uint64_t newPlayhead = ctx.playheadSamples + output.numFrames();
    _playheadSamples.store(newPlayhead, std::memory_order_release);
}

