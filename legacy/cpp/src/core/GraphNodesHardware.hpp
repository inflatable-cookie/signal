#pragma once

#include "core/GraphNode.hpp"
#include "core/NodeAudioConfig.hpp"
#include "core/EngineHost.hpp"
#include "logging/Logging.hpp"
#include <cstring>
#include <optional>
#include <string>
#include <sstream>
#include <vector>
#include <algorithm>

/// HardwareAudioOutputNode - writes to hardware audio output
/// Phase 2: Pass-through (writes to EngineHost output buffer)
///
/// This node represents an endpoint that streams audio to a hardware output.
/// Multiple output nodes may be supported in future (e.g. multiple hardware outputs, cue mixes, recording taps).
class HardwareAudioOutputNode : public GraphNode {
public:
    HardwareAudioOutputNode(const NodeId& id)
        : GraphNode(id, NodeKind::HardwareAudioOutput)
        , _engineHost(nullptr)
    {
        // Config will be set from device channel count during prepare()
    }

    /// Set EngineHost reference (called by GraphEngine after node creation)
    /// HardwareAudioOutputNode needs this to query active device channel count
    void setEngineHost(EngineHost* engineHost);

    /// Get device channel count (helper to avoid including EngineHost.hpp in header)
    int getDeviceChannelCount() const;

    void prepare(int sampleRate, int maxBlockSize) override {
        GraphNode::prepare(sampleRate, maxBlockSize);

        // Update channel config from active device
        if (_engineHost) {
            int deviceChannels = getDeviceChannelCount();
            if (deviceChannels > 0) {
                NodeAudioConfig config;
                config.numInputChannels = deviceChannels;
                config.numOutputChannels = deviceChannels;
                // Determine layout from channel count
                if (deviceChannels == 1) {
                    config.layout = ChannelLayout::Mono;
                } else if (deviceChannels == 2) {
                    config.layout = ChannelLayout::Stereo;
                } else {
                    config.layout = ChannelLayout::Stereo; // Default for multi-channel (future: add Multi enum)
                }
                setAudioConfig(config);

                // Resize buffers to match device channels
                // Note: resize() signature is resize(numChannels, numFrames)
                io.audioIn.resize(deviceChannels, maxBlockSize);
                io.audioOut.resize(deviceChannels, maxBlockSize);

                std::ostringstream msg;
                msg << "HardwareAudioOutputNode " << getId() << " configured for " << deviceChannels << " channel(s)";
                LOG_INFO({"HardwareAudioOutputNode", "Channels"}, msg.str());
            } else {
                // No active device - set to 0 channels (will invalidate routing)
                NodeAudioConfig config;
                config.numInputChannels = 0;
                config.numOutputChannels = 0;
                config.layout = ChannelLayout::Mono; // Not meaningful
                setAudioConfig(config);
                LOG_WARN({"HardwareAudioOutputNode"}, std::string("No active device - HardwareAudioOutputNode ") + getId() + " has 0 channels");
            }
        } else {
            // No EngineHost reference - use default stereo (will be updated when EngineHost is set)
            NodeAudioConfig config;
            config.numInputChannels = 2;
            config.numOutputChannels = 2;
            config.layout = ChannelLayout::Stereo;
            setAudioConfig(config);
            LOG_WARN({"HardwareAudioOutputNode"}, std::string("No EngineHost reference - HardwareAudioOutputNode ") + getId() + " using default stereo");
        }
    }

