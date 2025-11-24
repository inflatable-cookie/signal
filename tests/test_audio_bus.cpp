#include <catch2/catch_test_macros.hpp>
#include "core/AudioBus.hpp"
#include <cstring>
#include <cmath>

TEST_CASE("AudioBus basic properties", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 256;
    const int totalSamples = numChannels * numFrames;

    float buffer[totalSamples];
    std::memset(buffer, 0, sizeof(buffer));

    AudioBus bus(buffer, numChannels, numFrames, false);

    REQUIRE(bus.numChannels() == numChannels);
    REQUIRE(bus.numFrames() == numFrames);
    REQUIRE(bus.totalSamples() == totalSamples);
    REQUIRE(bus.isReadOnly() == false);
    REQUIRE(bus.data() != nullptr);
}

TEST_CASE("AudioBus read-only", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 256;
    float buffer[numChannels * numFrames];

    AudioBus readOnlyBus(buffer, numChannels, numFrames, true);
    REQUIRE(readOnlyBus.isReadOnly() == true);
    // Const data() should work for read-only bus
    const AudioBus& constReadOnlyBus = readOnlyBus;
    REQUIRE(constReadOnlyBus.data() != nullptr);
    // Non-const data() should return nullptr for read-only bus
    REQUIRE(readOnlyBus.data() == nullptr);

    // Writable data() should return non-null for writable bus
    AudioBus writableBus(buffer, numChannels, numFrames, false);
    REQUIRE(writableBus.isReadOnly() == false);
    REQUIRE(writableBus.data() != nullptr);
}

TEST_CASE("AudioBus sample access", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 4;
    float buffer[numChannels * numFrames];

    // Initialize with test pattern: frame 0 = [1.0, 2.0], frame 1 = [3.0, 4.0], etc.
    for (int frame = 0; frame < numFrames; ++frame) {
        for (int ch = 0; ch < numChannels; ++ch) {
            buffer[frame * numChannels + ch] = static_cast<float>(frame * numChannels + ch + 1);
        }
    }

    AudioBus bus(buffer, numChannels, numFrames, false);

    // Test reading samples
    REQUIRE(std::abs(bus.sample(0, 0) - 1.0f) < 1e-6f);
    REQUIRE(std::abs(bus.sample(0, 1) - 2.0f) < 1e-6f);
    REQUIRE(std::abs(bus.sample(1, 0) - 3.0f) < 1e-6f);
    REQUIRE(std::abs(bus.sample(1, 1) - 4.0f) < 1e-6f);

    // Test out-of-bounds access
    REQUIRE(bus.sample(-1, 0) == 0.0f);
    REQUIRE(bus.sample(0, -1) == 0.0f);
    REQUIRE(bus.sample(numFrames, 0) == 0.0f);
    REQUIRE(bus.sample(0, numChannels) == 0.0f);
}

TEST_CASE("AudioBus setSample", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 4;
    float buffer[numChannels * numFrames];
    std::memset(buffer, 0, sizeof(buffer));

    AudioBus writableBus(buffer, numChannels, numFrames, false);

    // Set samples
    writableBus.setSample(0, 0, 1.5f);
    writableBus.setSample(0, 1, 2.5f);
    writableBus.setSample(1, 0, 3.5f);

    // Verify
    REQUIRE(std::abs(writableBus.sample(0, 0) - 1.5f) < 1e-6f);
    REQUIRE(std::abs(writableBus.sample(0, 1) - 2.5f) < 1e-6f);
    REQUIRE(std::abs(writableBus.sample(1, 0) - 3.5f) < 1e-6f);

    // Test that setSample fails silently for out-of-bounds
    writableBus.setSample(-1, 0, 99.0f);
    writableBus.setSample(0, -1, 99.0f);
    writableBus.setSample(numFrames, 0, 99.0f);
    writableBus.setSample(0, numChannels, 99.0f);
    // Verify no changes
    REQUIRE(std::abs(writableBus.sample(0, 0) - 1.5f) < 1e-6f);
}

TEST_CASE("AudioBus setSample read-only", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 4;
    float buffer[numChannels * numFrames];
    std::memset(buffer, 0, sizeof(buffer));

    AudioBus readOnlyBus(buffer, numChannels, numFrames, true);

    // setSample should fail silently for read-only bus
    readOnlyBus.setSample(0, 0, 1.0f);
    REQUIRE(buffer[0] == 0.0f);  // Should remain unchanged
}

TEST_CASE("AudioBus clear", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 4;
    float buffer[numChannels * numFrames];

    // Fill with non-zero values
    for (int i = 0; i < numChannels * numFrames; ++i) {
        buffer[i] = 1.0f;
    }

    AudioBus writableBus(buffer, numChannels, numFrames, false);
    writableBus.clear();

    // Verify all samples are zero
    for (int frame = 0; frame < numFrames; ++frame) {
        for (int ch = 0; ch < numChannels; ++ch) {
            REQUIRE(std::abs(writableBus.sample(frame, ch)) < 1e-6f);
        }
    }
}

TEST_CASE("AudioBus clear read-only", "[core][audiobus]") {
    const int numChannels = 2;
    const int numFrames = 4;
    float buffer[numChannels * numFrames];

    // Fill with non-zero values
    for (int i = 0; i < numChannels * numFrames; ++i) {
        buffer[i] = 1.0f;
    }

    AudioBus readOnlyBus(buffer, numChannels, numFrames, true);
    readOnlyBus.clear();

    // Verify values are unchanged (clear should do nothing for read-only)
    for (int i = 0; i < numChannels * numFrames; ++i) {
        REQUIRE(std::abs(buffer[i] - 1.0f) < 1e-6f);
    }
}

