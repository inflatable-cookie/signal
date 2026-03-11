#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeAutomationSnapshot(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

void appendAutomationPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "automation",
        .name = "automationSnapshot",
        .kind = IpcKind::Command,
        .decode = &decodeAutomationSnapshot,
        .encode = nullptr,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
