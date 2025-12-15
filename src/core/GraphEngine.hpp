#pragma once

/// GraphEngine - Runtime graph container and execution manager
///
/// Thread: Control thread (loadGraphSnapshot, prepareGraph)
///         Audio thread (getExecutionOrder, getStreamBindings - read-only)
/// Ownership: Owned by EngineHost
///
/// This class owns the runtime graph structure:
/// - Node instances (keyed by NodeId)
/// - Connections between nodes
/// - Stream input bindings (where streams enter the graph)
/// - Execution order (topologically sorted)
///
/// For Phase 1, this is purely structural - no DSP is performed.

#include "core/GraphNode.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/NodeProcessContext.hpp"
#include "core/StreamScheduler.hpp"
#include "core/AudioAssetSource.hpp"
#include "core/PluginHost.hpp"
#include <memory>
#include <unordered_map>
#include <vector>
#include <string>

// Forward declaration
class EngineHost;

/// Connection between node ports
struct GraphConnection {
    NodeId fromNodeId;
    uint32_t fromOutputIndex;
    NodeId toNodeId;
    uint32_t toInputIndex;
    bool isValid = true;  // Set to false if routing validation fails
};

/// Stream input binding - where a stream enters the graph
struct StreamInputBinding {
    StreamId streamId;
    NodeId targetNodeId;   // Likely a MidiLaneNode or AudioLaneNode
    uint32_t targetInputIndex;
};

class GraphEngine {
public:
    GraphEngine();
    ~GraphEngine();

    /// Load a graph snapshot (replaces entire graph)
    /// Called on control thread
    /// @param snapshot Graph snapshot from Pulse
    /// @param pluginHost Plugin host for creating plugin instances
    /// @param engineHost EngineHost reference (for HardwareAudioOutputNode to query device channel count)
    void loadGraphSnapshot(const GraphSnapshot& snapshot, PluginHost* pluginHost = nullptr, EngineHost* engineHost = nullptr);

    /// Prepare graph for processing (called on control thread)
    /// Calls prepare() on all nodes in execution order
    void prepareGraph(int sampleRate, int maxBlockSize);

    /// Get execution order (read-only, safe for audio thread)
    /// Returns nodes in topological order (sources before consumers)
    const std::vector<GraphNode*>& getExecutionOrder() const noexcept;

    /// Get stream bindings (read-only, safe for audio thread)
    /// Returns all stream input bindings
    const std::vector<StreamInputBinding>& getStreamBindings() const noexcept;

    /// Find a node by ID (read-only, safe for audio thread)
    GraphNode* findNode(const NodeId& id) noexcept;
    const GraphNode* findNode(const NodeId& id) const noexcept;

    /// Check if graph has been loaded (has nodes)
    bool hasGraph() const noexcept;

    /// Clear the graph (remove all nodes and connections)
    void clear();

    /// Get total signal path latency in samples for the graph's main path.
    /// For now, this is a simple sum of all node latencies on the main chain.
    /// TODO: Future implementation should account for parallel paths and take maximum,
    ///       and should only consider nodes on the actual signal path (not all nodes).
    /// Real-time safe: read-only, no allocations or locks.
    int getTotalLatencyInSamples() const noexcept;

    /// Get simple upper bound on tail length for the entire graph.
    /// Returns the maximum tail length across all nodes.
    /// TODO: Future implementation should account for parallel paths and take maximum,
    ///       and should consider tail propagation through the graph.
    /// Real-time safe: read-only, no allocations or locks.
    int getMaxTailInSamples() const noexcept;

    /// Check if graph has active tails (plugins still producing output after playback stops)
    /// Real-time safe: read-only, no allocations or locks.
    /// For now, this is a stub that returns false. Future implementation should track
    /// actual tail state from plugins.
    bool hasActiveTails() const noexcept;

