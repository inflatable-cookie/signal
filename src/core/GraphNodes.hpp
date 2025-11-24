#pragma once

/// GraphNodes - Specialized node subclasses
///
/// Thread: Audio thread (process), Control thread (prepare)
/// Ownership: Owned by GraphEngine
///
/// These are concrete node implementations for each node kind.
/// For Phase 1, they are simple shells with no-op prepare/process.

#include "core/GraphNode.hpp"
#include <string>

/// MidiLaneNode - one per MIDI Lane
class MidiLaneNode : public GraphNode {
public:
    MidiLaneNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& laneId = ""
    )
        : GraphNode(id, NodeKind::MidiLane, trackId, laneId)
    {
    }
};

/// AudioLaneNode - one per Audio Lane
class AudioLaneNode : public GraphNode {
public:
    AudioLaneNode(
        const NodeId& id,
        const std::string& trackId = "",
        const std::string& laneId = ""
    )
        : GraphNode(id, NodeKind::AudioLane, trackId, laneId)
    {
    }
};

/// MidiFxNode - MIDI effect plugins
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

private:
    std::string _pluginId;
};

/// InstrumentNode - instruments (MIDI in → audio out)
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

private:
    std::string _pluginId;
};

/// AudioFxNode - audio effects
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

private:
    std::string _pluginId;
};

/// SendNode - sends to FX buses
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

private:
    std::string _busId;
};

/// MixerChannelNode - final channel output into busses/master
class MixerChannelNode : public GraphNode {
public:
    MixerChannelNode(
        const NodeId& id,
        const std::string& trackId = ""
    )
        : GraphNode(id, NodeKind::MixerChannel, trackId)
    {
    }
};

/// BusNode - receives from SendNodes (FX bus)
class BusNode : public GraphNode {
public:
    BusNode(
        const NodeId& id,
        const std::string& busName = ""
    )
        : GraphNode(id, NodeKind::Bus)
        , _busName(busName)
    {
    }

    const std::string& getBusName() const noexcept { return _busName; }

private:
    std::string _busName;
};

/// MasterNode - master output
class MasterNode : public GraphNode {
public:
    MasterNode(const NodeId& id)
        : GraphNode(id, NodeKind::Master)
    {
    }
};

