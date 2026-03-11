#pragma once

/// GraphNodes - Specialized node subclasses
///
/// Thread: Audio thread (process), Control thread (prepare)
/// Ownership: Owned by GraphEngine
///
/// These are concrete node implementations for each node kind.
/// Phase 2 adds stream injection and pass-through processing.

#include "core/GraphNode.hpp"
#include "core/ScheduleData.hpp"
#include "core/StreamScheduler.hpp"
#include "core/AudioAssetSource.hpp"
#include "core/PluginInstance.hpp"
#include "core/PluginHost.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/NodeAudioConfig.hpp"
#include "logging/Logging.hpp"
#include <array>
#include <cstdint>
#include <string>
#include <sstream>
#include <vector>
#include <cmath>
#include <algorithm>
#include <optional>

// Forward declaration (full include would create circular dependency)
// HardwareAudioOutputNode needs EngineHost to query device channel count
class EngineHost;

/// MidiLaneNode - one per MIDI Lane
/// Injects MIDI events from stream into graph
class MidiLaneNode : public GraphNode {
public:
    MidiLaneNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& laneId = ""
    )
        : GraphNode(id, NodeKind::MidiLane, trackId, laneId)
        , _streamId("") // Set by GraphEngine during stream injection
    {
    }

    void setStreamId(const StreamId& streamId) {
        _streamId = streamId;
    }

    const StreamId& getStreamId() const noexcept {
        return _streamId;
    }

    void process(const NodeProcessContext& npc) override {
        // Phase 3: MIDI events are injected via injectMidiEvents() before process() is called
        // This node just passes through (or could apply lane-level processing in future)
        // For now, MIDI is already in midiOut from injection
    }

    /// Inject MIDI events into this lane node (called by GraphEngine)
    void injectMidiEvents(const std::vector<const MidiEventCompiled*>& events, uint64_t blockStartSamples) {
        io.midiOut.clear();
        for (const auto* event : events) {
            if (event && event->streamId == _streamId) {
                MidiMessage msg;
                msg.status = event->status;
                msg.data1 = event->data1;
                msg.data2 = event->data2;
                msg.channel = event->channel;
                // Calculate sample offset within block
                if (event->timeSamples >= blockStartSamples) {
                    msg.sampleOffset = event->timeSamples - blockStartSamples;
                } else {
                    msg.sampleOffset = 0; // Event before block start
                }
                io.midiOut.addMessage(msg);
            }
        }
    }

private:
    StreamId _streamId;
};

