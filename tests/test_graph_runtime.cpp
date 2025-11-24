#include <catch2/catch_test_macros.hpp>
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphNode.hpp"
#include "core/GraphNodes.hpp"
#include <vector>
#include <string>

TEST_CASE("GraphEngine - Node creation", "[graph][node]") {
    GraphEngine engine;

    // Create a simple graph snapshot with various node types
    GraphSnapshot snapshot;
    snapshot.id = "test-graph-1";

    // Add nodes
    NodeDesc midiLaneNode;
    midiLaneNode.nodeId = "midi-lane-1";
    midiLaneNode.kind = NodeKind::MidiLane;
    midiLaneNode.trackId = "track-1";
    midiLaneNode.laneId = "lane-1";
    snapshot.nodes.push_back(midiLaneNode);

    NodeDesc audioLaneNode;
    audioLaneNode.nodeId = "audio-lane-1";
    audioLaneNode.kind = NodeKind::AudioLane;
    audioLaneNode.trackId = "track-1";
    audioLaneNode.laneId = "lane-2";
    snapshot.nodes.push_back(audioLaneNode);

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    fxNode.trackId = "track-1";
    fxNode.pluginId = "compressor-vst";
    snapshot.nodes.push_back(fxNode);

    NodeDesc masterNode;
    masterNode.nodeId = "master";
    masterNode.kind = NodeKind::Master;
    snapshot.nodes.push_back(masterNode);

    // Load snapshot
    engine.loadGraphSnapshot(snapshot);

    // Verify all nodes were created
    REQUIRE(engine.findNode("midi-lane-1") != nullptr);
    REQUIRE(engine.findNode("audio-lane-1") != nullptr);
    REQUIRE(engine.findNode("fx-1") != nullptr);
    REQUIRE(engine.findNode("master") != nullptr);

    // Verify node types
    REQUIRE(engine.findNode("midi-lane-1")->getKind() == NodeKind::MidiLane);
    REQUIRE(engine.findNode("audio-lane-1")->getKind() == NodeKind::AudioLane);
    REQUIRE(engine.findNode("fx-1")->getKind() == NodeKind::AudioFx);
    REQUIRE(engine.findNode("master")->getKind() == NodeKind::Master);

    // Verify metadata
    REQUIRE(engine.findNode("midi-lane-1")->getTrackId() == "track-1");
    REQUIRE(engine.findNode("midi-lane-1")->getLaneId() == "lane-1");
}

TEST_CASE("GraphEngine - Connections and stream bindings", "[graph][connection]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "test-graph-2";

    // Add nodes
    NodeDesc audioLaneNode;
    audioLaneNode.nodeId = "audio-lane-1";
    audioLaneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(audioLaneNode);

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(fxNode);

    NodeDesc masterNode;
    masterNode.nodeId = "master";
    masterNode.kind = NodeKind::Master;
    snapshot.nodes.push_back(masterNode);

    // Add stream binding (stream -> lane node)
    ConnectionDesc streamBinding;
    streamBinding.fromStreamId = "stream-1";
    streamBinding.toNodeId = "audio-lane-1";
    streamBinding.toInputIndex = 0;
    snapshot.connections.push_back(streamBinding);

    // Add node-to-node connection (lane -> fx)
    ConnectionDesc nodeConn;
    nodeConn.fromNodeId = "audio-lane-1";
    nodeConn.fromOutputIndex = 0;
    nodeConn.toNodeId = "fx-1";
    nodeConn.toInputIndex = 0;
    snapshot.connections.push_back(nodeConn);

    // Add another connection (fx -> master)
    ConnectionDesc masterConn;
    masterConn.fromNodeId = "fx-1";
    masterConn.fromOutputIndex = 0;
    masterConn.toNodeId = "master";
    masterConn.toInputIndex = 0;
    snapshot.connections.push_back(masterConn);

    // Load snapshot
    engine.loadGraphSnapshot(snapshot);

    // Verify stream bindings
    const auto& streamBindings = engine.getStreamBindings();
    REQUIRE(streamBindings.size() == 1);
    REQUIRE(streamBindings[0].streamId == "stream-1");
    REQUIRE(streamBindings[0].targetNodeId == "audio-lane-1");
    REQUIRE(streamBindings[0].targetInputIndex == 0);
}

TEST_CASE("GraphEngine - Execution order (topological sort)", "[graph][execution]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "test-graph-3";

    // Create a simple chain: lane -> fx -> master
    NodeDesc laneNode;
    laneNode.nodeId = "lane-1";
    laneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(laneNode);

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(fxNode);

    NodeDesc masterNode;
    masterNode.nodeId = "master";
    masterNode.kind = NodeKind::Master;
    snapshot.nodes.push_back(masterNode);

    // Connect: lane -> fx -> master
    ConnectionDesc conn1;
    conn1.fromNodeId = "lane-1";
    conn1.fromOutputIndex = 0;
    conn1.toNodeId = "fx-1";
    conn1.toInputIndex = 0;
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "fx-1";
    conn2.fromOutputIndex = 0;
    conn2.toNodeId = "master";
    conn2.toInputIndex = 0;
    snapshot.connections.push_back(conn2);

    // Load snapshot
    engine.loadGraphSnapshot(snapshot);

    // Verify execution order
    const auto& executionOrder = engine.getExecutionOrder();
    REQUIRE(executionOrder.size() == 3);

    // Find positions
    size_t lanePos = 0, fxPos = 0, masterPos = 0;
    for (size_t i = 0; i < executionOrder.size(); ++i) {
        if (executionOrder[i]->getId() == "lane-1") lanePos = i;
        if (executionOrder[i]->getId() == "fx-1") fxPos = i;
        if (executionOrder[i]->getId() == "master") masterPos = i;
    }

    // Lane should come before fx, fx should come before master
    REQUIRE(lanePos < fxPos);
    REQUIRE(fxPos < masterPos);
}

TEST_CASE("GraphEngine - Cycle detection", "[graph][cycle]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "test-graph-cycle";

    // Create nodes
    NodeDesc node1;
    node1.nodeId = "node-1";
    node1.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(node1);

    NodeDesc node2;
    node2.nodeId = "node-2";
    node2.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(node2);

    // Create cycle: node1 -> node2 -> node1
    ConnectionDesc conn1;
    conn1.fromNodeId = "node-1";
    conn1.fromOutputIndex = 0;
    conn1.toNodeId = "node-2";
    conn1.toInputIndex = 0;
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "node-2";
    conn2.fromOutputIndex = 0;
    conn2.toNodeId = "node-1";
    conn2.toInputIndex = 0;
    snapshot.connections.push_back(conn2);

    // Load snapshot (should handle cycle gracefully)
    engine.loadGraphSnapshot(snapshot);

    // Execution order should still include all nodes (fallback ordering)
    const auto& executionOrder = engine.getExecutionOrder();
    REQUIRE(executionOrder.size() == 2);
}

TEST_CASE("GraphEngine - Prepare graph", "[graph][prepare]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "test-graph-prepare";

    NodeDesc laneNode;
    laneNode.nodeId = "lane-1";
    laneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(laneNode);

    engine.loadGraphSnapshot(snapshot);

    // Prepare should not crash
    engine.prepareGraph(44100, 512);
    REQUIRE(true); // If we get here, prepare succeeded
}

TEST_CASE("GraphEngine - Empty graph", "[graph][empty]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "empty-graph";

    engine.loadGraphSnapshot(snapshot);

    REQUIRE(engine.getExecutionOrder().size() == 0);
    REQUIRE(engine.getStreamBindings().size() == 0);
}

