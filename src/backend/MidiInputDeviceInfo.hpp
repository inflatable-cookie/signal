#pragma once

/// MidiInputDeviceInfo - Information about a MIDI input device
///
/// Thread: Main/control thread
/// Ownership: Returned by backend enumeration methods

#include <string>

struct MidiInputDeviceInfo {
    std::string id;           // Stable identifier (backend-specific)
    std::string name;         // Human-readable name
    std::string manufacturer; // Vendor/manufacturer
    bool is_connected;        // Connection state
};