/// AudioLaneNode - one per Audio Lane
/// Injects audio segments from stream into graph
class AudioLaneNode : public GraphNode {
public:
    AudioLaneNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& laneId = ""
    )
        : GraphNode(id, NodeKind::AudioLane, trackId, laneId)
        , _streamId("") // Set by GraphEngine during stream injection
    {
    }

    void setStreamId(const StreamId& streamId) {
        _streamId = streamId;
    }

    const StreamId& getStreamId() const noexcept {
        return _streamId;
    }

    void process(const NodeProcessContext& npc) override {
        // Phase 3: Audio is injected via injectAudioSegment() before process() is called
        // This node just passes through (or could apply lane-level processing in future)
        // For now, audio is already in audioOut from injection
    }

    /// Inject audio segment into this lane node (called by GraphEngine)
    /// Phase 3: Loads real audio from AudioAssetSource
    void injectAudioSegment(
        const AudioSegmentCompiled* segment,
        uint64_t blockStartSamples,
        int numFrames,
        class AudioAssetSource* assetSource
    ) {
        if (!segment || segment->streamId != _streamId || !assetSource) {
            return;
        }

        // Calculate intersection of segment with current block
        uint64_t blockEndSamples = blockStartSamples + numFrames;
        uint64_t segmentStart = std::max(segment->startSamples, blockStartSamples);
        uint64_t segmentEnd = std::min(segment->endSamples, blockEndSamples);

        if (segmentStart >= segmentEnd) {
            return; // No intersection
        }

        // Calculate frame offsets within block
        int blockOffsetStart = static_cast<int>(segmentStart - blockStartSamples);
        int framesToRead = static_cast<int>(segmentEnd - segmentStart);

        // Calculate absolute sample position in asset
        uint64_t assetStartSample = segment->assetStartSamples + (segmentStart - segment->startSamples);

        // Read audio from asset source
        int numChannels = io.audioOut.numChannels();
        assetSource->readSamples(
            segment->assetId,
            assetStartSample,
            framesToRead,
            io.audioOut,
            blockOffsetStart,
            numChannels
        );

        // Phase 12b: Apply clip gain and fade envelopes
        if (segment->fadeInSamples > 0 || segment->fadeOutSamples > 0 || std::abs(segment->gainDb) > 0.0001) {
            // Convert dB gain to linear
            float gainLinear = 1.0f;
            if (std::abs(segment->gainDb) > 0.0001) {
                gainLinear = std::pow(10.0f, static_cast<float>(segment->gainDb) / 20.0f);
            }

            // Calculate clip length in samples (from segment duration)
            uint64_t clipLengthSamples = segment->endSamples - segment->startSamples;

            // Calculate position within the segment (relative to segment start)
            uint64_t segmentPosition = segmentStart - segment->startSamples;

            // Apply envelope to each sample in the block
            for (int frame = 0; frame < framesToRead; ++frame) {
                // Position within the clip (from clip start)
                uint64_t clipPosition = segmentPosition + frame;

                // Calculate fade factor (1.0 = no fade, 0.0 = fully faded)
                float fade = 1.0f;

                // Apply fade-in
                if (segment->fadeInSamples > 0 && clipPosition < segment->fadeInSamples) {
                    fade *= static_cast<float>(clipPosition) / static_cast<float>(segment->fadeInSamples);
                }

                // Apply fade-out
                if (segment->fadeOutSamples > 0 && clipPosition >= clipLengthSamples - segment->fadeOutSamples) {
                    uint64_t fadeOutPosition = clipLengthSamples - clipPosition;
                    fade *= static_cast<float>(fadeOutPosition) / static_cast<float>(segment->fadeOutSamples);
                }

                // Apply combined envelope (fade * gain)
                float envelope = fade * gainLinear;

                // Apply to all channels
                for (int ch = 0; ch < numChannels; ++ch) {
                    int frameIndex = blockOffsetStart + frame;
                    if (frameIndex >= 0 && frameIndex < io.audioOut.numFrames()) {
                        float sample = io.audioOut.getSample(frameIndex, ch);
                        io.audioOut.setSample(frameIndex, ch, sample * envelope);
                    }
                }
            }
        } else if (std::abs(segment->gainDb) > 0.0001) {
            // Only gain, no fades - apply gain to entire block
            float gainLinear = std::pow(10.0f, static_cast<float>(segment->gainDb) / 20.0f);
            for (int frame = 0; frame < framesToRead; ++frame) {
                int frameIndex = blockOffsetStart + frame;
                if (frameIndex >= 0 && frameIndex < io.audioOut.numFrames()) {
                    for (int ch = 0; ch < numChannels; ++ch) {
                        float sample = io.audioOut.getSample(frameIndex, ch);
                        io.audioOut.setSample(frameIndex, ch, sample * gainLinear);
                    }
                }
            }
        }

        // Phase 9: TODO - Apply time-stretching
        // For now, stretch metadata is stored but not applied
        // Future: Apply stretch algorithm based on segment->stretch.mode and segment->stretch.ratio
    }

private:
    StreamId _streamId;
};

/// PluginNodeKind - distinguishes plugin node types
/// Maps to NodeKind but used internally for plugin-specific behavior
enum class PluginNodeKind {
    MidiFx,    // MIDI effect plugins (MIDI in/out, optional audio passthrough)
    Instrument, // Instruments (MIDI in → audio out)
    AudioFx    // Audio effects (audio in/out, optional MIDI)
};

