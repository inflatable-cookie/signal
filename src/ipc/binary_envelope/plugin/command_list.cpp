#include "ipc/binary_envelope/CodecCommon.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodePluginListCommand(
    std::span<const std::uint8_t>,
    std::string&
) {
    return nlohmann::json::object();
}

} // namespace loophole::signal::ipc::binary_envelope
