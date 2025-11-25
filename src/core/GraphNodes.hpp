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
#include <string>
#include <vector>
#include <cmath>
#include <algorithm>
#include <iostream>

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

        // Clear output buffer (will be filled with audio segments)
        io.audioOut.clear();

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

        // Phase 9: TODO - Apply fade-in/fade-out ramps
        // For now, fade metadata is stored but not applied
        // Future: Apply gain ramps based on segment->fadeInSamples, fadeOutSamples, fadeInCurve, fadeOutCurve

        // Phase 9: TODO - Apply time-stretching
        // For now, stretch metadata is stored but not applied
        // Future: Apply stretch algorithm based on segment->stretch.mode and segment->stretch.ratio
    }

private:
    StreamId _streamId;
};

/// MidiFxNode - MIDI effect plugins
/// Phase 4: Processes MIDI through plugin
class MidiFxNode : public GraphNode {
public:
    // Legacy constructor (for backward compatibility)
    MidiFxNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, NodeKind::MidiFx, trackId)
        , _pluginId(pluginId)
        , _plugin(nullptr)
    {
    }

    // Phase 4: Constructor with plugin descriptor
    MidiFxNode(
        const NodeId& id,
        const std::string& trackId,
        const NodeDesc& desc,
        PluginHost* pluginHost
    )
        : GraphNode(id, NodeKind::MidiFx, trackId)
        , _pluginId(desc.pluginId.value_or(""))
        , _plugin(nullptr)
    {
        if (pluginHost && desc.pluginFormat.has_value() && desc.pluginId.has_value()) {
            // Build PluginDescriptor from NodeDesc
            PluginDescriptor pluginDesc;
            pluginDesc.format = desc.pluginFormat.value();
            pluginDesc.id = desc.pluginId.value();
            pluginDesc.name = desc.pluginId.value(); // Use ID as name for now
            pluginDesc.numAudioInputs = desc.numAudioInputs.value_or(0);
            pluginDesc.numAudioOutputs = desc.numAudioOutputs.value_or(0);
            pluginDesc.hasMidiInput = desc.numMidiInputs.value_or(1) > 0;
            pluginDesc.hasMidiOutput = desc.numMidiOutputs.value_or(1) > 0;

            _plugin = pluginHost->createInstance(pluginDesc);
            if (!_plugin) {
                std::cerr << "[MidiFxNode] Failed to create plugin instance: " << pluginDesc.id << std::endl;
            }
        }
    }

    const std::string& getPluginId() const noexcept { return _pluginId; }

    void prepare(int sampleRate, int maxBlockSize) override {
        GraphNode::prepare(sampleRate, maxBlockSize);
        if (_plugin) {
            _plugin->prepare(static_cast<double>(sampleRate), maxBlockSize);
            _plugin->reset();
        }
    }

    void process(const NodeProcessContext& npc) override {
        if (_plugin) {
            // Phase 4: Process through plugin
            _plugin->processAudioMidi(io.audioIn, io.audioOut, io.midiIn, io.midiOut, npc);
        } else {
            // Fallback: Pass-through MIDI
            io.midiOut.clear();
            io.midiOut.append(io.midiIn);
        }
    }

    PluginInstance* getPlugin() const noexcept { return _plugin.get(); }

private:
    std::string _pluginId;
    std::unique_ptr<PluginInstance> _plugin;
};

/// InstrumentNode - instruments (MIDI in → audio out)
/// Phase 4: Processes MIDI through instrument plugin
class InstrumentNode : public GraphNode {
public:
    // Legacy constructor (for backward compatibility)
    InstrumentNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, NodeKind::Instrument, trackId)
        , _pluginId(pluginId)
        , _plugin(nullptr)
    {
    }

    // Phase 4: Constructor with plugin descriptor
    InstrumentNode(
        const NodeId& id,
        const std::string& trackId,
        const NodeDesc& desc,
        PluginHost* pluginHost
    )
        : GraphNode(id, NodeKind::Instrument, trackId)
        , _pluginId(desc.pluginId.value_or(""))
        , _plugin(nullptr)
    {
        if (pluginHost && desc.pluginFormat.has_value() && desc.pluginId.has_value()) {
            // Build PluginDescriptor from NodeDesc
            PluginDescriptor pluginDesc;
            pluginDesc.format = desc.pluginFormat.value();
            pluginDesc.id = desc.pluginId.value();
            pluginDesc.name = desc.pluginId.value(); // Use ID as name for now
            pluginDesc.numAudioInputs = desc.numAudioInputs.value_or(0);
            pluginDesc.numAudioOutputs = desc.numAudioOutputs.value_or(2); // Instruments typically have audio out
            pluginDesc.hasMidiInput = desc.numMidiInputs.value_or(1) > 0;
            pluginDesc.hasMidiOutput = desc.numMidiOutputs.value_or(0) > 0;

            _plugin = pluginHost->createInstance(pluginDesc);
            if (!_plugin) {
                std::cerr << "[InstrumentNode] Failed to create plugin instance: " << pluginDesc.id << std::endl;
            }
        }
    }

    const std::string& getPluginId() const noexcept { return _pluginId; }

    void prepare(int sampleRate, int maxBlockSize) override {
        GraphNode::prepare(sampleRate, maxBlockSize);
        if (_plugin) {
            _plugin->prepare(static_cast<double>(sampleRate), maxBlockSize);
            _plugin->reset();
        }
    }

    void process(const NodeProcessContext& npc) override {
        if (_plugin) {
            // Phase 4: Process MIDI through instrument plugin
            _plugin->processAudioMidi(io.audioIn, io.audioOut, io.midiIn, io.midiOut, npc);
        } else {
            // Fallback: Pass-through audio, ignore MIDI
            io.audioOut.copyFrom(io.audioIn);
        }
    }

    PluginInstance* getPlugin() const noexcept { return _plugin.get(); }

