#pragma once

#include <libremidi/libremidi.hpp>
#include <cstdint>
#include <iomanip>
#include <sstream>
#include <string>
#include <string_view>

namespace loophole::signal::midi {
namespace {

inline std::string toHexByte(unsigned char value) {
    const char* digits = "0123456789abcdef";
    std::string out;
    out.push_back(digits[(value >> 4) & 0x0F]);
    out.push_back(digits[value & 0x0F]);
    return out;
}

inline std::string formatUuid(const libremidi::uuid& value) {
    std::string out;
    out.reserve(32);

    for (auto byte : value.bytes) {
        out += toHexByte(byte);
    }

    return out;
}

inline std::string formatUsbDeviceId(const libremidi::usb_device_identifier& value) {
    std::ostringstream out;
    out << "usb:" << std::hex << std::setw(4) << std::setfill('0')
        << value.vendor_id << ":" << std::setw(4) << value.product_id;
    return out.str();
}

inline std::string sanitizeSegment(std::string value) {
    for (auto& ch : value) {
        if (ch == ';' || ch == '=' || ch == '|') {
            ch = '_';
        }
    }

    return value;
}

inline std::uint64_t fnv1a64(std::string_view input) {
    std::uint64_t hash = 14695981039346656037ull;

    for (unsigned char byte : input) {
        hash ^= static_cast<std::uint64_t>(byte);
        hash *= 1099511628211ull;
    }

    return hash;
}

inline std::string formatHex64(std::uint64_t value) {
    std::ostringstream out;
    out << std::hex << std::setw(16) << std::setfill('0') << value;
    return out.str();
}

template <typename Variant>
inline std::string formatVariantIdentifier(const Variant& value) {
    struct VariantFormatter {
        std::string operator()(libremidi::monostate) const {
            return "";
        }

        std::string operator()(const std::string& v) const {
            return v;
        }

        std::string operator()(std::uint64_t v) const {
            return std::to_string(v);
        }

        std::string operator()(const libremidi::uuid& v) const {
            return formatUuid(v);
        }

        std::string operator()(const libremidi::usb_device_identifier& v) const {
            return formatUsbDeviceId(v);
        }

        template <typename T>
        std::string operator()(const T&) const {
            return "";
        }
    };

    return libremidi::visit(
        VariantFormatter{},
        value
    );
}

} // namespace

inline std::string buildStableMidiDeviceId(const libremidi::input_port& port) {
    std::ostringstream out;
    auto api_name = libremidi::get_api_name(port.api);
    out << "libremidi:" << api_name;

    auto container_id = formatVariantIdentifier(port.container);

    if (!container_id.empty()) {
        out << ";c=" << sanitizeSegment(container_id);
    }

    auto device_id = formatVariantIdentifier(port.device);

    if (!device_id.empty()) {
        out << ";d=" << sanitizeSegment(device_id);
    }

    if (!port.manufacturer.empty()) {
        out << ";m=" << sanitizeSegment(port.manufacturer);
    }

    if (!port.product.empty()) {
        out << ";prod=" << sanitizeSegment(port.product);
    }

    if (!port.serial.empty()) {
        out << ";sn=" << sanitizeSegment(port.serial);
    }

    if (port.port != static_cast<libremidi::port_handle>(-1)) {
        out << ";p=" << port.port;
    }

    std::string name = port.display_name;

    if (name.empty()) {
        name = port.port_name;
    }

    if (name.empty()) {
        name = port.device_name;
    }

    if (!name.empty()) {
        out << ";n=" << sanitizeSegment(name);
    }

    std::ostringstream hash_seed;
    hash_seed << api_name << "|" << port.manufacturer << "|" << port.product
              << "|" << port.serial << "|" << port.display_name << "|" << port.port_name
              << "|" << port.device_name;
    auto fallback_hash = fnv1a64(hash_seed.str());

    out << ";h=" << formatHex64(fallback_hash);

    return out.str();
}

inline std::string pickDeviceName(const libremidi::input_port& port, std::size_t fallbackIndex) {
    if (!port.display_name.empty()) {
        return port.display_name;
    }

    if (!port.port_name.empty()) {
        return port.port_name;
    }

    if (!port.device_name.empty()) {
        return port.device_name;
    }

    return "MIDI Input " + std::to_string(fallbackIndex + 1);
}

} // namespace loophole::signal::midi
