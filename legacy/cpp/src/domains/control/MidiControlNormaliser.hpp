#pragma once

#include <cstdint>
#include <optional>
#include <string>

namespace loophole::signal::control {

struct NormalisedControlEvent {
    std::string control_key;
    std::string action;
    std::optional<double> value;
};

std::optional<NormalisedControlEvent> normaliseMidiMessage(
    std::uint8_t status,
    std::uint8_t data1,
    std::uint8_t data2
);

} // namespace loophole::signal::control
