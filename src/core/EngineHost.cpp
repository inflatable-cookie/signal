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
#include "core/PluginHost.hpp"
#include "core/GraphNodes.hpp"
#include "core/AutomationData.hpp"
#include "core/RecordingCapture.hpp"
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
    _pluginHost = std::make_unique<PluginHost>(); // Phase 4: Plugin host
    _recordingSession = std::make_unique<RecordingSession>(); // Phase 7: Recording session
    _parameterChangesPending.store(false, std::memory_order_release);

    // Initialize transport state with default values
    _transportState = std::make_shared<TransportState>();
    _activeTransport.store(_transportState.get(), std::memory_order_release);

    // Initialize automation data with empty snapshot
    _automationData = std::make_shared<AutomationData>(AutomationData::empty());
    _activeAutomation.store(_automationData.get(), std::memory_order_release);

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

const AutomationData* EngineHost::getAutomationSnapshot() const {
    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous snapshot kept alive in _previousAutomation)
    return _activeAutomation.load(std::memory_order_acquire);
}

RecordingSession& EngineHost::recordingSession() {
    return *_recordingSession;
}

const RecordingSession& EngineHost::recordingSession() const {
    return *_recordingSession;
}

void EngineHost::loadAutomationSnapshot(const AutomationData& snapshot) {
    // Create a new snapshot from provided data (copy constructor)
    auto newSnapshot = std::make_shared<AutomationData>(snapshot);

    // Keep previous snapshot alive until next swap (ensures audio thread safety)
    _previousAutomation = _automationData;

    // Atomically swap pointer (old snapshot kept alive in _previousAutomation)
    _activeAutomation.store(newSnapshot.get(), std::memory_order_release);

    // Update _automationData to point to new snapshot
    _automationData = newSnapshot;

    std::cout << "[EngineHost] Loaded automation snapshot: " << snapshot.events.size() << " events" << std::endl;
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

PluginHost* EngineHost::pluginHost() {
    return _pluginHost.get();
}

const PluginHost* EngineHost::pluginHost() const {
    return _pluginHost.get();
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

void EngineHost::applyParameterChanges(const std::vector<ParameterChange>& changes) {
    // Called on control thread - queue changes for audio thread
    _pendingParameterChanges.insert(_pendingParameterChanges.end(), changes.begin(), changes.end());
    _parameterChangesPending.store(true, std::memory_order_release);
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

    // Handle loop wrapping (use sample-based loop region if available for efficiency)
    uint64_t effectivePlayhead = currentPlayhead;
    bool wrapped = false;
    if (transport && transport->loopEnabled) {
        uint64_t loopStartSamples = 0;
        uint64_t loopEndSamples = 0;
        bool hasLoop = false;

        // Prefer sample-based loop region (more efficient)
        if (transport->loopRegionSamples.has_value()) {
            const auto& loop = transport->loopRegionSamples.value();
            loopStartSamples = loop.startSamples;
            loopEndSamples = loop.endSamples;
            hasLoop = true;
        } else if (transport->loopRegion.has_value()) {
            // Fallback to seconds-based (convert to samples)
            const auto& loop = transport->loopRegion.value();
            loopStartSamples = static_cast<uint64_t>(loop.startSeconds * SAMPLE_RATE);
            loopEndSamples = static_cast<uint64_t>(loop.endSeconds * SAMPLE_RATE);
            hasLoop = true;
        }

        if (hasLoop && loopEndSamples > loopStartSamples) {
            if (currentPlayhead >= loopEndSamples) {
                // Wrap to loop start
                uint64_t loopLength = loopEndSamples - loopStartSamples;
                effectivePlayhead = loopStartSamples + ((currentPlayhead - loopStartSamples) % loopLength);
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

    // Handle loop wrapping at block boundary (use sample-based loop region if available)
    if (transport && transport->loopEnabled) {
        uint64_t loopStartSamples = 0;
        uint64_t loopEndSamples = 0;
        bool hasLoop = false;

        // Prefer sample-based loop region (more efficient)
        if (transport->loopRegionSamples.has_value()) {
            const auto& loop = transport->loopRegionSamples.value();
            loopStartSamples = loop.startSamples;
            loopEndSamples = loop.endSamples;
            hasLoop = true;
        } else if (transport->loopRegion.has_value()) {
            // Fallback to seconds-based (convert to samples)
            const auto& loop = transport->loopRegion.value();
            loopStartSamples = static_cast<uint64_t>(loop.startSeconds * SAMPLE_RATE);
            loopEndSamples = static_cast<uint64_t>(loop.endSeconds * SAMPLE_RATE);
            hasLoop = true;
        }

        if (hasLoop && loopEndSamples > loopStartSamples && newPlayhead >= loopEndSamples) {
            uint64_t loopLength = loopEndSamples - loopStartSamples;
            newPlayhead = loopStartSamples + ((newPlayhead - loopStartSamples) % loopLength);
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

    // Update transport/tempo info in context (Phase 8)
    if (transport) {
        ctx.tempo = transport->tempo;
        ctx.isPlaying = transport->isPlaying;
        ctx.loopEnabled = transport->loopEnabled;

        // Convert loop region from samples/seconds to beats if available
        // For Phase 8, use simple tempo conversion: beats = seconds * tempo / 60
        if (transport->loopEnabled && transport->loopRegion.has_value()) {
            const auto& loop = transport->loopRegion.value();
            ctx.loopStartBeats = (loop.startSeconds * transport->tempo) / 60.0;
            ctx.loopEndBeats = (loop.endSeconds * transport->tempo) / 60.0;
        } else {
            ctx.loopStartBeats = 0.0;
            ctx.loopEndBeats = 0.0;
        }
    } else {
        // Default values if transport not available
        ctx.tempo = 120.0;
        ctx.isPlaying = false;
        ctx.loopEnabled = false;
        ctx.loopStartBeats = 0.0;
        ctx.loopEndBeats = 0.0;
    }

    // Clear output buffer
    output.clear();

    // Phase 6: Apply automation events for this block
    const AutomationData* automation = getAutomationSnapshot();
    if (automation && !automation->events.empty()) {
        uint64_t blockStartSamples = ctx.playheadSamples;
        uint64_t blockEndSamples = blockStartSamples + static_cast<uint64_t>(output.numFrames());

        // Find automation events in this block range
        // For Phase 6, we use step interpolation: use the last event at or before block start
        // Future: support linear interpolation within blocks

        // Build a map of (nodeId, paramId) -> value for this block
        std::unordered_map<std::string, float> automationValues; // Key: "nodeId:paramId"

        // Find the last event at or before block start for each (nodeId, paramId) pair
        for (const auto& event : automation->events) {
            if (event.timeSamples > blockEndSamples) {
                // Past this block, stop searching (events are sorted)
                break;
            }

            if (event.timeSamples <= blockStartSamples) {
                // Event is at or before block start - use it
                std::string key = event.nodeId + ":" + event.paramId;
                automationValues[key] = event.valueNorm;
            }
        }

        // Apply automation values to nodes
        for (const auto& [key, valueNorm] : automationValues) {
            // Parse key: "nodeId:paramId"
            size_t colonPos = key.find(':');
            if (colonPos == std::string::npos) continue;

            std::string nodeId = key.substr(0, colonPos);
            std::string paramId = key.substr(colonPos + 1);

            GraphNode* node = _graphEngine->findNode(nodeId);
            if (!node) continue;

            // Route to appropriate node type
            if (node->getKind() == NodeKind::MixerChannel) {
                auto* mixer = dynamic_cast<MixerChannelNode*>(node);
                if (mixer) {
                    if (paramId == "gain") {
                        mixer->setGain(valueNorm);
                    } else if (paramId == "pan") {
                        // Convert normalised [0,1] to pan [-1,1]
                        float pan = (valueNorm * 2.0f) - 1.0f;
                        mixer->setPan(pan);
                    }
                }
            } else if (node->getKind() == NodeKind::Send) {
                auto* send = dynamic_cast<SendNode*>(node);
                if (send) {
                    // Handle "send-level" or "send-level:<busId>" format
                    if (paramId == "send-level" || paramId.find("send-level:") == 0) {
                        send->setSendLevel(valueNorm);
                    }
                }
            } else if (node->getKind() == NodeKind::MidiFx ||
                       node->getKind() == NodeKind::Instrument ||
                       node->getKind() == NodeKind::AudioFx) {
                // Plugin nodes: use parameter change mechanism
                // For Phase 6, apply directly (future: queue for sample-accurate timing)
                PluginInstance* plugin = nullptr;
                if (node->getKind() == NodeKind::MidiFx) {
                    auto* midiFx = dynamic_cast<MidiFxNode*>(node);
                    if (midiFx) plugin = midiFx->getPlugin();
                } else if (node->getKind() == NodeKind::Instrument) {
                    auto* instrument = dynamic_cast<InstrumentNode*>(node);
                    if (instrument) plugin = instrument->getPlugin();
                } else if (node->getKind() == NodeKind::AudioFx) {
                    auto* audioFx = dynamic_cast<AudioFxNode*>(node);
                    if (audioFx) plugin = audioFx->getPlugin();
                }

                if (plugin) {
                    plugin->setParameterValue(paramId, valueNorm);
                }
            }
        }
    }

    // Phase 4: Apply pending parameter changes (lock-free swap)
    if (_parameterChangesPending.load(std::memory_order_acquire)) {
        // Swap pending changes to active (control thread writes, audio thread reads)
        _activeParameterChanges.clear();
        _activeParameterChanges.swap(_pendingParameterChanges);
        _parameterChangesPending.store(false, std::memory_order_release);

        // Apply parameter changes to plugin nodes
        for (const auto& change : _activeParameterChanges) {
            GraphNode* node = _graphEngine->findNode(change.nodeId);
            if (!node) {
                continue;
            }

            // Check if node has a plugin
            PluginInstance* plugin = nullptr;
            if (node->getKind() == NodeKind::MidiFx) {
                auto* midiFx = dynamic_cast<MidiFxNode*>(node);
                if (midiFx) plugin = midiFx->getPlugin();
            } else if (node->getKind() == NodeKind::Instrument) {
                auto* instrument = dynamic_cast<InstrumentNode*>(node);
                if (instrument) plugin = instrument->getPlugin();
            } else if (node->getKind() == NodeKind::AudioFx) {
                auto* audioFx = dynamic_cast<AudioFxNode*>(node);
                if (audioFx) plugin = audioFx->getPlugin();
            }

            if (plugin) {
                plugin->setParameterValue(change.paramId, change.normalisedValue);
            }
        }
    }

    // Phase 7: Inject input data from backend into input nodes
    const auto& executionOrder = _graphEngine->getExecutionOrder();
    for (GraphNode* node : executionOrder) {
        if (node && node->getKind() == NodeKind::AudioInput) {
            auto* inputNode = dynamic_cast<AudioInputNode*>(node);
            if (inputNode) {
                // Extract channel from interleaved input buffer
                int channelIndex = inputNode->getInputChannelIndex();
                if (channelIndex < input.numChannels()) {
                    inputNode->injectInputAudio(
                        input.data(),
                        input.numChannels(),
                        input.numFrames(),
                        channelIndex
                    );
                }
            }
        } else if (node && node->getKind() == NodeKind::MidiInput) {
            // Phase 7: MIDI input injection (stub for now - no MIDI backend yet)
            // TODO: Inject MIDI from backend when MIDI backend is implemented
            auto* midiInputNode = dynamic_cast<MidiInputNode*>(node);
            if (midiInputNode) {
                // For Phase 7, MIDI input is empty (no backend yet)
                std::vector<MidiMessage> emptyMidi;
                midiInputNode->injectInputMidi(emptyMidi);
            }
        }
    }

    // Phase 7: Capture from input nodes if recording is active
    if (_recordingSession->isRecording()) {
        uint64_t blockStartSamples = ctx.playheadSamples;

        for (GraphNode* node : executionOrder) {
            if (node && node->getKind() == NodeKind::AudioInput) {
                auto* inputNode = dynamic_cast<AudioInputNode*>(node);
                if (inputNode) {
                    std::string laneId = _recordingSession->getTargetLaneForInput(inputNode->getId());
                    if (!laneId.empty() && _recordingSession->isLaneArmed(laneId)) {
                        // Capture audio from this input node
                        const auto& audioOut = inputNode->io.audioOut;
                        if (audioOut.numChannels() > 0 && audioOut.numFrames() > 0) {
                            RecordedAudioChunk chunk;
                            chunk.laneId = laneId;
                            chunk.numChannels = audioOut.numChannels();
                            chunk.sampleRate = static_cast<int>(ctx.sampleRate);
                            chunk.startSample = blockStartSamples;
                            chunk.provisionalAssetId = "temp-" + inputNode->getId() + "-" + std::to_string(blockStartSamples);

                            // Convert deinterleaved to interleaved
                            int numFrames = audioOut.numFrames();
                            chunk.interleaved.resize(chunk.numChannels * numFrames);
                            for (int frame = 0; frame < numFrames; ++frame) {
                                for (int ch = 0; ch < chunk.numChannels; ++ch) {
                                    chunk.interleaved[frame * chunk.numChannels + ch] = audioOut.getSample(ch, frame);
                                }
                            }

                            _recordingSession->captureAudioChunk(chunk);
                        }
                    }
                }
            } else if (node && node->getKind() == NodeKind::MidiInput) {
                auto* midiInputNode = dynamic_cast<MidiInputNode*>(node);
                if (midiInputNode) {
                    std::string laneId = _recordingSession->getTargetLaneForInput(midiInputNode->getId());
                    if (!laneId.empty() && _recordingSession->isLaneArmed(laneId)) {
                        // Capture MIDI from this input node
                        const auto& midiOut = midiInputNode->io.midiOut;
                        if (midiOut.size() > 0) {
                            RecordedMidiChunk chunk;
                            chunk.laneId = laneId;
                            chunk.startSample = blockStartSamples;

                            // Convert MidiBuffer to RecordedMidiEvent
                            const auto& messages = midiOut.getMessages();
                            for (const auto& msg : messages) {
                                RecordedMidiEvent event;
                                event.timeSamples = blockStartSamples + msg.sampleOffset;
                                event.status = msg.status;
                                event.data1 = msg.data1;
                                event.data2 = msg.data2;
                                event.channel = msg.channel;
                                chunk.events.push_back(event);
                            }

                            _recordingSession->captureMidiChunk(chunk);
                        }
                    }
                }
            }
        }
    }

    // Process graph (Phase 4: real audio streaming, send/receive routing, plugin processing)
    _graphEngine->processGraph(ctx, _streamScheduler.get(), _audioAssetSource.get());

    // Copy device node output to EngineHost output buffer
    // TODO: Support multiple device nodes in future (e.g., different output devices, cue mixes)
    // Reuse executionOrder from above
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

