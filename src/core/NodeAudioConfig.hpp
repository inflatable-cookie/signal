#pragma once

/// NodeAudioConfig - Channel layout configuration for audio nodes
///
/// Thread: Control thread (set during prepare)
///         Audio thread (read-only during process)
/// Ownership: Owned by GraphNode instances

/// Channel layout enumeration
enum class ChannelLayout {
    Mono,    // 1 channel
    Stereo,  // 2 channels
    // Future: Surround, Ambisonics, etc.
};

/// Audio configuration for a node
struct NodeAudioConfig {
    ChannelLayout layout;
    int numInputChannels;
    int numOutputChannels;

    NodeAudioConfig()
        : layout(ChannelLayout::Stereo)
        , numInputChannels(2)
        , numOutputChannels(2)
    {
    }

    NodeAudioConfig(ChannelLayout l, int inCh, int outCh)
        : layout(l)
        , numInputChannels(inCh)
        , numOutputChannels(outCh)
    {
    }
};

