#include <catch2/catch_test_macros.hpp>
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphNode.hpp"
#include "core/GraphNodes.hpp"
#include "core/StreamScheduler.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/NodeProcessContext.hpp"
#include "core/AudioAssetSource.hpp"
#include "core/AudioBus.hpp"
#include <vector>
#include <string>
#include <memory>

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

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Load snapshot
    engine.loadGraphSnapshot(snapshot);

    // Verify all nodes were created
    REQUIRE(engine.findNode("midi-lane-1") != nullptr);
    REQUIRE(engine.findNode("audio-lane-1") != nullptr);
    REQUIRE(engine.findNode("fx-1") != nullptr);
    REQUIRE(engine.findNode("device") != nullptr);

    // Verify node types
    REQUIRE(engine.findNode("midi-lane-1")->getKind() == NodeKind::MidiLane);
    REQUIRE(engine.findNode("audio-lane-1")->getKind() == NodeKind::AudioLane);
    REQUIRE(engine.findNode("fx-1")->getKind() == NodeKind::AudioFx);
    REQUIRE(engine.findNode("device")->getKind() == NodeKind::Device);

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

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

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

    // Add another connection (fx -> device)
    ConnectionDesc deviceConn;
    deviceConn.fromNodeId = "fx-1";
    deviceConn.fromOutputIndex = 0;
    deviceConn.toNodeId = "device";
    deviceConn.toInputIndex = 0;
    snapshot.connections.push_back(deviceConn);

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

    // Create a simple chain: lane -> fx -> device
    NodeDesc laneNode;
    laneNode.nodeId = "lane-1";
    laneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(laneNode);

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(fxNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Connect: lane -> fx -> device
    ConnectionDesc conn1;
    conn1.fromNodeId = "lane-1";
    conn1.fromOutputIndex = 0;
    conn1.toNodeId = "fx-1";
    conn1.toInputIndex = 0;
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "fx-1";
    conn2.fromOutputIndex = 0;
    conn2.toNodeId = "device";
    conn2.toInputIndex = 0;
    snapshot.connections.push_back(conn2);

    // Load snapshot
    engine.loadGraphSnapshot(snapshot);

    // Verify execution order
    const auto& executionOrder = engine.getExecutionOrder();
    REQUIRE(executionOrder.size() == 3);

    // Find positions
    size_t lanePos = 0, fxPos = 0, devicePos = 0;
    for (size_t i = 0; i < executionOrder.size(); ++i) {
        if (executionOrder[i]->getId() == "lane-1") lanePos = i;
        if (executionOrder[i]->getId() == "fx-1") fxPos = i;
        if (executionOrder[i]->getId() == "device") devicePos = i;
    }

    // Lane should come before fx, fx should come before device
    REQUIRE(lanePos < fxPos);
    REQUIRE(fxPos < devicePos);
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

TEST_CASE("GraphEngine - Stream injection into lane node", "[graph][stream-injection]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "stream-test";

    // Create audio lane node
    NodeDesc laneNode;
    laneNode.nodeId = "audio-lane-1";
    laneNode.kind = NodeKind::AudioLane;
    laneNode.trackId = "track-1";
    laneNode.laneId = "lane-1";
    snapshot.nodes.push_back(laneNode);

    // Create device node
    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Stream binding: stream-1 -> audio-lane-1
    ConnectionDesc streamBinding;
    streamBinding.fromStreamId = "stream-1";
    streamBinding.toNodeId = "audio-lane-1";
    streamBinding.toInputIndex = 0;
    snapshot.connections.push_back(streamBinding);

    // Connection: audio-lane-1 -> device
    ConnectionDesc conn;
    conn.fromNodeId = "audio-lane-1";
    conn.fromOutputIndex = 0;
    conn.toNodeId = "device";
    conn.toInputIndex = 0;
    snapshot.connections.push_back(conn);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Create schedule with audio segment
    StreamScheduler scheduler;
    std::vector<StreamDescriptor> streams;
    StreamDescriptor streamDesc;
    streamDesc.streamId = "stream-1";
    streamDesc.trackId = "track-1";
    streamDesc.laneId = "lane-1";
    streamDesc.streamType = "audio";
    streams.push_back(streamDesc);

    std::vector<AudioSegmentCompiled> audioSegments;
    AudioSegmentCompiled segment;
    segment.streamId = "stream-1";
    segment.assetId = "asset-1";
    segment.startSamples = 0;
    segment.endSamples = 88200; // 2 seconds at 44.1kHz
    segment.assetStartSamples = 0;
    audioSegments.push_back(segment);

    std::vector<MidiEventCompiled> midiEvents;
    TempoMap tempoMap;
    tempoMap.defaultTempo = 120.0;

    scheduler.setSchedule(streams, audioSegments, midiEvents, tempoMap, 44100.0);

    // Process graph
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    StubAudioAssetSource assetSource;
    engine.processGraph(ctx, &scheduler, &assetSource);

    // Verify lane node received stream
    auto* lane = dynamic_cast<AudioLaneNode*>(engine.findNode("audio-lane-1"));
    REQUIRE(lane != nullptr);
    REQUIRE(lane->getStreamId() == "stream-1");

    // Verify device node has output (pass-through from lane)
    auto* device = dynamic_cast<DeviceNode*>(engine.findNode("device"));
    REQUIRE(device != nullptr);
    // Device should have audio output (even if test tone)
    REQUIRE(device->io.audioOut.numFrames() > 0);
}

TEST_CASE("GraphEngine - Node pass-through", "[graph][pass-through]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "passthrough-test";

    // Create chain: lane -> fx -> device
    NodeDesc laneNode;
    laneNode.nodeId = "lane-1";
    laneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(laneNode);

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(fxNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Connections
    ConnectionDesc conn1;
    conn1.fromNodeId = "lane-1";
    conn1.toNodeId = "fx-1";
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "fx-1";
    conn2.toNodeId = "device";
    snapshot.connections.push_back(conn2);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Process graph (no scheduler - just test pass-through)
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    // Manually set some audio in lane output AFTER clear but before routing
    // We'll do this by processing, then manually setting, then processing again
    // Or better: set it, then manually route, then process
    auto* lane = engine.findNode("lane-1");
    REQUIRE(lane != nullptr);

    // Set audio in lane output (simulating lane node generating audio)
    lane->io.audioOut.setSample(0, 0, 0.5f);
    lane->io.audioOut.setSample(0, 1, 0.5f);

    // Manually route connection (simulating what processGraph does)
    auto* fx = engine.findNode("fx-1");
    REQUIRE(fx != nullptr);
    fx->io.audioIn.sumFrom(lane->io.audioOut);

    // Process FX node
    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;
    fx->process(npc);

    // Verify FX received and passed through audio
    REQUIRE(fx->io.audioIn.getSample(0, 0) == 0.5f);
    REQUIRE(fx->io.audioOut.getSample(0, 0) == 0.5f);

    // Route to device
    auto* device = engine.findNode("device");
    REQUIRE(device != nullptr);
    device->io.audioIn.sumFrom(fx->io.audioOut);
    device->process(npc);

    // Verify device received audio
    REQUIRE(device->io.audioIn.getSample(0, 0) == 0.5f);
    REQUIRE(device->io.audioOut.getSample(0, 0) == 0.5f);
}

TEST_CASE("GraphEngine - MIDI routing", "[graph][midi]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "midi-test";

    // Create chain: midi-lane -> midi-fx -> instrument -> device
    NodeDesc midiLaneNode;
    midiLaneNode.nodeId = "midi-lane-1";
    midiLaneNode.kind = NodeKind::MidiLane;
    snapshot.nodes.push_back(midiLaneNode);

    NodeDesc midiFxNode;
    midiFxNode.nodeId = "midi-fx-1";
    midiFxNode.kind = NodeKind::MidiFx;
    snapshot.nodes.push_back(midiFxNode);

    NodeDesc instrumentNode;
    instrumentNode.nodeId = "instrument-1";
    instrumentNode.kind = NodeKind::Instrument;
    snapshot.nodes.push_back(instrumentNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Connections
    ConnectionDesc conn1;
    conn1.fromNodeId = "midi-lane-1";
    conn1.toNodeId = "midi-fx-1";
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "midi-fx-1";
    conn2.toNodeId = "instrument-1";
    snapshot.connections.push_back(conn2);

    ConnectionDesc conn3;
    conn3.fromNodeId = "instrument-1";
    conn3.toNodeId = "device";
    snapshot.connections.push_back(conn3);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    // Manually add MIDI to lane output (simulating lane node generating MIDI)
    auto* midiLane = dynamic_cast<MidiLaneNode*>(engine.findNode("midi-lane-1"));
    REQUIRE(midiLane != nullptr);
    MidiMessage msg;
    msg.status = 0x90; // Note on
    msg.data1 = 60; // C4
    msg.data2 = 100; // Velocity
    msg.channel = 0;
    msg.sampleOffset = 0;
    midiLane->io.midiOut.addMessage(msg);

    // Manually route connection (simulating what processGraph does)
    auto* midiFx = engine.findNode("midi-fx-1");
    REQUIRE(midiFx != nullptr);
    midiFx->io.midiIn.append(midiLane->io.midiOut);

    // Verify MIDI flowed to FX input
    REQUIRE(midiFx->io.midiIn.size() == 1);

    // Process FX (pass-through)
    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;
    midiFx->process(npc);
    REQUIRE(midiFx->io.midiOut.size() == 1);

    // Route to instrument
    auto* instrument = engine.findNode("instrument-1");
    REQUIRE(instrument != nullptr);
    instrument->io.midiIn.append(midiFx->io.midiOut);

    // Verify MIDI flowed to instrument
    REQUIRE(instrument->io.midiIn.size() == 1);
}

TEST_CASE("GraphEngine - Fan-in (multiple inputs)", "[graph][fan-in]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "fanin-test";

    // Create two lanes feeding one FX
    NodeDesc lane1;
    lane1.nodeId = "lane-1";
    lane1.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(lane1);

    NodeDesc lane2;
    lane2.nodeId = "lane-2";
    lane2.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(lane2);

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    snapshot.nodes.push_back(fxNode);

    // Connections: both lanes -> fx
    ConnectionDesc conn1;
    conn1.fromNodeId = "lane-1";
    conn1.toNodeId = "fx-1";
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "lane-2";
    conn2.toNodeId = "fx-1";
    snapshot.connections.push_back(conn2);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    // Set different values in each lane output (simulating lane nodes generating audio)
    auto* lane1Node = engine.findNode("lane-1");
    auto* lane2Node = engine.findNode("lane-2");
    REQUIRE(lane1Node != nullptr);
    REQUIRE(lane2Node != nullptr);

    lane1Node->io.audioOut.setSample(0, 0, 0.3f);
    lane1Node->io.audioOut.setSample(0, 1, 0.3f);
    lane2Node->io.audioOut.setSample(0, 0, 0.5f);
    lane2Node->io.audioOut.setSample(0, 1, 0.5f);

    // Manually route connections (simulating what processGraph does)
    auto* fx = engine.findNode("fx-1");
    REQUIRE(fx != nullptr);

    // Clear FX input first
    fx->io.audioIn.clear();

    // Sum both lanes into FX input
    fx->io.audioIn.sumFrom(lane1Node->io.audioOut);
    fx->io.audioIn.sumFrom(lane2Node->io.audioOut);

    // Verify FX received sum of both lanes
    REQUIRE(fx->io.audioIn.getSample(0, 0) == 0.8f); // 0.3 + 0.5
    REQUIRE(fx->io.audioIn.getSample(0, 1) == 0.8f);
}

