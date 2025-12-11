#include <catch2/catch_test_macros.hpp>
#include "core/EngineHost.hpp"
#include "core/AutomationData.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/AudioBus.hpp"
#include "core/EngineRenderContext.hpp"
#include <vector>
#include <string>

TEST_CASE("AutomationData - Empty snapshot", "[automation]") {
    AutomationData data = AutomationData::empty();

    REQUIRE(data.events.empty());
    REQUIRE(data.tempoMap.defaultTempo == 120.0);
}

TEST_CASE("AutomationData - Event sorting", "[automation]") {
    AutomationData data;
    data.tempoMap.defaultTempo = 120.0;

    // Add events out of order
    AutomationEventCompiled event1;
    event1.nodeId = "node-1";
    event1.paramId = "gain";
    event1.timeSamples = 1000;
    event1.valueNorm = 0.5f;
    event1.curve = AutomationCurveType::Step;

    AutomationEventCompiled event2;
    event2.nodeId = "node-1";
    event2.paramId = "gain";
    event2.timeSamples = 500;
    event2.valueNorm = 0.25f;
    event2.curve = AutomationCurveType::Step;

    AutomationEventCompiled event3;
    event3.nodeId = "node-1";
    event3.paramId = "gain";
    event3.timeSamples = 2000;
    event3.valueNorm = 0.75f;
    event3.curve = AutomationCurveType::Step;

    data.events.push_back(event1);
    data.events.push_back(event2);
    data.events.push_back(event3);

    // Sort events
    std::sort(data.events.begin(), data.events.end(),
        [](const AutomationEventCompiled& a, const AutomationEventCompiled& b) {
            return a.timeSamples < b.timeSamples;
        });

    REQUIRE(data.events.size() == 3);
    REQUIRE(data.events[0].timeSamples == 500);
    REQUIRE(data.events[1].timeSamples == 1000);
    REQUIRE(data.events[2].timeSamples == 2000);
}

TEST_CASE("EngineHost - Load automation snapshot", "[automation]") {
    EngineHost host;

    AutomationData snapshot;
    snapshot.tempoMap.defaultTempo = 120.0;

    AutomationEventCompiled event;
    event.nodeId = "mixer-1";
    event.paramId = "gain";
    event.timeSamples = 0;
    event.valueNorm = 0.5f;
    event.curve = AutomationCurveType::Step;
    snapshot.events.push_back(event);

    host.loadAutomationSnapshot(snapshot);

    const AutomationData* loaded = host.getAutomationSnapshot();
    REQUIRE(loaded != nullptr);
    REQUIRE(loaded->events.size() == 1);
    REQUIRE(loaded->events[0].nodeId == "fader-1");
    REQUIRE(loaded->events[0].paramId == "gain");
    REQUIRE(loaded->events[0].valueNorm == 0.5f);
}

TEST_CASE("EngineHost - Automation application to FaderNode", "[automation]") {
    EngineHost host;
    host.prepareEngine(44100, 512);

    // Create a simple graph with a FaderNode
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc faderNodeDesc;
    faderNodeDesc.nodeId = "fader-1";
    faderNodeDesc.kind = NodeKind::Fader;
    faderNodeDesc.trackId = "track-1";
    snapshot.nodes.push_back(faderNodeDesc);

    // Load graph snapshot (needs PluginHost for plugin nodes, but FaderNode doesn't need it)
    host.loadGraphSnapshot(snapshot);

    // Load automation snapshot with gain automation
    AutomationData automation;
    automation.tempoMap.defaultTempo = 120.0;

    AutomationEventCompiled event;
    event.nodeId = "fader-1";
    event.paramId = "gain";
    event.timeSamples = 0;
    event.valueNorm = 0.75f; // 75% gain
    event.curve = AutomationCurveType::Step;
    automation.events.push_back(event);

    host.loadAutomationSnapshot(automation);

    // Mark transport as playing so the engine does work in renderBlock
    auto& transport = host.transport();
    transport.isPlaying = true;
    host.commitTransportUpdate();

    // Create render context and audio bus
    EngineRenderContext ctx;
    ctx.playheadSamples = 0;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;

    // Create audio buses with proper initialization
    std::vector<float> inputData(2 * 512, 0.0f);
    std::vector<float> outputData(2 * 512, 0.0f);
    AudioBus input(inputData.data(), 2, 512, false);
    AudioBus output(outputData.data(), 2, 512, false);

    // Render a block (this should apply automation)
    host.renderBlock(ctx, input, output);

    // Verify mixer node received automation
    GraphNode* node = host.graphEngine().findNode("fader-1");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::Fader);

    auto* faderNode = dynamic_cast<FaderNode*>(node);
    REQUIRE(faderNode != nullptr);
    REQUIRE(faderNode->getGain() == 0.75f); // Automation should have been applied
}

