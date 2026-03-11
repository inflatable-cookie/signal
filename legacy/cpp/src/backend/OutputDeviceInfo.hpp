#pragma once

/// OutputDeviceInfo - Information about an audio output device
///
/// Thread: Main/control thread
/// Ownership: Returned by backend enumeration methods

#include <string>
#include <cstdint>

struct OutputDeviceInfo {
    std::string id;          // Stable identifier (e.g. miniaudio ID or derived string)
    std::string name;        // Human-readable device name
    bool isDefault;          // Whether this is the OS default
    uint32_t maxChannels;    // Maximum output channels supported
    uint32_t preferredSampleRate; // Preferred sample rate (may be 0 if unknown)
};

