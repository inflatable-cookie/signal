#pragma once

/// GraphNode - Base class for all runtime graph nodes
///
/// Thread: Audio thread (process), Control thread (prepare)
/// Ownership: Owned by GraphEngine
///
/// This is the base interface for all nodes in the processing graph.
/// For Phase 1, prepare() and process() are no-op; they will be implemented
/// in future phases when DSP is added.

#include <string>
#include <cstdint>

/// Node kind - determines the type of processing node
enum class NodeKind {
    MidiLane,
    AudioLane,
    MidiFx,
    Instrument,
    AudioFx,
    Send,
    MixerChannel,
    Bus,
    Master
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
    /// For Phase 1, this is no-op; will allocate buffers, load plugins, etc. in future phases
    virtual void prepare(int sampleRate, int maxBlockSize) {
        (void)sampleRate;
        (void)maxBlockSize;
        // No-op for Phase 1
    }

    /// Process audio/MIDI (called on audio thread)
    /// For Phase 1, this is no-op; will process audio/MIDI buffers in future phases
    virtual void process() {
        // No-op for Phase 1
    }

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

private:
    NodeId _id;
    NodeKind _kind;
    std::string _trackId;
    std::string _laneId;
};