TEST_CASE("EngineHost - Automation application to SendNode", "[automation]") {
    EngineHost host;
    host.prepareEngine(44100, 512);

    // Create a simple graph with a SendNode
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc sendNode;
    sendNode.nodeId = "send-1";
    sendNode.kind = NodeKind::Send;
    sendNode.trackId = "track-1";
    snapshot.nodes.push_back(sendNode);

    // Load graph snapshot (needs PluginHost for plugin nodes, but FaderNode doesn't need it)
    host.loadGraphSnapshot(snapshot);

    // Load automation snapshot with send level automation
    AutomationData automation;
    automation.tempoMap.defaultTempo = 120.0;

    AutomationEventCompiled event;
    event.nodeId = "send-1";
    event.paramId = "send-level";
    event.timeSamples = 0;
    event.valueNorm = 0.5f; // 50% send level
    event.curve = AutomationCurveType::Step;
    automation.events.push_back(event);

    host.loadAutomationSnapshot(automation);

    // Mark transport as playing so the engine does work in renderBlock
    auto& transport = host.transport();
    transport.isPlaying = true;
    host.commitTransportUpdate();

    // Create render context and audio bus
    EngineRenderContext ctx;
    ctx.playheadSamples = 0;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;

    // Create audio buses with proper initialization
    std::vector<float> inputData(2 * 512, 0.0f);
    std::vector<float> outputData(2 * 512, 0.0f);
    AudioBus input(inputData.data(), 2, 512, false);
    AudioBus output(outputData.data(), 2, 512, false);

    // Render a block
    host.renderBlock(ctx, input, output);

    // Verify send node received automation
    GraphNode* node = host.graphEngine().findNode("send-1");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::Send);

    auto* send = dynamic_cast<SendNode*>(node);
    REQUIRE(send != nullptr);
    REQUIRE(send->getSendLevel() == 0.5f); // Automation should have been applied
}

TEST_CASE("EngineHost - Automation block-time step interpolation", "[automation]") {
    EngineHost host;
    host.prepareEngine(44100, 512);

    // Create a simple graph with a FaderNode
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc faderNodeDesc;
    faderNodeDesc.nodeId = "fader-1";
    faderNodeDesc.kind = NodeKind::Fader;
    faderNodeDesc.trackId = "track-1";
    snapshot.nodes.push_back(faderNodeDesc);

    // Load graph snapshot (needs PluginHost for plugin nodes, but FaderNode doesn't need it)
    host.loadGraphSnapshot(snapshot);

    // Load automation with events at different times
    AutomationData automation;
    automation.tempoMap.defaultTempo = 120.0;

    // Event at time 0: gain = 0.5
    AutomationEventCompiled event1;
    event1.nodeId = "fader-1";
    event1.paramId = "gain";
    event1.timeSamples = 0;
    event1.valueNorm = 0.5f;
    event1.curve = AutomationCurveType::Step;
    automation.events.push_back(event1);

    // Event at time 2048 (4 blocks later at 512 samples/block): gain = 1.0
    AutomationEventCompiled event2;
    event2.nodeId = "fader-1";
    event2.paramId = "gain";
    event2.timeSamples = 2048;
    event2.valueNorm = 1.0f;
    event2.curve = AutomationCurveType::Step;
    automation.events.push_back(event2);

    host.loadAutomationSnapshot(automation);

    // Mark transport as playing so the engine does work in renderBlock
    auto& transport = host.transport();
    transport.isPlaying = true;
    host.commitTransportUpdate();

    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;

    // Create audio buses with proper initialization
    std::vector<float> inputData(2 * 512, 0.0f);
    std::vector<float> outputData(2 * 512, 0.0f);
    AudioBus input(inputData.data(), 2, 512, false);
    AudioBus output(outputData.data(), 2, 512, false);

    // Render first block (playhead = 0, should use event1 value = 0.5)
    ctx.playheadSamples = 0;
    host.setPlayheadSamples(0);
    host.renderBlock(ctx, input, output);

    GraphNode* node = host.graphEngine().findNode("fader-1");
    auto* faderNode = dynamic_cast<FaderNode*>(node);
    REQUIRE(faderNode != nullptr);
    REQUIRE(faderNode->getGain() == 0.5f);

    // Render block at playhead = 2048 (should use event2 value = 1.0)
    ctx.playheadSamples = 2048;
    host.setPlayheadSamples(2048);
    host.renderBlock(ctx, input, output);

    REQUIRE(faderNode->getGain() == 1.0f);
}