private:
    std::string _pluginId;
    std::unique_ptr<PluginInstance> _plugin;
};

/// AudioFxNode - audio effects
/// Phase 4: Processes audio through plugin
class AudioFxNode : public GraphNode {
public:
    // Legacy constructor (for backward compatibility)
    AudioFxNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, NodeKind::AudioFx, trackId)
        , _pluginId(pluginId)
        , _plugin(nullptr)
    {
    }

    // Phase 4: Constructor with plugin descriptor
    AudioFxNode(
        const NodeId& id,
        const std::string& trackId,
        const NodeDesc& desc,
        PluginHost* pluginHost
    )
        : GraphNode(id, NodeKind::AudioFx, trackId)
        , _pluginId(desc.pluginId.value_or(""))
        , _plugin(nullptr)
    {
        if (pluginHost && desc.pluginFormat.has_value() && desc.pluginId.has_value()) {
            // Build PluginDescriptor from NodeDesc
            PluginDescriptor pluginDesc;
            pluginDesc.format = desc.pluginFormat.value();
            pluginDesc.id = desc.pluginId.value();
            pluginDesc.name = desc.pluginId.value(); // Use ID as name for now
            pluginDesc.numAudioInputs = desc.numAudioInputs.value_or(2);
            pluginDesc.numAudioOutputs = desc.numAudioOutputs.value_or(2);
            pluginDesc.hasMidiInput = desc.numMidiInputs.value_or(0) > 0;
            pluginDesc.hasMidiOutput = desc.numMidiOutputs.value_or(0) > 0;

            _plugin = pluginHost->createInstance(pluginDesc);
            if (!_plugin) {
                std::cerr << "[AudioFxNode] Failed to create plugin instance: " << pluginDesc.id << std::endl;
            }
        }
    }

    const std::string& getPluginId() const noexcept { return _pluginId; }

    void prepare(int sampleRate, int maxBlockSize) override {
        GraphNode::prepare(sampleRate, maxBlockSize);
        if (_plugin) {
            _plugin->prepare(static_cast<double>(sampleRate), maxBlockSize);
            _plugin->reset();
        }
    }

    void process(const NodeProcessContext& npc) override {
        if (_plugin) {
            // Phase 4: Process through plugin
            _plugin->processAudioMidi(io.audioIn, io.audioOut, io.midiIn, io.midiOut, npc);
        } else {
            // Fallback: Pass-through audio
            io.audioOut.copyFrom(io.audioIn);
        }
    }

    PluginInstance* getPlugin() const noexcept { return _plugin.get(); }

private:
    std::string _pluginId;
    std::unique_ptr<PluginInstance> _plugin;
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
        // Phase 3: Apply send level and output
        // The send level is applied during connection routing in GraphEngine
        // For now, just copy input to output (scaling happens in routing)
        io.audioOut.copyFrom(io.audioIn);
    }

private:
    std::string _receiveId; // Target ReceiveNode ID
    float _sendLevel;       // Linear gain (0.0 to 1.0+)
};

/// MixerChannelNode - final channel output into busses/device
/// Phase 3: Applies gain and panning
class MixerChannelNode : public GraphNode {
public:
    MixerChannelNode(
        const NodeId& id,
        const std::string& trackId = ""
    )
        : GraphNode(id, NodeKind::MixerChannel, trackId)
        , _gainLinear(1.0f)  // Default: unity gain
        , _pan(0.0f)         // Default: center pan
    {
    }

    /// Set gain (linear, 0.0 = off, 1.0 = unity)
    void setGain(float gain) {
        _gainLinear = gain;
    }

    float getGain() const noexcept { return _gainLinear; }

    /// Set pan (-1.0 = left, 0.0 = center, 1.0 = right)
    /// Only used for stereo layouts
    void setPan(float pan) {
        _pan = pan;
        // Clamp to [-1.0, 1.0]
        if (_pan < -1.0f) _pan = -1.0f;
        if (_pan > 1.0f) _pan = 1.0f;
    }

    float getPan() const noexcept { return _pan; }

