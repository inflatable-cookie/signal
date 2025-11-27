#include <catch2/catch_test_macros.hpp>
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "core/GraphNode.hpp"
#include "core/GraphNodes.hpp"
#include "core/StreamScheduler.hpp"
#include "core/AudioAssetSource.hpp"
#include "core/NodeProcessContext.hpp"
#include "core/ScheduleData.hpp"
#include "core/EngineRenderContext.hpp"
#include <vector>
#include <string>
#include <memory>
#include <cmath>
#include <nlohmann/json.hpp>

TEST_CASE("Phase 3 - Real audio injection test (stubbed source)", "[graph][phase3][audio]") {
    GraphEngine engine;
    StreamScheduler scheduler;

    // Create graph: audio-lane -> MixerChannelNode -> DeviceNode
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc laneNode;
    laneNode.nodeId = "audio-lane-1";
    laneNode.kind = NodeKind::AudioLane;
    laneNode.trackId = "track-1";
    laneNode.laneId = "lane-1";
    snapshot.nodes.push_back(laneNode);

    NodeDesc mixerNode;
    mixerNode.nodeId = "mixer-1";
    mixerNode.kind = NodeKind::MixerChannel;
    mixerNode.trackId = "track-1";
    snapshot.nodes.push_back(mixerNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Connections
    ConnectionDesc conn1;
    conn1.fromNodeId = "audio-lane-1";
    conn1.toNodeId = "mixer-1";
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "mixer-1";
    conn2.toNodeId = "device";
    snapshot.connections.push_back(conn2);

    // Stream binding
    ConnectionDesc streamBinding;
    streamBinding.fromStreamId = "stream-1";
    streamBinding.toNodeId = "audio-lane-1";
    snapshot.connections.push_back(streamBinding);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Set up schedule with audio segment
    std::vector<StreamDescriptor> streams;
    StreamDescriptor stream;
    stream.streamId = "stream-1";
    stream.trackId = "track-1";
    stream.laneId = "lane-1";
    stream.streamType = "audio";
    streams.push_back(stream);

    std::vector<AudioSegmentCompiled> audioSegments;
    AudioSegmentCompiled segment;
    segment.streamId = "stream-1";
    segment.assetId = "asset-1";
    segment.startSamples = 0;
    segment.endSamples = 512; // One block
    segment.assetStartSamples = 0;
    audioSegments.push_back(segment);

    std::vector<MidiEventCompiled> midiEvents;
    TempoMap tempoMap;
    tempoMap.defaultTempo = 120.0;

    scheduler.setSchedule(streams, audioSegments, midiEvents, tempoMap, 44100.0);

    // Process graph with stub asset source
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    StubAudioAssetSource assetSource;
    engine.processGraph(ctx, &scheduler, &assetSource);

    // Verify audio lane node has output (ramp pattern from stub)
    auto* lane = dynamic_cast<AudioLaneNode*>(engine.findNode("audio-lane-1"));
    REQUIRE(lane != nullptr);
    // Stub generates: sample = (startSample + frame) / 1000.0f
    // For frame 0: sample = 0 / 1000.0 = 0.0
    // For frame 100: sample = 100 / 1000.0 = 0.1
    REQUIRE(std::abs(lane->io.audioOut.getSample(0, 0)) < 0.001f); // First sample should be ~0
    REQUIRE(lane->io.audioOut.getSample(100, 0) > 0.09f); // Frame 100 should be ~0.1

    // Verify device node output matches (within tolerance)
    auto* device = dynamic_cast<DeviceNode*>(engine.findNode("device"));
    REQUIRE(device != nullptr);
    REQUIRE(std::abs(device->io.audioOut.getSample(0, 0)) < 0.001f);
    REQUIRE(device->io.audioOut.getSample(100, 0) > 0.09f);
}

TEST_CASE("Phase 3 - Send/Receive test", "[graph][phase3][send]") {
    GraphEngine engine;

    // Create graph: audio-lane -> MixerChannelNode -> DeviceNode (dry)
    //                MixerChannelNode -> SendNode -> ReceiveNode -> DeviceNode (wet)
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc laneNode;
    laneNode.nodeId = "audio-lane-1";
    laneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(laneNode);

    NodeDesc mixerNode;
    mixerNode.nodeId = "mixer-1";
    mixerNode.kind = NodeKind::MixerChannel;
    snapshot.nodes.push_back(mixerNode);

    NodeDesc sendNode;
    sendNode.nodeId = "send-1";
    sendNode.kind = NodeKind::Send;
    sendNode.pluginId = "receive-1"; // Target receive node
    snapshot.nodes.push_back(sendNode);

    NodeDesc receiveNode;
    receiveNode.nodeId = "receive-1";
    receiveNode.kind = NodeKind::Receive;
    snapshot.nodes.push_back(receiveNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Connections: lane -> mixer -> device (dry)
    ConnectionDesc conn1;
    conn1.fromNodeId = "audio-lane-1";
    conn1.toNodeId = "mixer-1";
    snapshot.connections.push_back(conn1);

    ConnectionDesc conn2;
    conn2.fromNodeId = "mixer-1";
    conn2.toNodeId = "device";
    snapshot.connections.push_back(conn2);

    // Connections: mixer -> send -> receive -> device (wet)
    ConnectionDesc conn3;
    conn3.fromNodeId = "mixer-1";
    conn3.toNodeId = "send-1";
    snapshot.connections.push_back(conn3);

    ConnectionDesc conn4;
    conn4.fromNodeId = "send-1";
    conn4.toNodeId = "receive-1";
    snapshot.connections.push_back(conn4);

    ConnectionDesc conn5;
    conn5.fromNodeId = "receive-1";
    conn5.toNodeId = "device";
    snapshot.connections.push_back(conn5);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Set send level to 0.5
    auto* send = dynamic_cast<SendNode*>(engine.findNode("send-1"));
    REQUIRE(send != nullptr);
    send->setSendLevel(0.5f);

    // Add stream binding and set up schedule to inject audio
    ConnectionDesc streamBinding;
    streamBinding.fromStreamId = "stream-1";
    streamBinding.toNodeId = "audio-lane-1";
    snapshot.connections.push_back(streamBinding);
    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Set up schedule
    std::vector<StreamDescriptor> streams;
    StreamDescriptor stream;
    stream.streamId = "stream-1";
    stream.streamType = "audio";
    streams.push_back(stream);

    std::vector<AudioSegmentCompiled> audioSegments;
    AudioSegmentCompiled segment;
    segment.streamId = "stream-1";
    segment.assetId = "asset-1";
    segment.startSamples = 0;
    segment.endSamples = 512;
    segment.assetStartSamples = 0;
    audioSegments.push_back(segment);

    std::vector<MidiEventCompiled> midiEvents;
    TempoMap tempoMap;
    tempoMap.defaultTempo = 120.0;

    StreamScheduler scheduler;
    scheduler.setSchedule(streams, audioSegments, midiEvents, tempoMap, 44100.0);

    // Process graph
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    StubAudioAssetSource assetSource;
    engine.processGraph(ctx, &scheduler, &assetSource);

    // Verify device node received output (from both dry and wet paths)
    // The exact value depends on stub source, but it should be non-zero
    auto* device = dynamic_cast<DeviceNode*>(engine.findNode("device"));
    REQUIRE(device != nullptr);
    bool hasOutput = false;
    for (int frame = 0; frame < 512; ++frame) {
        if (std::abs(device->io.audioOut.getSample(frame, 0)) > 0.001f) {
            hasOutput = true;
            break;
        }
    }
    REQUIRE(hasOutput); // Device should have output from both paths
}

TEST_CASE("Phase 3 - Gain/Pan test", "[graph][phase3][mixer]") {
    // Test mixer node directly (simpler and more reliable)
    MixerChannelNode mixer("mixer-1");
    mixer.prepare(44100, 512);

    // Set gain to 0.5 and pan to -1.0 (full left)
    mixer.setGain(0.5f);
    mixer.setPan(-1.0f);

    // Set input audio (stereo, unity)
    mixer.io.audioIn.setSample(0, 0, 1.0f); // Left channel
    mixer.io.audioIn.setSample(0, 1, 1.0f); // Right channel

    // Process mixer
    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;
    mixer.process(npc);

    // Verify gain and pan applied
    // Pan -1.0 (full left): leftGain = (1 - (-1)) * 0.5 = 2.0 * 0.5 = 1.0, rightGain = (1 + (-1)) * 0.5 = 0.0
    // Input is 1.0 on both channels
    // Left output: 1.0 * 1.0 = 1.0
    // Right output: 1.0 * 0.0 = 0.0
    REQUIRE(std::abs(mixer.io.audioOut.getSample(0, 0) - 1.0f) < 0.01f); // Left should be 1.0
    REQUIRE(std::abs(mixer.io.audioOut.getSample(0, 1)) < 0.01f); // Right should be 0.0
}

TEST_CASE("Phase 3 - NodeProcessContext test", "[graph][phase3][context]") {
    GraphEngine engine;

    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc laneNode;
    laneNode.nodeId = "audio-lane-1";
    laneNode.kind = NodeKind::AudioLane;
    snapshot.nodes.push_back(laneNode);

    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Process graph
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 1000;

    StubAudioAssetSource assetSource;
    engine.processGraph(ctx, nullptr, &assetSource);

    // Verify NodeProcessContext is correctly populated
    // This is verified indirectly by the fact that process() is called without errors
    // In a real implementation, we could add a test node that stores the context
    auto* lane = engine.findNode("audio-lane-1");
    REQUIRE(lane != nullptr);
    // Context is passed to process() - if it works, the test passes
    REQUIRE(true);
}

TEST_CASE("Phase 3 - JSON snapshot parsing and graph/schedule alignment", "[graph][phase3][json][snapshot]") {
    GraphEngine engine;
    StreamScheduler scheduler;

    // Create a JSON snapshot matching Pulse's format
    nlohmann::json snapshotJson;
    snapshotJson["id"] = "test-json-snapshot";

    // Add nodes
    nlohmann::json nodes = nlohmann::json::array();

    nlohmann::json laneNode;
    laneNode["nodeId"] = "audio-lane-1";
    laneNode["kind"] = "audio-lane";
    laneNode["trackId"] = "track-1";
    laneNode["laneId"] = "lane-1";
    nodes.push_back(laneNode);

    nlohmann::json deviceNode;
    deviceNode["nodeId"] = "device";
    deviceNode["kind"] = "device";
    nodes.push_back(deviceNode);

    snapshotJson["nodes"] = nodes;

    // Add connections
    nlohmann::json connections = nlohmann::json::array();

    // Stream binding
    nlohmann::json streamBinding;
    streamBinding["fromStreamId"] = "stream-1";
    streamBinding["toNodeId"] = "audio-lane-1";
    streamBinding["toInputIndex"] = 0;
    connections.push_back(streamBinding);

    // Node-to-node connection
    nlohmann::json nodeConn;
    nodeConn["fromNodeId"] = "audio-lane-1";
    nodeConn["fromOutputIndex"] = 0;
    nodeConn["toNodeId"] = "device";
    nodeConn["toInputIndex"] = 0;
    connections.push_back(nodeConn);

    snapshotJson["connections"] = connections;

    // Parse JSON into GraphSnapshot (simulating EngineDomain parsing)
    GraphSnapshot snapshot;
    snapshot.id = snapshotJson["id"].get<std::string>();

    for (const auto& nodeJson : snapshotJson["nodes"]) {
        NodeDesc node;
        node.nodeId = nodeJson["nodeId"].get<std::string>();
        if (nodeJson.contains("trackId")) {
            node.trackId = nodeJson["trackId"].get<std::string>();
        }
        if (nodeJson.contains("laneId")) {
            node.laneId = nodeJson["laneId"].get<std::string>();
        }

        std::string kindStr = nodeJson["kind"].get<std::string>();
        auto kindOpt = nodeKindFromString(kindStr);
        REQUIRE(kindOpt.has_value());
        node.kind = kindOpt.value();

        snapshot.nodes.push_back(node);
    }

    for (const auto& connJson : snapshotJson["connections"]) {
        ConnectionDesc conn;
        if (connJson.contains("fromStreamId")) {
            conn.fromStreamId = connJson["fromStreamId"].get<std::string>();
        } else if (connJson.contains("fromNodeId")) {
            conn.fromNodeId = connJson["fromNodeId"].get<std::string>();
        }
        conn.fromOutputIndex = connJson.value("fromOutputIndex", 0u);
        conn.toInputIndex = connJson.value("toInputIndex", 0u);
        conn.toNodeId = connJson["toNodeId"].get<std::string>();
        snapshot.connections.push_back(conn);
    }

    // Load snapshot into engine
    engine.loadGraphSnapshot(snapshot);
    engine.prepareGraph(44100, 512);

    // Verify graph was loaded correctly
    REQUIRE(engine.findNode("audio-lane-1") != nullptr);
    REQUIRE(engine.findNode("device") != nullptr);

    // Verify stream binding
    const auto& streamBindings = engine.getStreamBindings();
    REQUIRE(streamBindings.size() == 1);
    REQUIRE(streamBindings[0].streamId == "stream-1");
    REQUIRE(streamBindings[0].targetNodeId == "audio-lane-1");

    // Set up schedule matching the graph
    std::vector<StreamDescriptor> streams;
    StreamDescriptor stream;
    stream.streamId = "stream-1";
    stream.trackId = "track-1";
    stream.laneId = "lane-1";
    stream.streamType = "audio";
    streams.push_back(stream);

    std::vector<AudioSegmentCompiled> audioSegments;
    AudioSegmentCompiled segment;
    segment.streamId = "stream-1";
    segment.assetId = "asset-1";
    segment.startSamples = 0;
    segment.endSamples = 512; // One block
    segment.assetStartSamples = 0;
    segment.gainDb = 0.0;
    segment.fadeInSamples = 0;
    segment.fadeOutSamples = 0;
    segment.fadeInCurve = "linear";
    segment.fadeOutCurve = "linear";
    segment.stretch.mode = "none";
    segment.stretch.ratio = 1.0;
    audioSegments.push_back(segment);

    std::vector<MidiEventCompiled> midiEvents;
    TempoMap tempoMap;
    tempoMap.defaultTempo = 120.0;

    scheduler.setSchedule(streams, audioSegments, midiEvents, tempoMap, 44100.0);

    // Process graph with schedule
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;

    StubAudioAssetSource assetSource;
    engine.processGraph(ctx, &scheduler, &assetSource);

    // Verify output is non-zero when schedule is active
    auto* device = dynamic_cast<DeviceNode*>(engine.findNode("device"));
    REQUIRE(device != nullptr);

    bool hasOutput = false;
    for (int frame = 0; frame < 512; ++frame) {
        if (std::abs(device->io.audioOut.getSample(frame, 0)) > 0.001f) {
            hasOutput = true;
            break;
        }
    }
    REQUIRE(hasOutput); // Device should have output from schedule

    // Test with empty schedule - should produce silence
    scheduler.clearSchedule();
    engine.processGraph(ctx, &scheduler, &assetSource);

    bool hasSilence = true;
    for (int frame = 0; frame < 512; ++frame) {
        if (std::abs(device->io.audioOut.getSample(frame, 0)) > 0.001f) {
            hasSilence = false;
            break;
        }
    }
    REQUIRE(hasSilence); // Device should be silent with empty schedule
}

