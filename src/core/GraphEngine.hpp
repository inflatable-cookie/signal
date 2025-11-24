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
#include <memory>
#include <unordered_map>
#include <vector>
#include <string>

/// Connection between node ports
struct GraphConnection {
    NodeId fromNodeId;
    uint32_t fromOutputIndex;
    NodeId toNodeId;
    uint32_t toInputIndex;
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
    void loadGraphSnapshot(const GraphSnapshot& snapshot);

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

    /// Clear the graph (remove all nodes and connections)
    void clear();

private:
    /// Node factory - creates appropriate node subclass based on NodeKind
    std::unique_ptr<GraphNode> createNode(const NodeDesc& desc);

    /// Compute execution order using topological sort (Kahn's algorithm)
    void computeExecutionOrder();

    /// Build adjacency list for topological sort
    void buildAdjacencyList();

    /// Nodes keyed by ID
    std::unordered_map<NodeId, std::unique_ptr<GraphNode>> _nodes;

    /// Connections between nodes
    std::vector<GraphConnection> _connections;

    /// Stream input bindings
    std::vector<StreamInputBinding> _streamBindings;

    /// Execution order (topologically sorted)
    std::vector<GraphNode*> _executionOrder;

    /// Adjacency list for topological sort (nodeId -> list of connected node IDs)
    std::unordered_map<NodeId, std::vector<NodeId>> _adjacencyList;

    /// In-degree map for topological sort (nodeId -> number of incoming edges)
    std::unordered_map<NodeId, uint32_t> _inDegree;
};

