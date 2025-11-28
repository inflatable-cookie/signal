#include "core/EngineSelfTest.hpp"
#include "core/EngineHost.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/ScheduleData.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "core/StreamScheduler.hpp"
#include "logging/Logging.hpp"
#include <cmath>
#include <algorithm>
#include <memory>
#include <vector>
#include <sstream>

namespace {

constexpr float EPSILON = 0.0001f;
constexpr int TEST_SAMPLE_RATE = 44100;
constexpr int TEST_BLOCK_SIZE = 512;
constexpr int TEST_BLOCKS_PER_SCENARIO = 110; // ~0.25 seconds at 44.1kHz

/// Compute max absolute sample value from an audio bus
float computeMaxAbsSample(const AudioBus& bus) {
    float maxAbs = 0.0f;
    const int numChannels = bus.numChannels();
    const int numFrames = bus.numFrames();

    for (int frame = 0; frame < numFrames; ++frame) {
        for (int ch = 0; ch < numChannels; ++ch) {
            float absSample = std::abs(bus.sample(frame, ch));
            maxAbs = std::max(maxAbs, absSample);
        }
    }

    return maxAbs;
}

/// Scenario: Mono-to-stereo path
/// Creates a simple mono audio lane -> stereo device graph and renders a test tone
EngineSelfTestScenarioResult runMonoToStereoScenario() {
    EngineSelfTestScenarioResult result;
    result.id = "mono-to-stereo";

    try {
        // Create isolated EngineHost for this test
        EngineHost host;
        host.prepareEngine(TEST_SAMPLE_RATE, TEST_BLOCK_SIZE);

        // Build synthetic graph: mono lane -> stereo device
        GraphSnapshot snapshot;
        snapshot.id = "self-test-mono-stereo";

        // AudioLane node (mono output)
        NodeDesc laneNode;
        laneNode.nodeId = "lane-1";
        laneNode.kind = NodeKind::AudioLane;
        {
            NodeAudioConfigDesc audioConfig;
            audioConfig.numInputs = 0;
            audioConfig.numOutputs = 1; // Mono
            laneNode.audio = audioConfig;
        }
        snapshot.nodes.push_back(laneNode);

        // Device node (stereo input)
        NodeDesc deviceNode;
        deviceNode.nodeId = "device";
        deviceNode.kind = NodeKind::Device;
        {
            NodeAudioConfigDesc audioConfig;
            audioConfig.numInputs = 2; // Stereo
            audioConfig.numOutputs = 2;
            deviceNode.audio = audioConfig;
        }
        snapshot.nodes.push_back(deviceNode);

        // Stream binding
        ConnectionDesc streamBinding;
        streamBinding.fromStreamId = "test-stream";
        streamBinding.toNodeId = "lane-1";
        streamBinding.toInputIndex = 0;
        snapshot.connections.push_back(streamBinding);

        // Node connection: lane -> device
        ConnectionDesc nodeConn;
        nodeConn.fromNodeId = "lane-1";
        nodeConn.fromOutputIndex = 0;
        nodeConn.toNodeId = "device";
        nodeConn.toInputIndex = 0;
        snapshot.connections.push_back(nodeConn);

        host.loadGraphSnapshot(snapshot);

        // Build minimal schedule with test tone
        std::vector<StreamDescriptor> streams;
        StreamDescriptor stream;
        stream.streamId = "test-stream";
        stream.trackId = "track-1";
        stream.laneId = "lane-1";
        stream.streamType = "audio";
        streams.push_back(stream);

        std::vector<AudioSegmentCompiled> audioSegments;
        AudioSegmentCompiled segment;
        segment.streamId = "test-stream";
        segment.assetId = "test://tone-440hz";
        segment.startSamples = 0;
        segment.endSamples = TEST_BLOCK_SIZE * TEST_BLOCKS_PER_SCENARIO;
        segment.assetStartSamples = 0;
        segment.gainDb = 0.0;
        segment.fadeInSamples = 0;
        segment.fadeOutSamples = 0;
        audioSegments.push_back(segment);

        std::vector<MidiEventCompiled> midiEvents;
        TempoMap tempoMap;
        tempoMap.defaultTempo = 120.0;

        ScheduleData schedule(static_cast<double>(TEST_SAMPLE_RATE), 120.0);
        schedule.streams = streams;
        schedule.audioSegments = audioSegments;
        schedule.midiEvents = midiEvents;
        schedule.tempoMap = tempoMap;
        schedule.buildLookupMaps();

        host.streamScheduler().setSchedule(schedule);

        // Set transport to playing
        host.transport().isPlaying = true;
        host.transport().positionSamples = 0;
        host.transport().positionSeconds = 0.0;
        host.commitTransportUpdate();

        // Create render context
        EngineRenderContext ctx;
        ctx.sampleRate = static_cast<double>(TEST_SAMPLE_RATE);
        ctx.blockSize = TEST_BLOCK_SIZE;
        ctx.playheadSamples = 0;
        ctx.isPlaying = true;
        ctx.loopStartBeats = 0.0;
        ctx.loopEndBeats = 0.0;

        // Create audio buses
        std::vector<float> inputData(2 * TEST_BLOCK_SIZE, 0.0f);
        std::vector<float> outputData(2 * TEST_BLOCK_SIZE, 0.0f);
        AudioBus input(inputData.data(), 2, TEST_BLOCK_SIZE, false);
        AudioBus output(outputData.data(), 2, TEST_BLOCK_SIZE, false);

        // Render blocks and track max output
        float maxOutput = 0.0f;
        for (int block = 0; block < TEST_BLOCKS_PER_SCENARIO; ++block) {
            std::fill(outputData.begin(), outputData.end(), 0.0f);
            ctx.playheadSamples = block * TEST_BLOCK_SIZE;
            host.renderBlock(ctx, input, output);

            float blockMax = computeMaxAbsSample(output);
            maxOutput = std::max(maxOutput, blockMax);
        }

        result.maxAbsSample = maxOutput;
        // Pass if we have reasonable output (not silence, not clipping)
        result.ok = (maxOutput > EPSILON) && (maxOutput < 1.0f);

    } catch (const std::exception& e) {
        LOG_ERROR({"EngineSelfTest"}, std::string("Mono-to-stereo scenario failed: ") + e.what());
        result.ok = false;
    } catch (...) {
        LOG_ERROR({"EngineSelfTest"}, "Mono-to-stereo scenario failed with unknown exception");
        result.ok = false;
    }

    return result;
}

/// Scenario: Stereo path with valid graph
/// Creates a stereo audio lane -> stereo device graph
EngineSelfTestScenarioResult runStereoScenario() {
    EngineSelfTestScenarioResult result;
    result.id = "stereo";

    try {
        EngineHost host;
        host.prepareEngine(TEST_SAMPLE_RATE, TEST_BLOCK_SIZE);

        GraphSnapshot snapshot;
        snapshot.id = "self-test-stereo";

        NodeDesc laneNode;
        laneNode.nodeId = "lane-1";
        laneNode.kind = NodeKind::AudioLane;
        {
            NodeAudioConfigDesc audioConfig;
            audioConfig.numInputs = 0;
            audioConfig.numOutputs = 2; // Stereo
            laneNode.audio = audioConfig;
        }
        snapshot.nodes.push_back(laneNode);

        NodeDesc deviceNode;
        deviceNode.nodeId = "device";
        deviceNode.kind = NodeKind::Device;
        {
            NodeAudioConfigDesc audioConfig;
            audioConfig.numInputs = 2;
            audioConfig.numOutputs = 2;
            deviceNode.audio = audioConfig;
        }
        snapshot.nodes.push_back(deviceNode);

        ConnectionDesc streamBinding;
        streamBinding.fromStreamId = "test-stream";
        streamBinding.toNodeId = "lane-1";
        streamBinding.toInputIndex = 0;
        snapshot.connections.push_back(streamBinding);

        ConnectionDesc nodeConn;
        nodeConn.fromNodeId = "lane-1";
        nodeConn.fromOutputIndex = 0;
        nodeConn.toNodeId = "device";
        nodeConn.toInputIndex = 0;
        snapshot.connections.push_back(nodeConn);

        host.loadGraphSnapshot(snapshot);

        // Build schedule with test tone
        std::vector<StreamDescriptor> streams;
        StreamDescriptor stream;
        stream.streamId = "test-stream";
        stream.trackId = "track-1";
        stream.laneId = "lane-1";
        stream.streamType = "audio";
        streams.push_back(stream);

        std::vector<AudioSegmentCompiled> audioSegments;
        AudioSegmentCompiled segment;
        segment.streamId = "test-stream";
        segment.assetId = "test://tone-440hz";
        segment.startSamples = 0;
        segment.endSamples = TEST_BLOCK_SIZE * TEST_BLOCKS_PER_SCENARIO;
        segment.assetStartSamples = 0;
        segment.gainDb = 0.0;
        segment.fadeInSamples = 0;
        segment.fadeOutSamples = 0;
        audioSegments.push_back(segment);

        std::vector<MidiEventCompiled> midiEvents;
        TempoMap tempoMap;
        tempoMap.defaultTempo = 120.0;

        ScheduleData schedule(static_cast<double>(TEST_SAMPLE_RATE), 120.0);
        schedule.streams = streams;
        schedule.audioSegments = audioSegments;
        schedule.midiEvents = midiEvents;
        schedule.tempoMap = tempoMap;
        schedule.buildLookupMaps();

        host.streamScheduler().setSchedule(schedule);

        host.transport().isPlaying = true;
        host.transport().positionSamples = 0;
        host.transport().positionSeconds = 0.0;
        host.commitTransportUpdate();

        EngineRenderContext ctx;
        ctx.sampleRate = static_cast<double>(TEST_SAMPLE_RATE);
        ctx.blockSize = TEST_BLOCK_SIZE;
        ctx.playheadSamples = 0;
        ctx.isPlaying = true;
        ctx.loopStartBeats = 0.0;
        ctx.loopEndBeats = 0.0;

        std::vector<float> inputData(2 * TEST_BLOCK_SIZE, 0.0f);
        std::vector<float> outputData(2 * TEST_BLOCK_SIZE, 0.0f);
        AudioBus input(inputData.data(), 2, TEST_BLOCK_SIZE, false);
        AudioBus output(outputData.data(), 2, TEST_BLOCK_SIZE, false);

        float maxOutput = 0.0f;
        for (int block = 0; block < TEST_BLOCKS_PER_SCENARIO; ++block) {
            std::fill(outputData.begin(), outputData.end(), 0.0f);
            ctx.playheadSamples = block * TEST_BLOCK_SIZE;
            host.renderBlock(ctx, input, output);

            float blockMax = computeMaxAbsSample(output);
            maxOutput = std::max(maxOutput, blockMax);
        }

        result.maxAbsSample = maxOutput;
        result.ok = (maxOutput > EPSILON) && (maxOutput < 1.0f);

    } catch (const std::exception& e) {
        LOG_ERROR({"EngineSelfTest"}, std::string("Stereo scenario failed: ") + e.what());
        result.ok = false;
    } catch (...) {
        LOG_ERROR({"EngineSelfTest"}, "Stereo scenario failed with unknown exception");
        result.ok = false;
    }

    return result;
}

/// Scenario: Silence with valid graph (no schedule content)
/// Creates a valid graph but with no audio segments, should produce silence
EngineSelfTestScenarioResult runSilenceScenario() {
    EngineSelfTestScenarioResult result;
    result.id = "silence-valid-graph";

    try {
        EngineHost host;
        host.prepareEngine(TEST_SAMPLE_RATE, TEST_BLOCK_SIZE);

        GraphSnapshot snapshot;
        snapshot.id = "self-test-silence";

        NodeDesc laneNode;
        laneNode.nodeId = "lane-1";
        laneNode.kind = NodeKind::AudioLane;
        {
            NodeAudioConfigDesc audioConfig;
            audioConfig.numInputs = 0;
            audioConfig.numOutputs = 2;
            laneNode.audio = audioConfig;
        }
        snapshot.nodes.push_back(laneNode);

        NodeDesc deviceNode;
        deviceNode.nodeId = "device";
        deviceNode.kind = NodeKind::Device;
        {
            NodeAudioConfigDesc audioConfig;
            audioConfig.numInputs = 2;
            audioConfig.numOutputs = 2;
            deviceNode.audio = audioConfig;
        }
        snapshot.nodes.push_back(deviceNode);

        ConnectionDesc nodeConn;
        nodeConn.fromNodeId = "lane-1";
        nodeConn.fromOutputIndex = 0;
        nodeConn.toNodeId = "device";
        nodeConn.toInputIndex = 0;
        snapshot.connections.push_back(nodeConn);

        host.loadGraphSnapshot(snapshot);

        // Empty schedule (no audio segments)
        std::vector<StreamDescriptor> streams;
        std::vector<AudioSegmentCompiled> audioSegments;
        std::vector<MidiEventCompiled> midiEvents;
        TempoMap tempoMap;
        tempoMap.defaultTempo = 120.0;

        ScheduleData schedule(static_cast<double>(TEST_SAMPLE_RATE), 120.0);
        schedule.streams = streams;
        schedule.audioSegments = audioSegments;
        schedule.midiEvents = midiEvents;
        schedule.tempoMap = tempoMap;
        schedule.buildLookupMaps();

        host.streamScheduler().setSchedule(schedule);

        host.transport().isPlaying = true;
        host.transport().positionSamples = 0;
        host.transport().positionSeconds = 0.0;
        host.commitTransportUpdate();

        EngineRenderContext ctx;
        ctx.sampleRate = static_cast<double>(TEST_SAMPLE_RATE);
        ctx.blockSize = TEST_BLOCK_SIZE;
        ctx.playheadSamples = 0;
        ctx.isPlaying = true;
        ctx.loopStartBeats = 0.0;
        ctx.loopEndBeats = 0.0;

        std::vector<float> inputData(2 * TEST_BLOCK_SIZE, 0.0f);
        std::vector<float> outputData(2 * TEST_BLOCK_SIZE, 0.0f);
        AudioBus input(inputData.data(), 2, TEST_BLOCK_SIZE, false);
        AudioBus output(outputData.data(), 2, TEST_BLOCK_SIZE, false);

        float maxOutput = 0.0f;
        for (int block = 0; block < TEST_BLOCKS_PER_SCENARIO; ++block) {
            std::fill(outputData.begin(), outputData.end(), 0.0f);
            ctx.playheadSamples = block * TEST_BLOCK_SIZE;
            host.renderBlock(ctx, input, output);

            float blockMax = computeMaxAbsSample(output);
            maxOutput = std::max(maxOutput, blockMax);
        }

        result.maxAbsSample = maxOutput;
        // Pass if output is silence (or very close to it)
        result.ok = (maxOutput <= EPSILON);

    } catch (const std::exception& e) {
        LOG_ERROR({"EngineSelfTest"}, std::string("Silence scenario failed: ") + e.what());
        result.ok = false;
    } catch (...) {
        LOG_ERROR({"EngineSelfTest"}, "Silence scenario failed with unknown exception");
        result.ok = false;
    }

    return result;
}

} // anonymous namespace

EngineSelfTestResult runEngineSelfTest() {
    EngineSelfTestResult result;
    result.ok = true;

    LOG_INFO({"EngineSelfTest"}, "Starting engine self-test");

    // Run scenarios
    auto scenario1 = runMonoToStereoScenario();
    result.scenarios.push_back(scenario1);
    if (!scenario1.ok) {
        result.ok = false;
    }

    auto scenario2 = runStereoScenario();
    result.scenarios.push_back(scenario2);
    if (!scenario2.ok) {
        result.ok = false;
    }

    auto scenario3 = runSilenceScenario();
    result.scenarios.push_back(scenario3);
    if (!scenario3.ok) {
        result.ok = false;
    }

    // Log summary
    int passed = 0;
    int failed = 0;
    for (const auto& scenario : result.scenarios) {
        if (scenario.ok) {
            passed++;
        } else {
            failed++;
        }
    }

    std::ostringstream msg;
    msg << "Engine self-test complete: " << passed << " passed, " << failed << " failed";
    if (result.ok) {
        LOG_INFO({"EngineSelfTest"}, msg.str());
    } else {
        LOG_WARN({"EngineSelfTest"}, msg.str());
    }

    return result;
}