/// PluginNode - unified plugin node implementation
/// Replaces MidiFxNode, InstrumentNode, and AudioFxNode
///
/// I/O semantics by kind:
/// - MidiFx: MIDI in/out (required), audio optional/passthrough
/// - Instrument: MIDI in (required), audio out (required), MIDI out optional
/// - AudioFx: Audio in/out (required), MIDI optional
class PluginNode : public GraphNode {
public:
    // Legacy constructor (for backward compatibility)
    PluginNode(
        PluginNodeKind kind,
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, pluginKindToNodeKind(kind), trackId)
        , _pluginKind(kind)
        , _pluginId(pluginId)
        , _pluginInstanceId(std::string("plugin-instance:") + id)
        , _plugin(nullptr)
        , _muted(false)
    {
    }

    // Constructor with plugin descriptor
    PluginNode(
        PluginNodeKind kind,
        const NodeId& id,
        const std::string& trackId,
        const NodeDesc& desc,
        PluginHost* pluginHost
    )
        : GraphNode(id, pluginKindToNodeKind(kind), trackId)
        , _pluginKind(kind)
        , _pluginId(desc.pluginId.value_or(""))
        , _pluginInstanceId(
            desc.pluginInstanceId.has_value() && !desc.pluginInstanceId->empty()
                ? desc.pluginInstanceId.value()
                : std::string("plugin-instance:") + id
        )
        , _plugin(nullptr)
        , _muted(false)
    {
        if (pluginHost && desc.pluginFormat.has_value() && desc.pluginId.has_value()) {
            // Build PluginDescriptor from NodeDesc with kind-specific defaults
            PluginDescriptor pluginDesc;
            pluginDesc.format = desc.pluginFormat.value();
            pluginDesc.id = desc.pluginId.value();
            pluginDesc.name = desc.pluginId.value(); // Use ID as name for now

            // Set defaults based on plugin kind
            switch (_pluginKind) {
                case PluginNodeKind::MidiFx:
                    pluginDesc.numAudioInputs = desc.numAudioInputs.value_or(0);
                    pluginDesc.numAudioOutputs = desc.numAudioOutputs.value_or(0);
                    pluginDesc.hasMidiInput = desc.numMidiInputs.value_or(1) > 0;
                    pluginDesc.hasMidiOutput = desc.numMidiOutputs.value_or(1) > 0;
                    break;

                case PluginNodeKind::Instrument:
                    pluginDesc.numAudioInputs = desc.numAudioInputs.value_or(0);
                    pluginDesc.numAudioOutputs = desc.numAudioOutputs.value_or(2); // Instruments typically have audio out
                    pluginDesc.hasMidiInput = desc.numMidiInputs.value_or(1) > 0;
                    pluginDesc.hasMidiOutput = desc.numMidiOutputs.value_or(0) > 0;
                    break;

                case PluginNodeKind::AudioFx:
                    pluginDesc.numAudioInputs = desc.numAudioInputs.value_or(2);
                    pluginDesc.numAudioOutputs = desc.numAudioOutputs.value_or(2);
                    pluginDesc.hasMidiInput = desc.numMidiInputs.value_or(0) > 0;
                    pluginDesc.hasMidiOutput = desc.numMidiOutputs.value_or(0) > 0;
                    break;
            }

            // Store requested I/O for later negotiation (after GraphEngine sets config from snapshot)
            _requestedInputs = pluginDesc.numAudioInputs;
            _requestedOutputs = pluginDesc.numAudioOutputs;

            _plugin = pluginHost->createInstance(pluginDesc);
            if (!_plugin) {
                LOG_ERROR({"PluginNode"}, std::string("Failed to create plugin instance: ") + pluginDesc.id);
                _ioNegotiationOk = false;
            } else if (desc.pluginStateChunk.has_value()) {
                _plugin->setStateChunk(desc.pluginStateChunk.value());
            }
            // Negotiation will happen in prepare() after GraphEngine sets config from snapshot
        }
    }

