#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeRegisterAudioAsset(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

void appendAssetsPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "assets",
        .name = "registerAudioAsset",
        .kind = IpcKind::Command,
        .decode = &decodeRegisterAudioAsset,
        .encode = nullptr,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
