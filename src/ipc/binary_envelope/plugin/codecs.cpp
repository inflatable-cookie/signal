#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodePluginListCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodePluginListEvent(
    const nlohmann::json& payload,
    std::string& error
);

void appendPluginPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "list",
        .kind = IpcKind::Command,
        .decode = &decodePluginListCommand,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "list",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginListEvent,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
