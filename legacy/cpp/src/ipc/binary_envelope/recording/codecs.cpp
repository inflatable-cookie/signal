#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {
namespace {

std::optional<nlohmann::json> decodeJsonPayload(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader r(payloadBytes);
        std::optional<std::string> payloadJson;

        while (const auto header = r.readNextHeader()) {
            const auto valueBytes = r.readValueBytes(header->byteLen);
            if (header->fieldId == 2 && header->fieldType == TLV_STRING) {
                payloadJson = readTlvString(valueBytes);
            }
        }

        if (!payloadJson.has_value()) {
            return nlohmann::json::object();
        }

        return nlohmann::json::parse(payloadJson.value());
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeJsonPayload(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        const auto payloadJson = payload.dump();
        TlvWriter w;
        w.writeU32(1, 1);
        w.writeString(2, payloadJson);
        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace

void appendRecordingPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "recording",
        .name = "setArmState",
        .kind = IpcKind::Command,
        .decode = &decodeJsonPayload,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "recording",
        .name = "startRecording",
        .kind = IpcKind::Command,
        .decode = &decodeJsonPayload,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "recording",
        .name = "stopRecording",
        .kind = IpcKind::Command,
        .decode = &decodeJsonPayload,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "recording",
        .name = "state",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeJsonPayload,
    });

    out.push_back(PayloadCodec{
        .domain = "recording",
        .name = "recordingFinished",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeJsonPayload,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
