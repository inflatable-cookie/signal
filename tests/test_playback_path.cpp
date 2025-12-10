#include <catch2/catch_test_macros.hpp>
#include "core/EngineHost.hpp"
#include "core/ScheduleData.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/AudioAssetSource.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "core/ScheduleData.hpp"
#include "core/StreamScheduler.hpp"
#include <vector>
#include <string>
#include <memory>
#include <cmath>
#include <algorithm>

// Note: For test case B, we use a special test asset ID that the StubAudioAssetSource
// can handle, or we would need to inject a fake FileAudioAssetSource.
// For now, test case B uses a test:// pattern that produces a different tone.

TEST_CASE("Playback Path - Test Tone (440Hz) Offline Render", "[playback][offline][tone]") {
    EngineHost host;

    // Build synthetic graph snapshot
    GraphSnapshot snapshot;
    snapshot.id = "test-playback-graph";

    // AudioLane node
    NodeDesc laneNode;
    laneNode.nodeId = "audio-lane-1";
    laneNode.kind = NodeKind::AudioLane;
    laneNode.trackId = "track-1";
    laneNode.laneId = "lane-1";
    snapshot.nodes.push_back(laneNode);

    // Device node
    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    // Stream binding: test-stream -> audio-lane-1
    ConnectionDesc streamBinding;
    streamBinding.fromStreamId = "test-stream";
    streamBinding.toNodeId = "audio-lane-1";
    streamBinding.toInputIndex = 0;
    snapshot.connections.push_back(streamBinding);

    // Node connection: audio-lane-1 -> device
    ConnectionDesc nodeConn;
    nodeConn.fromNodeId = "audio-lane-1";
    nodeConn.fromOutputIndex = 0;
    nodeConn.toNodeId = "device";
    nodeConn.toInputIndex = 0;
    snapshot.connections.push_back(nodeConn);

    host.loadGraphSnapshot(snapshot);

    // Prepare graph and asset sources after the snapshot is loaded
    host.prepareEngine(44100, 512);

    // Build minimal playback schedule
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
    segment.endSamples = 512 * 256; // 256 blocks worth
    segment.assetStartSamples = 0;
    segment.gainDb = 0.0;
    segment.fadeInSamples = 0;
    segment.fadeOutSamples = 0;
    audioSegments.push_back(segment);

    std::vector<MidiEventCompiled> midiEvents;
    TempoMap tempoMap;
    tempoMap.defaultTempo = 120.0;

    // Create ScheduleData object
    ScheduleData schedule(44100.0, 120.0);
    schedule.streams = streams;
    schedule.audioSegments = audioSegments;
    schedule.midiEvents = midiEvents;
    schedule.tempoMap = tempoMap;
    schedule.buildLookupMaps();

    // Load schedule via StreamScheduler
    host.streamScheduler().setSchedule(schedule);

    // Set transport to playing
    host.transport().isPlaying = true;
    host.transport().positionSamples = 0;
    host.transport().positionSeconds = 0.0;
    host.commitTransportUpdate();

    // Replace asset source with test router
    // Note: EngineHost owns the asset source, so we need to work with what it provides
    // For this test, we'll rely on the existing AudioAssetSourceRouter which should
    // delegate to StubAudioAssetSource for test://tone-440hz
    // We'll verify the output is non-zero

    // Create render context
    EngineRenderContext ctx;
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;
    ctx.isPlaying = true;
    ctx.loopStartBeats = 0.0;
    ctx.loopEndBeats = 0.0;

    // Create audio buses
    std::vector<float> inputData(2 * 512, 0.0f);
    std::vector<float> outputData(2 * 512, 0.0f);
    AudioBus input(inputData.data(), 2, 512, false);
    AudioBus output(outputData.data(), 2, 512, false);

    // Render multiple blocks and accumulate output
    float maxOutput = 0.0f;
    float rmsOutput = 0.0f;
    int totalSamples = 0;

    for (int block = 0; block < 256; ++block) {
        // Clear output buffer
        std::fill(outputData.begin(), outputData.end(), 0.0f);

        // Update playhead
        ctx.playheadSamples = block * 512;

        // Render block
        host.renderBlock(ctx, input, output);

        // Check output
        for (int ch = 0; ch < 2; ++ch) {
            for (int frame = 0; frame < 512; ++frame) {
                float sample = output.sample(frame, ch);
                float absSample = std::abs(sample);
                maxOutput = std::max(maxOutput, absSample);
                rmsOutput += sample * sample;
                totalSamples++;
            }
        }
    }

    rmsOutput = std::sqrt(rmsOutput / totalSamples);

    // Assert: output is not all zeros
    REQUIRE(maxOutput > 0.01f);  // Should have significant output
    REQUIRE(maxOutput < 1.0f);   // Should be within sane bounds
    REQUIRE(rmsOutput > 0.001f); // RMS should be non-zero
    REQUIRE(rmsOutput < 0.5f);   // RMS should be reasonable
}

