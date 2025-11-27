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
///
/// Thread: Audio thread (readSamples)
/// Ownership: Owned by EngineHost
///
/// This implementation provides test audio generation for development and testing.
/// It handles special test asset IDs and generates audio patterns.
class StubAudioAssetSource : public AudioAssetSource {
public:
    StubAudioAssetSource() : _sampleRate(44100.0) {
        // Default sample rate (will be updated when engine is prepared)
    }

    /// Set sample rate for tone generation
    void setSampleRate(double sampleRate) {
        _sampleRate = sampleRate;
    }

    bool readSamples(
        const AssetId& assetId,
        uint64_t startSample,
        int numFrames,
        AudioBuffer& buffer,
        int destFrameOffset,
        int numChannels
    ) override {
        // Handle special test asset IDs
        if (assetId == "test://tone-440hz") {
            // Generate 440 Hz sine wave at comfortable level (0.15 amplitude)
            const float amplitude = 0.15f;
            const float frequency = 440.0f;
            const float twoPi = 2.0f * 3.14159265358979323846f;

            for (int frame = 0; frame < numFrames; ++frame) {
                uint64_t absoluteSample = startSample + frame;
                float phase = static_cast<float>(absoluteSample) * frequency / static_cast<float>(_sampleRate);
                float sample = amplitude * std::sin(twoPi * phase);

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

        // For other asset IDs, generate silence (safe default)
        // This ensures unknown assets don't produce noise
        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels && ch < buffer.numChannels(); ++ch) {
                int destFrame = destFrameOffset + frame;
                if (destFrame >= 0 && destFrame < buffer.numFrames()) {
                    buffer.setSample(destFrame, ch, 0.0f);
                }
            }
        }
        return true;
    }

private:
    double _sampleRate;  // Sample rate for tone generation
};

