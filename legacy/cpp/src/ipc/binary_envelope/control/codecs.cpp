#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeControlDeviceInventory(
    const nlohmann::json& payload,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeControlEvent(
    const nlohmann::json& payload,
    std::string& error
);

void appendControlPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "control",
        .name = "deviceInventory",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeControlDeviceInventory,
    });

    out.push_back(PayloadCodec{
        .domain = "control",
        .name = "event",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeControlEvent,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
