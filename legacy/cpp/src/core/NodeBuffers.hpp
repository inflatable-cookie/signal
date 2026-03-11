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
    /// Channel-aware: handles channel count mismatches with upmix/downmix rules
    /// - Exact match: direct sum per channel
    /// - Source < target: upmix by duplicating last source channel
    /// - Source > target: downmix by truncating (sum first N channels)
    /// Both buffers must have same frame count
    void sumFrom(const AudioBuffer& other) {
        if (other._numFrames != _numFrames) {
            return; // Frame count mismatch - skip
        }

        const int sourceChannels = other._numChannels;
        const int targetChannels = _numChannels;
        const int numFrames = _numFrames;

        if (sourceChannels == targetChannels) {
            // Exact match: direct sum per channel
            for (int ch = 0; ch < _numChannels; ++ch) {
                const float* src = other._data[ch].data();
                float* dst = _data[ch].data();
                for (int frame = 0; frame < numFrames; ++frame) {
                    dst[frame] += src[frame];
                }
            }
        } else if (sourceChannels < targetChannels) {
            // Upmix: copy available channels, duplicate last channel to remaining
            for (int ch = 0; ch < sourceChannels; ++ch) {
                const float* src = other._data[ch].data();
                float* dst = _data[ch].data();
                for (int frame = 0; frame < numFrames; ++frame) {
                    dst[frame] += src[frame];
                }
            }
            // Duplicate last source channel to remaining target channels
            if (sourceChannels > 0) {
                const float* lastChannel = other._data[sourceChannels - 1].data();
                for (int ch = sourceChannels; ch < targetChannels; ++ch) {
                    float* dst = _data[ch].data();
                    for (int frame = 0; frame < numFrames; ++frame) {
                        dst[frame] += lastChannel[frame];
                    }
                }
            }
        } else {
            // Downmix: sum first N channels (truncate extra channels)
            const int channelsToSum = targetChannels;
            for (int ch = 0; ch < channelsToSum; ++ch) {
                const float* src = other._data[ch].data();
                float* dst = _data[ch].data();
                for (int frame = 0; frame < numFrames; ++frame) {
                    dst[frame] += src[frame];
                }
            }
            // Remaining source channels are dropped (deterministic truncation)
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

    /// Copy deinterleaved buffer to interleaved format
    /// Real-time safe: no allocations, writes directly to destination
    /// @param dest Destination interleaved buffer (must have space for numChannels * numFrames samples)
    /// @param destNumChannels Number of channels in destination (must match or be larger)
    /// @param destNumFrames Number of frames in destination (must match or be larger)
    void copyToInterleaved(float* dest, int destNumChannels, int destNumFrames) const {
        if (!dest) {
            return;
        }

        const int numChannels = std::min(_numChannels, destNumChannels);
        const int numFrames = std::min(_numFrames, destNumFrames);

        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels; ++ch) {
                dest[frame * destNumChannels + ch] = _data[ch][frame];
            }
            // Zero-pad remaining channels if destination has more channels
            for (int ch = numChannels; ch < destNumChannels; ++ch) {
                dest[frame * destNumChannels + ch] = 0.0f;
            }
        }
    }

    /// Copy from interleaved format to deinterleaved buffer
    /// Real-time safe: no allocations, writes directly to internal buffers
    /// @param src Source interleaved buffer
    /// @param srcNumChannels Number of channels in source
    /// @param srcNumFrames Number of frames in source
    /// @param destChannelOffset Channel offset in destination (default 0)
    /// @param destFrameOffset Frame offset in destination (default 0)
    void copyFromInterleaved(
        const float* src,
        int srcNumChannels,
        int srcNumFrames,
        int destChannelOffset = 0,
        int destFrameOffset = 0
    ) {
        if (!src) {
            return;
        }

        const int numChannels = std::min(srcNumChannels, _numChannels - destChannelOffset);
        const int numFrames = std::min(srcNumFrames, _numFrames - destFrameOffset);

        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels; ++ch) {
                int srcIndex = frame * srcNumChannels + ch;
                int destCh = destChannelOffset + ch;
                int destFrame = destFrameOffset + frame;
                if (destCh >= 0 && destCh < _numChannels && destFrame >= 0 && destFrame < _numFrames) {
                    _data[destCh][destFrame] = src[srcIndex];
                }
            }
        }
    }

private:
    int _numChannels;
    int _numFrames;
    std::vector<std::vector<float>> _data; // [channel][frame]
};

