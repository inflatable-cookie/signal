#include "domains/control/MidiControlNormaliser.hpp"
#include <sstream>

namespace loophole::signal::control {
namespace {

std::string toHexByte(std::uint8_t value) {
    std::ostringstream oss;
    oss << std::hex << static_cast<int>(value);
    return oss.str();
}

} // namespace

std::optional<NormalisedControlEvent> normaliseMidiMessage(
    std::uint8_t status,
    std::uint8_t data1,
    std::uint8_t data2
) {
    if (status >= 0xf8) {
        return NormalisedControlEvent{
            "midi:rt:" + toHexByte(status),
            "press",
            std::nullopt
        };
    }

    std::uint8_t type = status & 0xf0;
    std::uint8_t channel = static_cast<std::uint8_t>((status & 0x0f) + 1);

    if (type == 0x80) {
        return NormalisedControlEvent{
            "midi:note-off:" + std::to_string(data1) + ":" + std::to_string(channel),
            "release",
            std::nullopt
        };
    }

    if (type == 0x90) {
        if (data2 == 0) {
            return NormalisedControlEvent{
                "midi:note-off:" + std::to_string(data1) + ":" + std::to_string(channel),
                "release",
                std::nullopt
            };
        }

        return NormalisedControlEvent{
            "midi:note-on:" + std::to_string(data1) + ":" + std::to_string(channel),
            "press",
            static_cast<double>(data2)
        };
    }

    if (type == 0xa0) {
        return NormalisedControlEvent{
            "midi:poly-pressure:" + std::to_string(data1) + ":" + std::to_string(channel),
            "change",
            static_cast<double>(data2)
        };
    }

    if (type == 0xb0) {
        return NormalisedControlEvent{
            "midi:cc:" + std::to_string(data1) + ":" + std::to_string(channel),
            "change",
            static_cast<double>(data2)
        };
    }

    if (type == 0xc0) {
        return NormalisedControlEvent{
            "midi:pc:" + std::to_string(data1) + ":" + std::to_string(channel),
            "press",
            std::nullopt
        };
    }

    if (type == 0xd0) {
        return NormalisedControlEvent{
            "midi:ch-pressure:" + std::to_string(data1) + ":" + std::to_string(channel),
            "change",
            static_cast<double>(data1)
        };
    }

    if (type == 0xe0) {
        std::uint16_t value = static_cast<std::uint16_t>(data1 | (static_cast<std::uint16_t>(data2) << 7));
        return NormalisedControlEvent{
            "midi:pitch:" + std::to_string(value) + ":" + std::to_string(channel),
            "change",
            static_cast<double>(value)
        };
    }

    return std::nullopt;
}

} // namespace loophole::signal::control