    PluginNodeKind getPluginKind() const noexcept { return _pluginKind; }
    const std::string& getPluginId() const noexcept { return _pluginId; }
    const std::optional<std::string>& getPluginInstanceId() const noexcept { return _pluginInstanceId; }
    PluginInstance* getPlugin() const noexcept { return _plugin.get(); }
    std::vector<std::uint8_t> getStateChunk() const {
        if (!_plugin) {
            return {};
        }

        return _plugin->getStateChunk();
    }
    void setStateChunk(const std::vector<std::uint8_t>& chunk) {
        if (_plugin) {
            _plugin->setStateChunk(chunk);
        }
    }
    void setMuted(bool muted) noexcept { _muted = muted; }
    bool isMuted() const noexcept { return _muted; }

    void prepare(int sampleRate, int maxBlockSize) override {
        GraphNode::prepare(sampleRate, maxBlockSize);

        // Perform I/O negotiation now that GraphEngine has set config from snapshot
        if (_plugin) {
            // Get current config (set by GraphEngine from snapshot)
            const auto& config = getAudioConfig();
            int requestedInputs = config.numInputChannels;
            int requestedOutputs = config.numOutputChannels;

            // Negotiate audio I/O with plugin (query actual capabilities)
            // Snapshot config is source of truth - negotiation verifies plugin can support it
            if (_plugin->negotiateAudioIO(requestedInputs, requestedOutputs)) {
                // Check if negotiated config matches requested (snapshot) config
                const auto& negotiatedDesc = _plugin->getDescriptor();

                if (negotiatedDesc.numAudioInputs == requestedInputs &&
                    negotiatedDesc.numAudioOutputs == requestedOutputs) {
                    // Exact match - negotiation succeeded
                    _ioNegotiationOk = true;
                    LOG_DEBUG({"PluginNode", "IO"},
                        std::string("I/O negotiation succeeded for ") + getId() +
                        ": " + std::to_string(requestedInputs) + "/" + std::to_string(requestedOutputs));
                } else {
                    // Mismatch - plugin doesn't support requested layout
                    _ioNegotiationOk = false;
                    std::ostringstream msg;
                    msg << "I/O negotiation failed for " << getId()
                        << ": requested " << requestedInputs << "/" << requestedOutputs
                        << ", plugin supports " << negotiatedDesc.numAudioInputs << "/"
                        << negotiatedDesc.numAudioOutputs << " - node will be bypassed";
                    LOG_ERROR({"PluginNode", "IO"}, msg.str());
                    // Keep NodeAudioConfig as requested (snapshot is source of truth)
                }
            } else {
                // Negotiation call failed
                _ioNegotiationOk = false;
                LOG_ERROR({"PluginNode", "IO"},
                    std::string("I/O negotiation call failed for ") + getId() + " - node will be bypassed");
            }

            // Prepare plugin with negotiated I/O
            _plugin->prepare(static_cast<double>(sampleRate), maxBlockSize);
            _plugin->reset();
        } else {
            _ioNegotiationOk = false;
        }
    }

