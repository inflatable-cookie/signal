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
#include <string>
#include <vector>
#include <cmath>

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

    void process(EngineRenderContext& ctx) override {
        // Clear output
        io.midiOut.clear();

        // TODO: Phase 3 - Get MIDI events from StreamScheduler
        // For Phase 2, we'll inject events externally via injectMidiEvents()
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

    void process(EngineRenderContext& ctx) override {
        // Clear output
        io.audioOut.clear();

        // TODO: Phase 3 - Load audio from assets
        // For Phase 2, we'll inject audio externally via injectAudioSegment()
        // For now, output silence or test tone
    }

    /// Inject audio segment into this lane node (called by GraphEngine)
    /// For Phase 2, this generates a test tone; Phase 3 will load real audio
    void injectAudioSegment(
        const AudioSegmentCompiled* segment,
        uint64_t blockStartSamples,
        int numFrames
    ) {
        if (!segment || segment->streamId != _streamId) {
            return;
        }

        // For Phase 2: Generate test tone (440 Hz sine wave)
        // Phase 3 will load real audio from assets
        const float testToneFreq = 440.0f;
        const float amplitude = 0.1f;
        const float sampleRate = static_cast<float>(_sampleRate);

        for (int frame = 0; frame < numFrames; ++frame) {
            uint64_t globalSample = blockStartSamples + frame;
            if (globalSample >= segment->startSamples && globalSample < segment->endSamples) {
                float time = static_cast<float>(globalSample) / sampleRate;
                float sample = amplitude * std::sin(2.0f * 3.14159265359f * testToneFreq * time);
                // Write to both channels (stereo)
                io.audioOut.setSample(frame, 0, sample);
                io.audioOut.setSample(frame, 1, sample);
            }
        }
    }

private:
    StreamId _streamId;
};

/// MidiFxNode - MIDI effect plugins
/// Phase 2: Pass-through MIDI
class MidiFxNode : public GraphNode {
public:
    MidiFxNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, NodeKind::MidiFx, trackId)
        , _pluginId(pluginId)
    {
    }

    const std::string& getPluginId() const noexcept { return _pluginId; }

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through MIDI
        io.midiOut = io.midiIn;
        // TODO: Phase 4 - Process through plugin
    }

private:
    std::string _pluginId;
};

/// InstrumentNode - instruments (MIDI in → audio out)
/// Phase 2: Pass-through audio, ignore MIDI
class InstrumentNode : public GraphNode {
public:
    InstrumentNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, NodeKind::Instrument, trackId)
        , _pluginId(pluginId)
    {
    }

    const std::string& getPluginId() const noexcept { return _pluginId; }

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through audio, ignore MIDI
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 4 - Process MIDI through instrument plugin
    }

private:
    std::string _pluginId;
};

/// AudioFxNode - audio effects
/// Phase 2: Pass-through audio
class AudioFxNode : public GraphNode {
public:
    AudioFxNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& pluginId = ""
    )
        : GraphNode(id, NodeKind::AudioFx, trackId)
        , _pluginId(pluginId)
    {
    }

    const std::string& getPluginId() const noexcept { return _pluginId; }

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through audio
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 4 - Process through plugin
    }

private:
    std::string _pluginId;
};

/// SendNode - sends to FX buses
/// Phase 2: Pass-through audio (forking to buses will come later)
class SendNode : public GraphNode {
public:
    SendNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& busId = ""
    )
        : GraphNode(id, NodeKind::Send, trackId)
        , _busId(busId)
    {
    }

    const std::string& getBusId() const noexcept { return _busId; }

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through audio
        // TODO: Phase 3 - Fork audio to bus
        io.audioOut.copyFrom(io.audioIn);
    }

private:
    std::string _busId;
};

/// MixerChannelNode - final channel output into busses/device
/// Phase 2: Pass-through or sum inputs
class MixerChannelNode : public GraphNode {
public:
    MixerChannelNode(
        const NodeId& id,
        const std::string& trackId = ""
    )
        : GraphNode(id, NodeKind::MixerChannel, trackId)
    {
    }

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through (summing happens via connection routing)
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 3 - Apply gain/mute/solo
    }
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

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through (summing happens via connection routing)
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 3 - Apply receive processing
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

    void process(EngineRenderContext& ctx) override {
        // Phase 2: Pass-through (output will be copied to EngineHost output buffer)
        io.audioOut.copyFrom(io.audioIn);
        // TODO: Phase 3 - Apply device processing, metering
    }
};

