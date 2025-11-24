#pragma once

/// AudioBus - Abstraction for multi-channel audio buffers
///
/// Thread: Audio thread (must be real-time safe)
/// Ownership: Wraps buffers owned by AudioBackend
///
/// Provides read-only access for input buses and read-write access for output buses.

#include <cstddef>
#include <cstdint>
#include <cstring>

class AudioBus {
public:
    /// Create an AudioBus wrapping an existing buffer
    /// @param data Pointer to interleaved audio data (channels × frames)
    /// @param numChannels Number of audio channels
    /// @param numFrames Number of frames (samples per channel)
    /// @param isReadOnly If true, this bus is read-only (input)
    AudioBus(
        float* data,
        int numChannels,
        int numFrames,
        bool isReadOnly = false
    )
        : _data(data)
        , _numChannels(numChannels)
        , _numFrames(numFrames)
        , _isReadOnly(isReadOnly)
    {
    }

    /// Get number of channels
    int numChannels() const noexcept {
        return _numChannels;
    }

    /// Get number of frames
    int numFrames() const noexcept {
        return _numFrames;
    }

    /// Get total number of samples (channels × frames)
    int totalSamples() const noexcept {
        return _numChannels * _numFrames;
    }

    /// Get read-only pointer to raw interleaved data
    const float* data() const noexcept {
        return _data;
    }

    /// Get writable pointer to raw interleaved data (only for non-read-only buses)
    float* data() {
        if (_isReadOnly) {
            return nullptr;
        }
        return _data;
    }

    /// Get interleaved sample at frame and channel
    /// @param frame Frame index (0-based)
    /// @param channel Channel index (0-based)
    /// @return Sample value, or 0.0f if invalid
    float sample(int frame, int channel) const {
        if (frame < 0 || frame >= _numFrames || channel < 0 || channel >= _numChannels || !_data) {
            return 0.0f;
        }
        return _data[frame * _numChannels + channel];
    }

    /// Set interleaved sample at frame and channel (only for non-read-only buses)
    /// @param frame Frame index (0-based)
    /// @param channel Channel index (0-based)
    /// @param value Sample value
    void setSample(int frame, int channel, float value) {
        if (!_isReadOnly && frame >= 0 && frame < _numFrames && channel >= 0 && channel < _numChannels && _data) {
            _data[frame * _numChannels + channel] = value;
        }
    }

    /// Clear all samples to zero (only for non-read-only buses)
    void clear() {
        if (!_isReadOnly && _data) {
            std::memset(_data, 0, totalSamples() * sizeof(float));
        }
    }

    /// Check if this bus is read-only
    bool isReadOnly() const noexcept {
        return _isReadOnly;
    }

private:
    float* _data;           // Interleaved audio data: [frame0_ch0, frame0_ch1, ..., frame0_chN, frame1_ch0, ...]
    int _numChannels;
    int _numFrames;
    bool _isReadOnly;
};

