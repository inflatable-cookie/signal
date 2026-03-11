#pragma once

/// MidiInputDeviceInfo - Information about a MIDI input device
///
/// Thread: Main/control thread
/// Ownership: Returned by backend enumeration methods

#include <optional>
#include <string>
#include <cstdint>

struct MidiInputDeviceInfo {
    std::string id;           // Stable identifier (backend-specific)
    std::string name;         // Human-readable name
    std::string manufacturer; // Vendor/manufacturer
    std::string api;          // Backend API name
    std::string container_id;
    std::string device_id;
    std::optional<std::uint64_t> port_handle;
    std::string port_name;
    std::string device_name;
    std::string display_name;
    std::string product;
    std::string serial;
    std::optional<std::uint64_t> last_seen_timestamp_ms;
    bool is_connected;        // Connection state
};