    void process(const NodeProcessContext& npc) override {
        if (_muted) {
            io.audioOut.clear();
            io.midiOut.clear();
            io.midiOut.append(io.midiIn);
            return;
        }

        // If plugin doesn't exist or I/O negotiation failed, bypass plugin
        if (!_plugin || !_ioNegotiationOk) {
            // Bypass behavior: pass input to output without processing
            // This ensures the graph continues to work even if plugin negotiation fails
            switch (_pluginKind) {
                case PluginNodeKind::MidiFx:
                    // Pass-through MIDI only
                    io.midiOut.clear();
                    io.midiOut.append(io.midiIn);
                    // Audio outputs remain cleared (MidiFx has no audio)
                    break;

                case PluginNodeKind::Instrument:
                    // Instrument: output silence if negotiation failed (no plugin to generate audio)
                    io.audioOut.clear();
                    io.midiOut.clear();
                    io.midiOut.append(io.midiIn);
                    break;

                case PluginNodeKind::AudioFx:
                    // Audio FX: pass-through audio (safe fallback)
                    io.audioOut.copyFrom(io.audioIn);
                    io.midiOut.clear();
                    io.midiOut.append(io.midiIn);
                    break;
            }
            return;
        }

        // Process through plugin (negotiation succeeded)
        _plugin->processAudioMidi(io.audioIn, io.audioOut, io.midiIn, io.midiOut, npc);
    }

private:
    PluginNodeKind _pluginKind;
    std::string _pluginId;
    std::optional<std::string> _pluginInstanceId;
    std::unique_ptr<PluginInstance> _plugin;
    bool _ioNegotiationOk = false;  // Set to true if I/O negotiation succeeded
    int _requestedInputs = 0;      // Stored from NodeDesc for negotiation
    int _requestedOutputs = 0;     // Stored from NodeDesc for negotiation
    bool _muted;

    // Convert PluginNodeKind to NodeKind for GraphNode base class
    static NodeKind pluginKindToNodeKind(PluginNodeKind kind) {
        switch (kind) {
            case PluginNodeKind::MidiFx:
                return NodeKind::MidiFx;
            case PluginNodeKind::Instrument:
                return NodeKind::Instrument;
            case PluginNodeKind::AudioFx:
                return NodeKind::AudioFx;
        }
    }
};

/// SendNode - sends to FX buses (ReceiveNodes)
/// Phase 3: Applies send level and outputs to connections
class SendNode : public GraphNode {
public:
    SendNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& receiveId = ""
    )
        : GraphNode(id, NodeKind::Send, trackId)
        , _receiveId(receiveId)
        , _sendLevel(1.0f) // Default: unity gain
    {
    }

    const std::string& getReceiveId() const noexcept { return _receiveId; }

    /// Set send level (linear gain, 0.0 = off, 1.0 = unity)
    void setSendLevel(float level) {
        _sendLevel = level;
    }

    float getSendLevel() const noexcept { return _sendLevel; }

    void process(const NodeProcessContext& npc) override {
        // Apply send level to input and output
        // Real-time safe: no allocations, just sample-by-sample scaling
        int numChannels = io.audioOut.numChannels();
        int numFrames = io.audioOut.numFrames();

        for (int ch = 0; ch < numChannels; ++ch) {
            for (int frame = 0; frame < numFrames; ++frame) {
                float sample = io.audioIn.getSample(frame, ch) * _sendLevel;
                io.audioOut.setSample(frame, ch, sample);
            }
        }

        // Pass through MIDI unchanged
        io.midiOut.clear();
        io.midiOut.append(io.midiIn);
    }

private:
    std::string _receiveId; // Target ReceiveNode ID
    float _sendLevel;       // Linear gain (0.0 to 1.0+)
};

/// FaderNode - final channel output into busses/device
/// Phase 3: Applies gain and panning
class FaderNode : public GraphNode {
public:
    enum class SpatialAdapter {
        None,
        Balance,
        PerChannelGain,
    };

    FaderNode(
        const NodeId& id,
        const std::string& trackId = ""
    )
        : GraphNode(id, NodeKind::Fader, trackId)
        , _gainLinear(1.0f)  // Default: unity gain
        , _pan(0.0f)         // Default: center pan
        , _muted(false)
        , _spatialAdapter(SpatialAdapter::Balance)
    {
        _channelGains.fill(1.0f);
    }

    /// Set gain (linear, 0.0 = off, 1.0 = unity)
    void setGain(float gain) {
        _gainLinear = gain;
    }

    float getGain() const noexcept { return _gainLinear; }

