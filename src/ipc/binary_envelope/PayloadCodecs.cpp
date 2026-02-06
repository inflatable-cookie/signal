#include "ipc/binary_envelope/PayloadCodecs.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {
namespace {

const std::vector<PayloadCodec>& allCodecs() {
    static const std::vector<PayloadCodec> codecs = []() {
        std::vector<PayloadCodec> out;
        out.reserve(32);
        appendAssetsPayloadCodecs(out);
        appendAutomationPayloadCodecs(out);
        appendControlPayloadCodecs(out);
        appendDiagnosticsPayloadCodecs(out);
        appendEnginePayloadCodecs(out);
        appendHardwarePayloadCodecs(out);
        appendMeteringPayloadCodecs(out);
        appendNodePayloadCodecs(out);
        appendParameterPayloadCodecs(out);
        appendTransportPayloadCodecs(out);
        return out;
    }();

    return codecs;
}

const PayloadCodec* findCodec(std::string_view domain, std::string_view name, IpcKind kind) {
    for (const auto& codec : allCodecs()) {
        if (codec.domain != domain) {
            continue;
        }

        if (codec.name != name) {
            continue;
        }

        if (codec.kind != kind) {
            continue;
        }

        return &codec;
    }

    return nullptr;
}

} // namespace

std::optional<nlohmann::json> decodeTypedPayload(
    std::string_view domain,
    std::string_view name,
    IpcKind kind,
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    const auto* codec = findCodec(domain, name, kind);
    if (codec == nullptr || codec->decode == nullptr) {
        return std::nullopt;
    }

    return codec->decode(payloadBytes, error);
}

std::optional<std::vector<std::uint8_t>> encodeTypedPayload(
    std::string_view domain,
    std::string_view name,
    IpcKind kind,
    const nlohmann::json& payload,
    std::string& error
) {
    const auto* codec = findCodec(domain, name, kind);
    if (codec == nullptr || codec->encode == nullptr) {
        return std::nullopt;
    }

    return codec->encode(payload, error);
}

} // namespace loophole::signal::ipc::binary_envelope
