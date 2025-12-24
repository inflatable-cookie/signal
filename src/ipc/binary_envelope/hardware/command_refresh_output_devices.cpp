#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
std::optional<nlohmann::json> decodeRefreshOutputDevices(
    std::span<const std::uint8_t>,
    std::string&
) {
    return nlohmann::json::object();
}
} // namespace loophole::signal::ipc::binary_envelope
