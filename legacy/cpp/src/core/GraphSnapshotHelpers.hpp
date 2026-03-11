#pragma once

/// GraphSnapshotHelpers - Helper functions for GraphSnapshot parsing
///
/// Thread: Control thread
/// Ownership: Utility functions
///
/// Provides conversion functions for parsing GraphSnapshot from JSON/IPC.
/// Includes backward compatibility for node kind names.

#include "core/GraphNode.hpp"
#include <string>
#include <optional>

/// Convert string to NodeKind (for JSON parsing)
/// Handles backward compatibility:
///   - "bus" → NodeKind::Receive
///   - "device" / "master" → NodeKind::HardwareAudioOutput
/// TODO: Remove backward compatibility once Pulse switches to new names
inline std::optional<NodeKind> nodeKindFromString(const std::string& str) {
    if (str == "midi-lane") return NodeKind::MidiLane;
    if (str == "audio-lane") return NodeKind::AudioLane;
    if (str == "midi-fx") return NodeKind::MidiFx;
    if (str == "instrument") return NodeKind::Instrument;
    if (str == "audio-fx") return NodeKind::AudioFx;
    if (str == "send") return NodeKind::Send;
    if (str == "fader") return NodeKind::Fader;
    if (str == "receive") return NodeKind::Receive;
    if (str == "bus") return NodeKind::Receive; // Backward compatibility - TODO: remove once Pulse switches
    if (str == "hardware-audio-output") return NodeKind::HardwareAudioOutput;
    if (str == "device") return NodeKind::HardwareAudioOutput; // Backward compatibility - TODO: remove once Pulse switches
    if (str == "master") return NodeKind::HardwareAudioOutput; // Backward compatibility - TODO: remove once Pulse switches
    if (str == "hardware-audio-input") return NodeKind::HardwareAudioInput;
    if (str == "audio-input") return NodeKind::HardwareAudioInput; // Legacy alias (early prototypes)
    if (str == "hardware-midi-input") return NodeKind::HardwareMidiInput;
    if (str == "midi-input") return NodeKind::HardwareMidiInput; // Legacy alias (early prototypes)
    if (str == "hardware-midi-output") return NodeKind::HardwareMidiOutput;
    return std::nullopt;
}

/// Convert NodeKind to string (for JSON serialization)
inline std::string nodeKindToString(NodeKind kind) {
    switch (kind) {
        case NodeKind::MidiLane: return "midi-lane";
        case NodeKind::AudioLane: return "audio-lane";
        case NodeKind::MidiFx: return "midi-fx";
        case NodeKind::Instrument: return "instrument";
        case NodeKind::AudioFx: return "audio-fx";
        case NodeKind::Send: return "send";
        case NodeKind::Fader: return "fader";
        case NodeKind::Receive: return "receive";
        case NodeKind::HardwareAudioOutput: return "hardware-audio-output";
        case NodeKind::HardwareAudioInput: return "hardware-audio-input";
        case NodeKind::HardwareMidiInput: return "hardware-midi-input";
        case NodeKind::HardwareMidiOutput: return "hardware-midi-output";
        default: return "unknown";
    }
}
