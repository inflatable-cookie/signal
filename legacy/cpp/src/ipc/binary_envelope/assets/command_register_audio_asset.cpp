#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
namespace {

nlohmann::json decodeAssetsRegisterAudioAssetPayload(std::span<const std::uint8_t> tlvBytes) {
    TlvReader r(tlvBytes);
    std::optional<std::string> assetId;
    std::optional<std::string> path;
    std::optional<std::uint32_t> channels;
    std::optional<std::uint32_t> sampleRate;
    std::optional<std::uint64_t> frames;

    while (true) {
        auto hOpt = r.readNextHeader();
        if (!hOpt.has_value()) {
            break;
        }

        TlvHeader h = hOpt.value();
        auto valueBytes = r.readValueBytes(h.byteLen);

        if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
            assetId = readTlvString(valueBytes);
            continue;
        }

        if (h.fieldId == 3 && h.fieldType == TLV_STRING) {
            path = readTlvString(valueBytes);
            continue;
        }

        if (h.fieldId == 4 && h.fieldType == TLV_U32) {
            channels = readTlvU32(valueBytes);
            continue;
        }

        if (h.fieldId == 5 && h.fieldType == TLV_U32) {
            sampleRate = readTlvU32(valueBytes);
            continue;
        }

        if (h.fieldId == 6 && h.fieldType == TLV_U64) {
            frames = readTlvU64(valueBytes);
            continue;
        }
    }

    nlohmann::json out = nlohmann::json::object();
    if (assetId.has_value()) {
        out["assetId"] = assetId.value();
    }
    if (path.has_value()) {
        out["path"] = path.value();
    }
    if (channels.has_value()) {
        out["channels"] = channels.value();
    }
    if (sampleRate.has_value()) {
        out["sampleRate"] = sampleRate.value();
    }
    if (frames.has_value()) {
        out["frames"] = frames.value();
    }
    return out;
}

} // namespace

std::optional<nlohmann::json> decodeRegisterAudioAsset(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        return decodeAssetsRegisterAudioAssetPayload(payloadBytes);
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