    void setSpatialAdapter(SpatialAdapter adapter) noexcept {
        _spatialAdapter = adapter;
    }

    SpatialAdapter getSpatialAdapter() const noexcept {
        return _spatialAdapter;
    }

    /// Set pan (-1.0 = left, 0.0 = center, 1.0 = right)
    /// Only used for stereo layouts
    void setPan(float pan) {
        _pan = pan;
        // Clamp to [-1.0, 1.0]
        if (_pan < -1.0f) _pan = -1.0f;
        if (_pan > 1.0f) _pan = 1.0f;
    }

    float getPan() const noexcept { return _pan; }

    void setChannelGain(int channelIndex, float gain) noexcept {
        if (channelIndex < 0 || channelIndex >= static_cast<int>(MAX_CHANNEL_GAINS)) {
            return;
        }

        float g = gain;
        if (g < 0.0f) g = 0.0f;
        if (g > 4.0f) g = 4.0f;
        _channelGains[static_cast<size_t>(channelIndex)] = g;
    }

    void setMuted(bool muted) noexcept { _muted = muted; }
    bool isMuted() const noexcept { return _muted; }

    void process(const NodeProcessContext& npc) override {
        // Phase 3: Apply gain and panning
        int numChannels = io.audioOut.numChannels();
        int numFrames = io.audioOut.numFrames();

        if (_muted) {
            for (int ch = 0; ch < numChannels; ++ch) {
                for (int frame = 0; frame < numFrames; ++frame) {
                    io.audioOut.setSample(frame, ch, 0.0f);
                }
            }
            return;
        }

        if (_spatialAdapter == SpatialAdapter::PerChannelGain) {
            // Apply gain-per-channel, independent of layout.
            for (int ch = 0; ch < numChannels; ++ch) {
                const float channelGain = (ch < static_cast<int>(MAX_CHANNEL_GAINS))
                    ? _channelGains[static_cast<size_t>(ch)]
                    : 1.0f;

                const float g = _gainLinear * channelGain;
                for (int frame = 0; frame < numFrames; ++frame) {
                    float sample = io.audioIn.getSample(frame, ch) * g;
                    io.audioOut.setSample(frame, ch, sample);
                }
            }
            return;
        }

        if (numChannels == 1) {
            // Mono: Apply gain only.
            //
            // Note: the `balance` adapter is a layout-aware control. For mono
            // nodes, balance has no meaningful left/right split at this node’s
            // channel surface, so we treat it as pass-through.
            const float channelGain = _channelGains[0];
            for (int frame = 0; frame < numFrames; ++frame) {
                float sample = io.audioIn.getSample(frame, 0) * (_gainLinear * channelGain);
                io.audioOut.setSample(frame, 0, sample);
            }
        } else if (numChannels == 2) {
            // Stereo: Apply gain + balance.
            //
            // `spatial.balance` is defined as a stereo-friendly balance control
            // that must not amplify above unity. Mapping:
            //
            // - if balance >= 0: gL = 1 - balance, gR = 1
            // - if balance < 0:  gL = 1,          gR = 1 + balance
            float gL = 1.0f;
            float gR = 1.0f;
            if (_pan >= 0.0f) {
                gL = 1.0f - _pan;
            } else {
                gR = 1.0f + _pan;
            }

            float leftGain = gL * _gainLinear * _channelGains[0];
            float rightGain = gR * _gainLinear * _channelGains[1];

            for (int frame = 0; frame < numFrames; ++frame) {
                float inLeft = io.audioIn.getSample(frame, 0);
                float inRight = (io.audioIn.numChannels() > 1) ? io.audioIn.getSample(frame, 1) : inLeft;

                io.audioOut.setSample(frame, 0, inLeft * leftGain);
                io.audioOut.setSample(frame, 1, inRight * rightGain);
            }
        } else {
            // Multi-channel:
            //
            // For common layouts, `balance` is applied as left/right group
            // attenuation (see Chorus: `docs/specs/engine/spatial-adapters.md`).
            //
            // If the channel count does not match a known layout, treat balance
            // as unsupported and fall back to uniform gain.
            float gL = 1.0f;
            float gR = 1.0f;
            if (_pan >= 0.0f) {
                gL = 1.0f - _pan;
            } else {
                gR = 1.0f + _pan;
            }

            const float gC = (gL + gR) * 0.5f;

            for (int ch = 0; ch < numChannels; ++ch) {
                const float channelGain = (ch < static_cast<int>(MAX_CHANNEL_GAINS))
                    ? _channelGains[static_cast<size_t>(ch)]
                    : 1.0f;

                const bool isKnownLayout =
                    (numChannels == 6) || (numChannels == 8) || (numChannels == 12);
                const float balanceMul = [&]() noexcept -> float {
                    if (!isKnownLayout) {
                        return 1.0f;
                    }

                    // Canonical order:
                    // - 5.1:   [L, R, C, LFE, Ls, Rs]
                    // - 7.1:   [L, R, C, LFE, Ls, Rs, Lrs, Rrs]
                    // - 7.1.4: [L, R, C, LFE, Ls, Rs, Lrs, Rrs, Ltf, Rtf, Ltr, Rtr]
                    switch (numChannels) {
                        case 6: {
                            // left: 0(L),4(Ls) | right: 1(R),5(Rs) | centre: 2(C),3(LFE)
                            if (ch == 0 || ch == 4) return gL;
                            if (ch == 1 || ch == 5) return gR;
                            return gC;
                        }
                        case 8: {
                            // left: 0(L),4(Ls),6(Lrs) | right: 1(R),5(Rs),7(Rrs) | centre: 2(C),3(LFE)
                            if (ch == 0 || ch == 4 || ch == 6) return gL;
                            if (ch == 1 || ch == 5 || ch == 7) return gR;
                            return gC;
                        }
                        case 12: {
                            // left: 0(L),4(Ls),6(Lrs),8(Ltf),10(Ltr)
                            // right: 1(R),5(Rs),7(Rrs),9(Rtf),11(Rtr)
                            // centre: 2(C),3(LFE)
                            if (ch == 0 || ch == 4 || ch == 6 || ch == 8 || ch == 10) return gL;
                            if (ch == 1 || ch == 5 || ch == 7 || ch == 9 || ch == 11) return gR;
                            return gC;
                        }
                        default:
                            return 1.0f;
                    }
                }();

                const float g = _gainLinear * channelGain * balanceMul;
                for (int frame = 0; frame < numFrames; ++frame) {
                    float sample = io.audioIn.getSample(frame, ch) * g;
                    io.audioOut.setSample(frame, ch, sample);
                }
            }
        }
    }

private:
    static constexpr size_t MAX_CHANNEL_GAINS = 64;

    float _gainLinear; // Linear gain (0.0 to 1.0+)
    float _pan;        // Pan position (-1.0 = left, 0.0 = center, 1.0 = right)
    bool _muted;
    SpatialAdapter _spatialAdapter;
    std::array<float, MAX_CHANNEL_GAINS> _channelGains;
};

/// ReceiveNode - receives from SendNodes (receive point for routed audio/MIDI)
/// Phase 2: Sum inputs (fan-in)
class ReceiveNode : public GraphNode {
public:
    ReceiveNode(
        const NodeId& id,
        const std::string& receiveName = ""
    )
        : GraphNode(id, NodeKind::Receive)
        , _receiveName(receiveName)
    {
    }

    const std::string& getReceiveName() const noexcept { return _receiveName; }

    void process(const NodeProcessContext& npc) override {
        // Phase 3: Pass-through (summing happens via connection routing)
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 4 - Apply receive processing (FX chain)
    }

private:
    std::string _receiveName;
};


#include "core/GraphNodesHardware.hpp"
