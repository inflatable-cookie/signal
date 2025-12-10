#include <catch2/catch_test_macros.hpp>
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphNodes.hpp"

TEST_CASE("Phase 7 - Subgroup routing structure", "[graph][phase7][routing]") {
    GraphEngine engine;

    // Build a simple subgroup:
    // Track 1 -> mixer-track-1 -> mixer-bus -> device
    // Track 2 -> mixer-track-2 /
    GraphSnapshot snapshot;
    snapshot.id = "test-subgroup";

    // Nodes
    NodeDesc mixerTrack1;
    mixerTrack1.nodeId = "mixer-track-1";
    mixerTrack1.kind = NodeKind::MixerChannel;
    snapshot.nodes.push_back(mixerTrack1);

    NodeDesc mixerTrack2;
    mixerTrack2.nodeId = "mixer-track-2";
    mixerTrack2.kind = NodeKind::MixerChannel;
    snapshot.nodes.push_back(mixerTrack2);

    NodeDesc mixerBus;
    mixerBus.nodeId = "mixer-bus";
    mixerBus.kind = NodeKind::MixerChannel;
    snapshot.nodes.push_back(mixerBus);

    NodeDesc device;
    device.nodeId = "device";
    device.kind = NodeKind::Device;
    snapshot.nodes.push_back(device);

    // Connections: tracks -> bus -> device
    ConnectionDesc t1ToBus;
    t1ToBus.fromNodeId = "mixer-track-1";
    t1ToBus.toNodeId = "mixer-bus";
    t1ToBus.fromOutputIndex = 0;
    t1ToBus.toInputIndex = 0;
    snapshot.connections.push_back(t1ToBus);

    ConnectionDesc t2ToBus;
    t2ToBus.fromNodeId = "mixer-track-2";
    t2ToBus.toNodeId = "mixer-bus";
    t2ToBus.fromOutputIndex = 0;
    t2ToBus.toInputIndex = 0;
    snapshot.connections.push_back(t2ToBus);

    ConnectionDesc busToDevice;
    busToDevice.fromNodeId = "mixer-bus";
    busToDevice.toNodeId = "device";
    busToDevice.fromOutputIndex = 0;
    busToDevice.toInputIndex = 0;
    snapshot.connections.push_back(busToDevice);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Verify nodes exist
    REQUIRE(engine.findNode("mixer-track-1") != nullptr);
    REQUIRE(engine.findNode("mixer-track-2") != nullptr);
    REQUIRE(engine.findNode("mixer-bus") != nullptr);
    REQUIRE(engine.findNode("device") != nullptr);

    // Adjacency and execution order are validated implicitly by prepareGraph()
}