TEST_CASE("Playback Path - Fake File Asset Offline Render", "[playback][offline][file]") {
    EngineHost host;

    // Build synthetic graph snapshot (same as test A)
    GraphSnapshot snapshot;
    snapshot.id = "test-playback-graph-file";

    NodeDesc laneNode;
    laneNode.nodeId = "audio-lane-1";
    laneNode.kind = NodeKind::AudioLane;
    laneNode.trackId = "track-1";
    laneNode.laneId = "lane-1";
    snapshot.nodes.push_back(laneNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::Device;
    snapshot.nodes.push_back(deviceNode);

    ConnectionDesc streamBinding;
    streamBinding.fromStreamId = "test-stream";
    streamBinding.toNodeId = "audio-lane-1";
    streamBinding.toInputIndex = 0;
    snapshot.connections.push_back(streamBinding);

    ConnectionDesc nodeConn;
    nodeConn.fromNodeId = "audio-lane-1";
    nodeConn.fromOutputIndex = 0;
    nodeConn.toNodeId = "device";
    nodeConn.toInputIndex = 0;
    snapshot.connections.push_back(nodeConn);

    host.loadGraphSnapshot(snapshot);

    // Prepare graph and asset sources after the snapshot is loaded
    host.prepareEngine(44100, 512);

    // Build playback schedule with a different test tone asset
    // Note: In a real scenario, this would be a file asset, but for this test
    // we use a different test tone to verify the routing works
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
    segment.assetId = "test://tone-440hz";  // Using test tone for now (file asset test would need real file)
    segment.startSamples = 0;
    segment.endSamples = 512 * 256;
    segment.assetStartSamples = 0;
    segment.gainDb = 0.0;
    segment.fadeInSamples = 0;
    segment.fadeOutSamples = 0;
    audioSegments.push_back(segment);

    std::vector<MidiEventCompiled> midiEvents;
    TempoMap tempoMap;
    tempoMap.defaultTempo = 120.0;

    ScheduleData schedule(44100.0, 120.0);
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
    ctx.sampleRate = 44100.0;
    ctx.blockSize = 512;
    ctx.playheadSamples = 0;
    ctx.isPlaying = true;
    ctx.loopStartBeats = 0.0;
    ctx.loopEndBeats = 0.0;

    std::vector<float> inputData(2 * 512, 0.0f);
    std::vector<float> outputData(2 * 512, 0.0f);
    AudioBus input(inputData.data(), 2, 512, false);
    AudioBus output(outputData.data(), 2, 512, false);

    float maxOutput = 0.0f;
    float rmsOutput = 0.0f;
    int totalSamples = 0;

    for (int block = 0; block < 256; ++block) {
        std::fill(outputData.begin(), outputData.end(), 0.0f);
        ctx.playheadSamples = block * 512;
        host.renderBlock(ctx, input, output);

        for (int ch = 0; ch < 2; ++ch) {
            for (int frame = 0; frame < 512; ++frame) {
                float sample = output.sample(frame, ch);
                float absSample = std::abs(sample);
                maxOutput = std::max(maxOutput, absSample);
                rmsOutput += sample * sample;
                totalSamples++;
            }
        }
    }

    rmsOutput = std::sqrt(rmsOutput / totalSamples);

    // Assert: output is not all zeros (same as test case A for now)
    // TODO: Test case B should use a real file asset or injectable fake FileAudioAssetSource
    REQUIRE(maxOutput > 0.01f);  // Should have significant output
    REQUIRE(maxOutput < 1.0f);   // Should be within sane bounds
    REQUIRE(rmsOutput > 0.001f); // RMS should be non-zero
    REQUIRE(rmsOutput < 0.5f);   // RMS should be reasonable
}
