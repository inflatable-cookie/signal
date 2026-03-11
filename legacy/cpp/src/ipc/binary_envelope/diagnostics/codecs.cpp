#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeDiagnosticsEngineMetrics(
    const nlohmann::json& payload,
    std::string& error
);

void appendDiagnosticsPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "diagnostics",
        .name = "engineMetrics",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeDiagnosticsEngineMetrics,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