    void process(const NodeProcessContext& npc) override {
        // HardwareAudioOutputNode processes audio from upstream and prepares it for hardware output
        // Handles channel count mismatches explicitly (expansion/truncation)
        // Real-time safe: no allocations, no locks, no logging in hot path

        const int inputChannels = io.audioIn.numChannels();
        const int outputChannels = io.audioOut.numChannels();
        const int numFrames = io.audioOut.numFrames();

        // Clear output first
        io.audioOut.clear();

        if (inputChannels == 0 || outputChannels == 0) {
            // No input or output - output is already cleared
            return;
        }

        if (inputChannels == outputChannels) {
            // Exact match - simple copy
            io.audioOut.copyFrom(io.audioIn);
        } else if (inputChannels < outputChannels) {
            // Channel expansion: copy input channels, duplicate to fill remaining
            // Strategy: copy available channels, duplicate last channel to remaining channels
            const int channelsToCopy = std::min(inputChannels, outputChannels);

            for (int ch = 0; ch < channelsToCopy; ++ch) {
                const float* src = io.audioIn.getChannelData(ch);
                float* dst = io.audioOut.getChannelData(ch);
                if (src && dst) {
                    std::memcpy(dst, src, numFrames * sizeof(float));
                }
            }

            // Fill remaining channels by duplicating the last input channel
            // For mono -> stereo: duplicate to all output channels
            // For stereo -> 4ch: duplicate L to ch 2, R to ch 3
            if (inputChannels == 1 && outputChannels >= 2) {
                // Mono -> stereo: duplicate to all output channels
                const float* mono = io.audioIn.getChannelData(0);
                if (mono) {
                    for (int ch = 1; ch < outputChannels; ++ch) {
                        float* dst = io.audioOut.getChannelData(ch);
                        if (dst) {
                            std::memcpy(dst, mono, numFrames * sizeof(float));
                        }
                    }
                }
            } else {
                // Multi-channel expansion: duplicate last channel to remaining
                const float* lastChannel = io.audioIn.getChannelData(inputChannels - 1);
                if (lastChannel) {
                    for (int ch = inputChannels; ch < outputChannels; ++ch) {
                        float* dst = io.audioOut.getChannelData(ch);
                        if (dst) {
                            std::memcpy(dst, lastChannel, numFrames * sizeof(float));
                        }
                    }
                }
            }
        } else {
            // Channel truncation: copy first N channels, drop the rest
            // Strategy: copy first outputChannels from input
            for (int ch = 0; ch < outputChannels; ++ch) {
                const float* src = io.audioIn.getChannelData(ch);
                float* dst = io.audioOut.getChannelData(ch);
                if (src && dst) {
                    std::memcpy(dst, src, numFrames * sizeof(float));
                }
            }
            // Remaining input channels are dropped (deterministic: first N channels kept)
        }

        // TODO: Phase 4 - Apply device processing, metering
    }

private:
    EngineHost* _engineHost;  // Reference to EngineHost for querying device channel count
};

// Implementation of HardwareAudioOutputNode methods that need EngineHost (after class definition)
#include "core/EngineHost.hpp"

inline void HardwareAudioOutputNode::setEngineHost(EngineHost* engineHost) {
    _engineHost = engineHost;
}

inline int HardwareAudioOutputNode::getDeviceChannelCount() const {
    if (_engineHost) {
        return _engineHost->getNumOutputChannels();
    }
    return 0;
}

/// AudioInputNode - represents a hardware audio input stream
/// Feeds live audio from the audio backend into the graph
class AudioInputNode : public GraphNode {
public:
    AudioInputNode(
        const NodeId& id,
        const std::string& deviceId = "",
        int inputChannelIndex = 0
    )
        : GraphNode(id, NodeKind::HardwareAudioInput)
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
    /// Extracts a single channel from interleaved input and writes to output buffer
    void injectInputAudio(const float* inputData, int numChannels, int numFrames, int channelOffset) {
        // Extract single channel from interleaved input
        if (channelOffset < numChannels && io.audioOut.numChannels() > 0) {
            const int numFramesToCopy = std::min(numFrames, io.audioOut.numFrames());
            float* outChannel = io.audioOut.getChannelData(0);
            if (outChannel) {
                for (int frame = 0; frame < numFramesToCopy; ++frame) {
                    outChannel[frame] = inputData[frame * numChannels + channelOffset];
                }
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
        : GraphNode(id, NodeKind::HardwareMidiInput)
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
