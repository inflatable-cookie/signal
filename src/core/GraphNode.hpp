#pragma once

/// GraphNode - Base class for all runtime graph nodes
///
/// Thread: Audio thread (process), Control thread (prepare)
/// Ownership: Owned by GraphEngine
///
/// This is the base interface for all nodes in the processing graph.
/// Phase 2 adds internal buffers and stream injection support.

#include "core/NodeBuffers.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/NodeProcessContext.hpp"
#include "core/NodeAudioConfig.hpp"
#include <string>
#include <cstdint>
#include <forward_list>

/// Node kind - determines the type of processing node
enum class NodeKind {
    MidiLane,
    AudioLane,
    MidiFx,
    Instrument,
    AudioFx,
    Send,
    MixerChannel,
    Receive,  // Receives from SendNodes (was Bus)
    Device    // Writes to hardware device output (was Master)
};

/// Node identifier (matches Pulse NodeId)
using NodeId = std::string;

/// Stream identifier (matches Pulse StreamId)
using StreamId = std::string;

/// Base class for all graph nodes
class GraphNode {
public:
    virtual ~GraphNode() = default;

    /// Get node ID
    const NodeId& getId() const noexcept { return _id; }

    /// Get node kind
    NodeKind getKind() const noexcept { return _kind; }

    /// Get optional track ID (for lane/channel nodes)
    const std::string& getTrackId() const noexcept { return _trackId; }

    /// Get optional lane ID (for lane nodes)
    const std::string& getLaneId() const noexcept { return _laneId; }

    /// Prepare node for processing (called on control thread)
    /// Allocates buffers based on node configuration
    virtual void prepare(int sampleRate, int maxBlockSize) {
        _sampleRate = sampleRate;
        _maxBlockSize = maxBlockSize;
        // Use audio config for buffer sizing
        io.audioIn.resize(_audioConfig.numInputChannels, maxBlockSize);
        io.audioOut.resize(_audioConfig.numOutputChannels, maxBlockSize);
    }

    /// Process audio/MIDI (called on audio thread)
    /// Must be implemented by subclasses
    /// Phase 3: Uses NodeProcessContext instead of EngineRenderContext
    virtual void process(const NodeProcessContext& npc) = 0;

    /// Get audio configuration
    const NodeAudioConfig& getAudioConfig() const noexcept { return _audioConfig; }

    /// Set audio configuration (called during prepare)
    void setAudioConfig(const NodeAudioConfig& config) {
        _audioConfig = config;
    }

    /// Node I/O buffers
    struct NodeIO {
        AudioBuffer audioIn;
        AudioBuffer audioOut;
        MidiBuffer midiIn;
        MidiBuffer midiOut;
    };

    NodeIO io;

protected:
    GraphNode(
        const NodeId& id,
        NodeKind kind,
        const std::string& trackId = "",
        const std::string& laneId = ""
    )
        : _id(id)
        , _kind(kind)
        , _trackId(trackId)
        , _laneId(laneId)
    {
    }

protected:
    int _sampleRate = 44100;
    int _maxBlockSize = 512;
    NodeAudioConfig _audioConfig; // Channel layout configuration

private:
    NodeId _id;
    NodeKind _kind;
    std::string _trackId;
    std::string _laneId;
};

