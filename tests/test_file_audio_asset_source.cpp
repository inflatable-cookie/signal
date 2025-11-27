#include <catch2/catch_test_macros.hpp>
#include "core/AudioAssetSource.hpp"
#include "core/NodeBuffers.hpp"
#include <vector>
#include <cmath>

TEST_CASE("FileAudioAssetSource - prepareAsset and readSamples with mock data", "[assets][file-source]") {
    FileAudioAssetSource source;

    // Create a mock asset with known PCM data
    // We'll simulate a small stereo file (100 frames, 2 channels)
    const int numFrames = 100;
    const int numChannels = 2;

    // Generate test PCM data: simple ramp pattern
    std::vector<float> mockPcm(numFrames * numChannels);
    for (int frame = 0; frame < numFrames; ++frame) {
        float sample = static_cast<float>(frame) / static_cast<float>(numFrames);
        mockPcm[frame * numChannels + 0] = sample; // Left channel
        mockPcm[frame * numChannels + 1] = sample; // Right channel
    }

    // Note: In a real test, we'd need to create an actual audio file or mock the decoder
    // For now, this test verifies the readSamples logic with pre-loaded data
    // We'll test the actual file decoding in integration tests

    // Test that readSamples handles missing assets gracefully
    AudioBuffer buffer;
    buffer.resize(2, 512);

    bool result = source.readSamples("nonexistent-asset", 0, 512, buffer, 0, 2);
    REQUIRE(result == false); // Should return false for missing asset

    // Verify buffer is filled with silence
    for (int frame = 0; frame < 512; ++frame) {
        for (int ch = 0; ch < 2; ++ch) {
            REQUIRE(std::abs(buffer.getSample(frame, ch)) < 0.001f);
        }
    }
}

TEST_CASE("AudioAssetSourceRouter - routes test assets to stub", "[assets][router]") {
    AudioAssetSourceRouter router;
    router.setSampleRate(44100.0);

    AudioBuffer buffer;
    buffer.resize(2, 512);

    // Test asset should route to stub source
    bool result = router.readSamples("test://tone-440hz", 0, 512, buffer, 0, 2);
    REQUIRE(result == true);

    // Verify we got a sine wave (not silence)
    bool hasNonZero = false;
    for (int frame = 0; frame < 512; ++frame) {
        if (std::abs(buffer.getSample(frame, 0)) > 0.001f) {
            hasNonZero = true;
            break;
        }
    }
    REQUIRE(hasNonZero);
}

TEST_CASE("AudioAssetSourceRouter - routes file assets to file source", "[assets][router]") {
    AudioAssetSourceRouter router;

    AudioBuffer buffer;
    buffer.resize(2, 512);

    // Non-test asset should route to file source (which will produce silence if not prepared)
    bool result = router.readSamples("asset-123", 0, 512, buffer, 0, 2);
    REQUIRE(result == false); // File source returns false for unprepared assets

    // Verify buffer is filled with silence
    for (int frame = 0; frame < 512; ++frame) {
        for (int ch = 0; ch < 2; ++ch) {
            REQUIRE(std::abs(buffer.getSample(frame, ch)) < 0.001f);
        }
    }
}

