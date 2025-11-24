#pragma once

/// NodeBuffers - Internal buffer model for audio and MIDI flow through nodes
///
/// Thread: Audio thread (read/write during processing)
/// Ownership: Owned by GraphNode instances
///
/// This provides a minimal buffer model for Phase 2:
/// - Audio buffers (multi-channel, per-frame)
/// - MIDI buffers (list of MIDI messages with sample offsets)

#include <vector>
#include <cstdint>
#include <cstring>

/// MIDI message with sample offset within block
struct MidiMessage {
    uint8_t status;
    uint8_t data1;
    uint8_t data2;
    uint8_t channel;
    uint64_t sampleOffset; // Within the block, 0..blockSize-1
};

/// MIDI buffer - collection of MIDI messages
class MidiBuffer {
public:
    MidiBuffer() = default;

    /// Clear all messages
    void clear() {
        messages.clear();
    }

    /// Add a MIDI message
    void addMessage(const MidiMessage& msg) {
        messages.push_back(msg);
    }

    /// Get all messages
    const std::vector<MidiMessage>& getMessages() const noexcept {
        return messages;
    }

    /// Get number of messages
    size_t size() const noexcept {
        return messages.size();
    }

    /// Append messages from another buffer
    void append(const MidiBuffer& other) {
        messages.insert(messages.end(), other.messages.begin(), other.messages.end());
    }

private:
    std::vector<MidiMessage> messages;
};

/// Audio buffer - multi-channel audio data
/// Uses deinterleaved storage (vector of channel vectors)
class AudioBuffer {
public:
    AudioBuffer() : _numChannels(0), _numFrames(0) {}

    /// Resize buffer (called during prepare)
    void resize(int numChannels, int numFrames) {
        _numChannels = numChannels;
        _numFrames = numFrames;
        _data.resize(numChannels);
        for (int ch = 0; ch < numChannels; ++ch) {
            _data[ch].resize(numFrames);
        }
    }

    /// Clear all samples to zero
    void clear() {
        for (auto& channel : _data) {
            std::memset(channel.data(), 0, _numFrames * sizeof(float));
        }
    }

    /// Get number of channels
    int numChannels() const noexcept {
        return _numChannels;
    }

    /// Get number of frames
    int numFrames() const noexcept {
        return _numFrames;
    }

    /// Get read-only pointer to channel data
    const float* getChannelData(int channel) const {
        if (channel < 0 || channel >= _numChannels) {
            return nullptr;
        }
        return _data[channel].data();
    }

    /// Get writable pointer to channel data
    float* getChannelData(int channel) {
        if (channel < 0 || channel >= _numChannels) {
            return nullptr;
        }
        return _data[channel].data();
    }

    /// Get sample at frame and channel
    float getSample(int frame, int channel) const {
        if (frame < 0 || frame >= _numFrames || channel < 0 || channel >= _numChannels) {
            return 0.0f;
        }
        return _data[channel][frame];
    }

    /// Set sample at frame and channel
    void setSample(int frame, int channel, float value) {
        if (frame >= 0 && frame < _numFrames && channel >= 0 && channel < _numChannels) {
            _data[channel][frame] = value;
        }
    }

    /// Sum samples from another buffer into this buffer (fan-in)
    /// Both buffers must have same dimensions
    void sumFrom(const AudioBuffer& other) {
        if (other._numChannels != _numChannels || other._numFrames != _numFrames) {
            return; // Mismatch - skip
        }

        for (int ch = 0; ch < _numChannels; ++ch) {
            const float* src = other._data[ch].data();
            float* dst = _data[ch].data();
            for (int frame = 0; frame < _numFrames; ++frame) {
                dst[frame] += src[frame];
            }
        }
    }

    /// Copy samples from another buffer (pass-through)
    /// Both buffers must have same dimensions
    void copyFrom(const AudioBuffer& other) {
        if (other._numChannels != _numChannels || other._numFrames != _numFrames) {
            return; // Mismatch - skip
        }

        for (int ch = 0; ch < _numChannels; ++ch) {
            std::memcpy(_data[ch].data(), other._data[ch].data(), _numFrames * sizeof(float));
        }
    }

private:
    int _numChannels;
    int _numFrames;
    std::vector<std::vector<float>> _data; // [channel][frame]
};

