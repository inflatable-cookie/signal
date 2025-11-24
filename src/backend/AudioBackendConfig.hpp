#pragma once

/// AudioBackendConfig - Configuration for audio backend initialization
///
/// Thread: Main/control thread
/// Ownership: Created by application, passed to AudioBackend::initialise

#include <cstdint>
#include <string>
#include <optional>

struct AudioBackendConfig {
    // Device selection (optional - backend may use defaults if not specified)
    std::optional<std::string> inputDeviceId;
    std::optional<std::string> outputDeviceId;

    // Sample rate preference (backend may negotiate actual rate)
    double preferredSampleRate = 44100.0;

    // Buffer size preference (backend may negotiate actual size)
    int preferredBufferSize = 512;

    // Number of channels
    int numInputChannels = 0;   // 0 = no input
    int numOutputChannels = 2;   // Default to stereo output

    // Additional backend-specific options can be added here
};