    /// Check if graph has live inputs or monitors that require processing even when transport is stopped
    /// Real-time safe: read-only atomic flag.
    /// Returns true if there are AudioInput or MidiInput nodes, or instrument nodes
    /// that receive live MIDI, requiring real-time processing even when stopped.
    bool hasLiveInputsOrMonitors() const noexcept;

    /// Set live inputs/monitors flag (called from control thread when graph changes)
    /// @param active true if graph has live input paths that need processing when stopped
    void setLiveInputsOrMonitorsActive(bool active) noexcept;

    /// Execute graph processing (called on audio thread)
    /// - Clears all node buffers
    /// - Routes connections (fan-in, with send levels)
    /// - Processes nodes in execution order
    /// Note: Source/Input Pass must be called separately before processGraph()
    void processGraph(EngineRenderContext& ctx);

    /// Run Source/Input Pass (called on audio thread before processGraph)
    /// - Injects schedule data into lane nodes (audio + MIDI)
    /// - Injects hardware input into input nodes (audio + MIDI)
    /// Real-time safe: no allocations, no locks, no logging
    void runSourceInputPass(
        const EngineRenderContext& ctx,
        const StreamScheduler* scheduler,
        AudioAssetSource* assetSource,
        const float* hardwareAudioInput,
        int hardwareAudioChannels,
        int hardwareAudioFrames,
        const std::vector<MidiMessage>& hardwareMidiInput
    );

private:
    /// Node factory - creates appropriate node subclass based on NodeKind
    std::unique_ptr<GraphNode> createNode(const NodeDesc& desc, PluginHost* pluginHost);

    /// Create NodeAudioConfig from NodeDesc
    /// Applies node-type-specific defaults and validates channel counts
    NodeAudioConfig createAudioConfigFromDesc(const NodeDesc& desc, GraphNode* node);

    /// Compute execution order using topological sort (Kahn's algorithm)
    void computeExecutionOrder();

    /// Build adjacency list for topological sort
    void buildAdjacencyList();

    /// Build incoming connections map (pre-computed for efficient routing)
    void buildIncomingConnections();

    /// Channel compatibility check result
    struct ChannelCompatibility {
        int sourceChannels;
        int destChannels;
        bool isCompatible;    // exact match
        bool isMismatch;      // non-zero and !=
    };

    /// Check channel compatibility between two nodes
    /// @param source Source node
    /// @param dest Destination node
    /// @return Compatibility result
    static ChannelCompatibility checkChannelCompatibility(
        const GraphNode& source,
        const GraphNode& dest
    );

    /// Validate routing rules for all connections
    /// Checks channel compatibility and marks invalid connections
    /// Called after graph construction and channel config assignment
    void validateRouting();

    /// Nodes keyed by ID
    std::unordered_map<NodeId, std::unique_ptr<GraphNode>> _nodes;

    /// Connections between nodes
    std::vector<GraphConnection> _connections;

    /// Stream input bindings
    std::vector<StreamInputBinding> _streamBindings;

    /// Execution order (topologically sorted)
    std::vector<GraphNode*> _executionOrder;

    /// Pre-computed incoming connections per node (for efficient routing)
    /// Maps nodeId -> list of connections that feed into this node
    std::unordered_map<NodeId, std::vector<GraphConnection>> _incomingConnections;

    /// Adjacency list for topological sort (nodeId -> list of connected node IDs)
    std::unordered_map<NodeId, std::vector<NodeId>> _adjacencyList;

    /// In-degree map for topological sort (nodeId -> number of incoming edges)
    std::unordered_map<NodeId, uint32_t> _inDegree;

    /// Plugin host (for creating plugin instances)
    PluginHost* _pluginHost;

    /// Live inputs/monitors flag (updated on control thread, read on audio thread)
    /// True if graph has AudioInput, MidiInput nodes, or instrument nodes that need
    /// real-time processing even when transport is stopped.
    std::atomic<bool> _hasLiveInputsOrMonitors;
};