    void process(const NodeProcessContext& npc) override {
        // Phase 3: Apply gain and panning
        int numChannels = io.audioOut.numChannels();
        int numFrames = io.audioOut.numFrames();

        if (numChannels == 1) {
            // Mono: Apply gain only
            for (int frame = 0; frame < numFrames; ++frame) {
                float sample = io.audioIn.getSample(frame, 0) * _gainLinear;
                io.audioOut.setSample(frame, 0, sample);
            }
        } else if (numChannels == 2) {
            // Stereo: Apply gain and pan
            // Simple linear pan: left = (1 - pan), right = (1 + pan)
            // For pan = -1.0 (left): left = 2.0, right = 0.0
            // For pan = 0.0 (center): left = 1.0, right = 1.0
            // For pan = 1.0 (right): left = 0.0, right = 2.0
            float leftGain = (1.0f - _pan) * _gainLinear;
            float rightGain = (1.0f + _pan) * _gainLinear;

            for (int frame = 0; frame < numFrames; ++frame) {
                float inLeft = io.audioIn.getSample(frame, 0);
                float inRight = (io.audioIn.numChannels() > 1) ? io.audioIn.getSample(frame, 1) : inLeft;

                io.audioOut.setSample(frame, 0, inLeft * leftGain);
                io.audioOut.setSample(frame, 1, inRight * rightGain);
            }
        } else {
            // Multi-channel: Apply gain uniformly (no panning)
            for (int ch = 0; ch < numChannels; ++ch) {
                for (int frame = 0; frame < numFrames; ++frame) {
                    float sample = io.audioIn.getSample(frame, ch) * _gainLinear;
                    io.audioOut.setSample(frame, ch, sample);
                }
            }
        }
    }

private:
    float _gainLinear; // Linear gain (0.0 to 1.0+)
    float _pan;        // Pan position (-1.0 = left, 0.0 = center, 1.0 = right)
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

/// DeviceNode - writes to hardware device output
/// Phase 2: Pass-through (writes to EngineHost output buffer)
///
/// This node represents an endpoint that streams audio (and possibly MIDI) to a hardware device.
/// Multiple DeviceNodes may be supported in future (e.g., different output devices, cue mixes, recording taps).
/// For now, it acts in the role previously named "master" (single device output).
class DeviceNode : public GraphNode {
public:
    DeviceNode(const NodeId& id)
        : GraphNode(id, NodeKind::Device)
    {
    }

    void process(const NodeProcessContext& npc) override {
        // Phase 3: Pass-through (output will be copied to EngineHost output buffer)
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 4 - Apply device processing, metering
    }
};

/// AudioInputNode - represents a hardware audio input stream
/// Feeds live audio from the audio backend into the graph
class AudioInputNode : public GraphNode {
public:
    AudioInputNode(
        const NodeId& id,
        const std::string& deviceId = "",
        int inputChannelIndex = 0
    )
        : GraphNode(id, NodeKind::AudioInput)
        , _deviceId(deviceId)
        , _inputChannelIndex(inputChannelIndex)
    {
        // Audio input nodes typically have mono or stereo output
        // Default to mono for Phase 7
        NodeAudioConfig config;
        config.layout = ChannelLayout::Mono;
        config.numInputChannels = 0;  // No inputs (reads from backend)
        config.numOutputChannels = 1; // Mono output
        setAudioConfig(config);
    }

    const std::string& getDeviceId() const noexcept {
        return _deviceId;
    }

    int getInputChannelIndex() const noexcept {
        return _inputChannelIndex;
    }

    void process(const NodeProcessContext& npc) override {
        // Audio is injected from backend input buffers before process() is called
        // This node just passes through (could add input gain/trim in future)
    }

    /// Inject audio from backend input (called by EngineHost before graph processing)
    void injectInputAudio(const float* inputData, int numChannels, int numFrames, int channelOffset) {
        // Copy input channel data to output buffer
        // For Phase 7, assume mono extraction from interleaved input
        if (channelOffset < numChannels && io.audioOut.numChannels() > 0) {
            for (int frame = 0; frame < numFrames && frame < io.audioOut.numFrames(); ++frame) {
                int inputIndex = frame * numChannels + channelOffset;
                io.audioOut.setSample(0, frame, inputData[inputIndex]);
            }
        }
    }

private:
    std::string _deviceId;
    int _inputChannelIndex;
};

/// MidiInputNode - represents a MIDI input source
/// Feeds live MIDI from the MIDI backend into the graph
class MidiInputNode : public GraphNode {
public:
    MidiInputNode(
        const NodeId& id,
        const std::string& portId = ""
    )
        : GraphNode(id, NodeKind::MidiInput)
        , _portId(portId)
    {
    }

    const std::string& getPortId() const noexcept {
        return _portId;
    }

    void process(const NodeProcessContext& npc) override {
        // MIDI is injected from backend input before process() is called
        // This node just passes through
    }

    /// Inject MIDI from backend input (called by EngineHost before graph processing)
    void injectInputMidi(const std::vector<MidiMessage>& messages) {
        io.midiOut.clear();
        for (const auto& msg : messages) {
            io.midiOut.addMessage(msg);
        }
    }

private:
    std::string _portId;
};

