#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeSetParameter(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

void appendNodePayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "node",
        .name = "setParameter",
        .kind = IpcKind::Command,
        .decode = &decodeSetParameter,
        .encode = nullptr,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
