#pragma once

/// AudioAssetSource - Abstraction for reading audio from assets
///
/// Thread: Audio thread (readSamples)
/// Ownership: Owned by EngineHost, passed to GraphEngine
///
/// This interface allows the graph runtime to read audio samples from assets
/// identified by AssetId. Implementations may use file I/O, memory buffers,
/// or streaming decoders.

#include "core/NodeBuffers.hpp"
#include <string>
#include <cstdint>

using AssetId = std::string;

/// Abstract interface for audio asset sources
class AudioAssetSource {
public:
    virtual ~AudioAssetSource() = default;

    /// Read audio samples from an asset
    /// @param assetId Asset identifier (from AudioSegmentCompiled)
    /// @param startSample Absolute sample position in asset (0 = start of asset)
    /// @param numFrames Number of frames to read
    /// @param buffer Destination buffer (will be resized if needed)
    /// @param destFrameOffset Offset in buffer where samples should be written
    /// @param numChannels Number of channels to read (1 = mono, 2 = stereo, etc.)
    /// @return true if read succeeded, false otherwise
    virtual bool readSamples(
        const AssetId& assetId,
        uint64_t startSample,
        int numFrames,
        AudioBuffer& buffer,
        int destFrameOffset,
        int numChannels
    ) = 0;
};

/// Stub implementation for testing (generates predictable test patterns)
class StubAudioAssetSource : public AudioAssetSource {
public:
    StubAudioAssetSource() = default;

    bool readSamples(
        const AssetId& assetId,
        uint64_t startSample,
        int numFrames,
        AudioBuffer& buffer,
        int destFrameOffset,
        int numChannels
    ) override {
        // Generate a simple ramp pattern for testing
        // Sample value = (startSample + frame) / 1000.0f (scaled to avoid clipping)
        for (int frame = 0; frame < numFrames; ++frame) {
            float sample = static_cast<float>(startSample + frame) / 1000.0f;
            // Clamp to [-1.0, 1.0]
            if (sample > 1.0f) sample = 1.0f;
            if (sample < -1.0f) sample = -1.0f;

            // Write to all requested channels
            for (int ch = 0; ch < numChannels && ch < buffer.numChannels(); ++ch) {
                int destFrame = destFrameOffset + frame;
                if (destFrame >= 0 && destFrame < buffer.numFrames()) {
                    buffer.setSample(destFrame, ch, sample);
                }
            }
        }
        return true;
    }
};

