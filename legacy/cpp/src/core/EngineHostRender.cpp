#include "core/EngineHost.hpp"
#include "core/MeteringService.hpp"
#include "core/AutomationService.hpp"
#include "core/StreamScheduler.hpp"
#include "core/GraphEngine.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "core/RecordingCapture.hpp"
#include "core/GraphNodes.hpp"
#include "logging/Logging.hpp"
#include <cmath>
#include <cstring>
#include <sstream>

void EngineHost::renderBlock(
    EngineRenderContext& ctx,
    AudioBus& input,
    AudioBus& output
) {
    // Real-time safety: No allocations, locks, or I/O in this function

    // Read transport state snapshot once (lock-free)
    // Pointer remains valid for the entire renderBlock (previous snapshot kept alive)
    const TransportState* transport = getTransportSnapshot();

    // Update context with current playhead (needed for hasWorkToDo check)
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

    // Idle fast-path: skip processing if truly idle
    if (!hasWorkToDo(ctx)) {
        // Clear output and return early - no schedule, no tails, no live inputs
        output.clear();
        return;
    }

    // Update context with current playhead (already set above, but keeping for clarity)

    // Clear output buffer
    output.clear();

    // Step 1: Begin automation block evaluation (pre-computes all parameter values)
    _automationService->beginBlock(ctx.playheadSamples, ctx.blockSize, ctx.sampleRate);

    // Step 2: Apply automation values to nodes and services
    // Real-time safe: no allocations, just value lookups and assignments
    const auto& executionOrder = _graphEngine->getExecutionOrder();
    for (GraphNode* node : executionOrder) {
        if (!node) continue;

        std::string nodeId = node->getId();

        // Apply fader automation (gain/spatial.balance)
        if (node->getKind() == NodeKind::Fader) {
            auto* faderNode = dynamic_cast<FaderNode*>(node);
            if (faderNode) {
                // Use node ID as automation target for fader parameters
                const std::string& targetId = node->getId();

                float gain = _automationService->getParameterValue(targetId, "gain");
                float balance = _automationService->getParameterValue(targetId, "spatial.balance");

                faderNode->setGain(gain);
                faderNode->setPan(balance);
            }
        }

        // Apply send level automation
        if (node->getKind() == NodeKind::Send) {
            auto* sendNode = dynamic_cast<SendNode*>(node);
            if (sendNode) {
                const std::string& targetId = node->getId();
                float sendLevel = _automationService->getParameterValue(targetId, "send-level");
                sendNode->setSendLevel(sendLevel);
            }
        }

        // Plugin parameter automation is applied via applyParameterChanges mechanism
        // AutomationService values are pushed into that queue from control thread
        // This keeps plugin parameter updates synchronized with other parameter changes
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
            if (
                node->getKind() == NodeKind::MidiFx ||
                node->getKind() == NodeKind::Instrument ||
                node->getKind() == NodeKind::AudioFx
            ) {
                auto* pluginNode = dynamic_cast<PluginNode*>(node);
                if (pluginNode) {
                    plugin = pluginNode->getPlugin();
                }
            }

            if (plugin) {
                plugin->setParameterValue(change.paramId, change.normalisedValue);
            }
        }
    }

    // Step 2: Clear all node buffers (prepares for Source/Input Pass and processing)
    // This is done here so buffers are cleared before the Source/Input Pass populates outputs
    for (GraphNode* node : executionOrder) {
        if (node) {
            node->io.audioIn.clear();
            node->io.midiIn.clear();
            node->io.audioOut.clear();
            node->io.midiOut.clear();
        }
    }

    // Step 3: Source/Input Pass - inject schedule data and hardware input
    // This unified pass populates all source and input node outputs before processing
    // Real-time safe: no allocations, no locks, no logging
    std::vector<MidiMessage> hardwareMidiInput; // TODO: Get from MIDI backend when implemented
    _graphEngine->runSourceInputPass(
        ctx,
        _streamScheduler.get(),
        _audioAssetSource.get(),
        input.data(),
        input.numChannels(),
        input.numFrames(),
        hardwareMidiInput
    );

    // Phase 7: Capture from input nodes if recording is active
    if (_recordingSession->isRecording()) {
        uint64_t blockStartSamples = ctx.playheadSamples;

        for (GraphNode* node : executionOrder) {
            if (node && node->getKind() == NodeKind::HardwareAudioInput) {
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

                            // Convert deinterleaved AudioBuffer to interleaved format
                            int numFrames = audioOut.numFrames();
                            chunk.interleaved.resize(chunk.numChannels * numFrames);
                            audioOut.copyToInterleaved(chunk.interleaved.data(), chunk.numChannels, numFrames);

                            _recordingSession->captureAudioChunk(chunk);
                        }
                    }
                }
            } else if (node && node->getKind() == NodeKind::HardwareMidiInput) {
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

    // Step 4: Process graph (routing, plugin processing)
    // Note: Source/Input Pass was already called in Step 3, so nodes are ready to process
    _graphEngine->processGraph(ctx);

    // Step 3: Find hardware audio output node and mix to host output bus
    GraphNode* outputNode = nullptr;
    const std::string* preferredOutputNodeId = _activeOutputNodeId.load(std::memory_order_acquire);
    const auto& executionOrderAfterGraph = _graphEngine->getExecutionOrder();

    for (GraphNode* node : executionOrderAfterGraph) {
        if (node && node->getKind() == NodeKind::HardwareAudioOutput) {
            if (
                preferredOutputNodeId &&
                !preferredOutputNodeId->empty() &&
                node->getId() == *preferredOutputNodeId
            ) {
                outputNode = node;
                break;
            }

            if (!outputNode) {
                outputNode = node;
            }
        }
    }

    if (outputNode) {
        const std::string* outputMixId = _activeOutputMixId.load(std::memory_order_acquire);
        const std::string& mixId = outputMixId ? *outputMixId : outputNode->getId();

        // Step 4: Mix output node into host output bus.
        //
        // Note: Mute/gain/spatial.balance are owned by nodes in the graph (e.g. FaderNode),
        // so EngineHost can do a straightforward copy/format conversion here.
        const int numChannels = output.numChannels();
        const int numFrames = std::min(output.numFrames(), outputNode->io.audioOut.numFrames());

        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels; ++ch) {
                const float* inChannel = outputNode->io.audioOut.getChannelData(ch);

                if (!inChannel) {
                    output.setSample(frame, ch, 0.0f);
                    continue;
                }

                output.setSample(frame, ch, inChannel[frame]);
            }
        }

        if (numFrames < output.numFrames()) {
            for (int frame = numFrames; frame < output.numFrames(); ++frame) {
                for (int ch = 0; ch < numChannels; ++ch) {
                    output.setSample(frame, ch, 0.0f);
                }
            }
        }

        // Step 5: Capture metering levels from final mixed output
        // Real-time safe: submitSampleBlock is lock-free (uses shared_lock for map lookup only)
        _meteringService->submitSampleBlock(
            mixId,
            output.data(),
            output.numChannels(),
            output.numFrames()
        );

        // Step 6: Capture final output for recording (if recording is active)
        if (_recordingSession->isRecording()) {
            _recordingSession->captureFinalOutput(
                output,
                ctx.playheadSamples,
                mixId,
                static_cast<int>(ctx.sampleRate)
            );
        }
    } else {
        // No hardware output node - output will be silence
        output.clear();
    }

    // Diagnostic: Check output level
    float maxOutput = 0.0f;
    bool hasOutput = false;
    const int numChannels = output.numChannels();
    const int numFrames = output.numFrames();
    for (int frame = 0; frame < numFrames; ++frame) {
        for (int ch = 0; ch < numChannels; ++ch) {
            float absSample = std::abs(output.sample(frame, ch));
            if (absSample > maxOutput) {
                maxOutput = absSample;
            }
            if (absSample > 0.0001f) {
                hasOutput = true;
            }
        }
    }

    // Diagnostic logging: Periodic status (every ~1 second when playing, less frequent when stopped)
    uint64_t blockCount = _renderBlockCount.fetch_add(1, std::memory_order_acq_rel) + 1;
    uint64_t lastLog = _lastDebugLogBlock.load(std::memory_order_acquire);

    // Only log when playing, or occasionally when stopped (every ~10 seconds) to confirm engine is alive
    bool shouldLog = false;
    if (ctx.isPlaying) {
        // Log every ~1 second when playing
        shouldLog = (blockCount - lastLog >= DEBUG_LOG_INTERVAL_BLOCKS);
    } else {
        // Log every ~10 seconds when stopped (much less frequent)
        static constexpr uint32_t STOPPED_LOG_INTERVAL_BLOCKS = DEBUG_LOG_INTERVAL_BLOCKS * 10;
        shouldLog = (blockCount - lastLog >= STOPPED_LOG_INTERVAL_BLOCKS);
    }

    if (shouldLog) {
        _lastDebugLogBlock.store(blockCount, std::memory_order_release);

        // Log diagnostic info (non-real-time, but throttled)
        bool graphLoaded = _graphEngine->hasGraph();
        bool scheduleLoaded = _streamScheduler->hasSchedule();
        int activeStreamCount = _streamScheduler->getActiveStreamCount();
        // TODO: Future tail handling - check if any nodes have active tail (hasTailCurrently())
        //   Continue rendering even after schedule ends if tail is active

        // Format diagnostic message
        std::ostringstream diagMsg;
        diagMsg << "Block " << blockCount
                << ": playing=" << (ctx.isPlaying ? "yes" : "no")
                << ", playhead=" << ctx.playheadSamples
                << ", graph=" << (graphLoaded ? "yes" : "no")
                << ", schedule=" << (scheduleLoaded ? "yes" : "no")
                << ", activeStreams=" << activeStreamCount
                << ", maxOutput=" << maxOutput;
        LOG_DEBUG({"EngineHost", "Render"}, diagMsg.str());
    }

    // Diagnostic: Track consecutive silence
    if (hasOutput) {
        _consecutiveSilenceBlocks.store(0, std::memory_order_release);
    } else if (ctx.isPlaying) {
        uint64_t silenceCount = _consecutiveSilenceBlocks.fetch_add(1, std::memory_order_acq_rel) + 1;
        // Log warning if we've had silence for a while (e.g., 1 second = ~86 blocks at 44.1kHz/512)
        if (silenceCount == 86) {
            LOG_WARN({"EngineHost", "Render"}, "⚠ WARNING: Output still silence after 1 second of playback");
        }
    }

    // TODO: Phase 2 - Attach Stream Inputs & Minimal Audio/MIDI Flow:
    // - Get active audio segments per stream from StreamScheduler
    // - Load audio data from assets (per streamId)
    // - Feed streams into lane nodes using getStreamBindings()
    // - Process through node graph with real audio/MIDI buffers
    // - Apply automation per node/parameter
    // - Apply per-node mix controls (gain/spatial.balance/mute) via node parameters
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

    // Update playhead for next block (only when playing)
    // When stopped, playhead is explicitly set by seek/play commands
    if (ctx.isPlaying) {
        uint64_t newPlayhead = ctx.playheadSamples + output.numFrames();
        _playheadSamples.store(newPlayhead, std::memory_order_release);
    }
}
