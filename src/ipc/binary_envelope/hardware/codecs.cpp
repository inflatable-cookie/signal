#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeRefreshOutputDevices(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeSelectOutputDevice(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeHardwareState(
    const nlohmann::json& payload,
    std::string& error
);

void appendHardwarePayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "hardware",
        .name = "refreshOutputDevices",
        .kind = IpcKind::Command,
        .decode = &decodeRefreshOutputDevices,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "hardware",
        .name = "selectOutputDevice",
        .kind = IpcKind::Command,
        .decode = &decodeSelectOutputDevice,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "hardware",
        .name = "state",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeHardwareState,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
