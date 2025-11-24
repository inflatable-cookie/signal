#include "core/EngineHost.hpp"
#include "core/AudioThread.hpp"
#include "backend/AudioBackend.hpp"
#include "backend/MiniaudioBackend.hpp"
#include "backend/AudioBackendConfig.hpp"
#include "core/MeteringService.hpp"
#include "core/MixerService.hpp"
#include "core/AutomationService.hpp"
#include "core/StreamScheduler.hpp"
#include "core/GraphEngine.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "core/AudioAssetSource.hpp"
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
    _streamScheduler = std::make_unique<StreamScheduler>();
    _graphEngine = std::make_unique<GraphEngine>();
    _audioAssetSource = std::make_unique<StubAudioAssetSource>(); // Phase 3: Use stub for now

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
    _streamScheduler->clearSchedule();
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

StreamScheduler& EngineHost::streamScheduler() {
    return *_streamScheduler;
}

const StreamScheduler& EngineHost::streamScheduler() const {
    return *_streamScheduler;
}

GraphEngine& EngineHost::graphEngine() {
    return *_graphEngine;
}

const GraphEngine& EngineHost::graphEngine() const {
    return *_graphEngine;
}

void EngineHost::loadGraphSnapshot(const GraphSnapshot& snapshot) {
    _graphEngine->loadGraphSnapshot(snapshot);
    // Mark that prepareEngine should be called before next render
    // For Phase 1, we'll call prepareEngine explicitly when needed
}

void EngineHost::prepareEngine(int sampleRate, int maxBlockSize) {
    _graphEngine->prepareGraph(sampleRate, maxBlockSize);
    // TODO: Also prepare plugins, allocate buffers, etc. in future phases
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

    // Update automation current values once per block (for efficiency)
    _automationService->updateCurrentValues(effectivePlayhead);

    // TODO: Process audio via node graph (future implementation)
    // The new architecture processes streams via node graph, not clips/channels:
    // 1. Get active audio segments per stream from StreamScheduler
    // 2. Load audio data from assets (per stream)
    // 3. Feed streams into lane nodes (from GraphSnapshot)
    // 4. Process through node graph (lane → fx → mixer → output)
    // 5. Apply automation per node/parameter (not per channel)
    // 6. Render final output
    //
    // Current implementation: Output silence until node graph is implemented
    // Legacy clip/channel-based processing has been removed to align with new architecture

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

    // Process graph (Phase 3: real audio streaming, send/receive routing)
    _graphEngine->processGraph(ctx, _streamScheduler.get(), _audioAssetSource.get());

    // Copy device node output to EngineHost output buffer
    // TODO: Support multiple device nodes in future (e.g., different output devices, cue mixes)
    const auto& executionOrder = _graphEngine->getExecutionOrder();
    GraphNode* deviceNode = nullptr;
    for (GraphNode* node : executionOrder) {
        if (node && node->getKind() == NodeKind::Device) {
            deviceNode = node;
            break; // For now, use first device node (previously assumed single master)
        }
    }

    if (deviceNode) {
        // Copy audio from device node to output bus
        const int numChannels = std::min(deviceNode->io.audioOut.numChannels(), output.numChannels());
        const int numFrames = std::min(deviceNode->io.audioOut.numFrames(), output.numFrames());

        for (int ch = 0; ch < numChannels; ++ch) {
            const float* src = deviceNode->io.audioOut.getChannelData(ch);
            for (int frame = 0; frame < numFrames; ++frame) {
                output.setSample(frame, ch, src[frame]);
            }
        }
    }

    // TODO: Phase 2 - Attach Stream Inputs & Minimal Audio/MIDI Flow:
    // - Get active audio segments per stream from StreamScheduler
    // - Load audio data from assets (per streamId)
    // - Feed streams into lane nodes using getStreamBindings()
    // - Process through node graph with real audio/MIDI buffers
    // - Apply automation per node/parameter
    // - Apply mixer gain/mute/solo per channel (channels are processing paths, not tracks)
    // - Loop handling
    // - Metering
    //
    // Architecture: Signal processes streams via node graph, not clips/channels.
    // Pulse compiles Tracks → Lanes → Streams and sends stream-based schedules.

    // For now, produce silence (nodes are processed but don't generate audio yet)
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

