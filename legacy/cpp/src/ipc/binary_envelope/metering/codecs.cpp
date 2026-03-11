#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeMeteringUpdate(
    const nlohmann::json& payload,
    std::string& error
);

void appendMeteringPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "metering",
        .name = "update",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeMeteringUpdate,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
