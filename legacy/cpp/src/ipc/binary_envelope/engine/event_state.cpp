#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {
std::optional<nlohmann::json> decodeEngineState(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        nlohmann::json payload = nlohmann::json::object();

        TlvReader r(payloadBytes);
        while (true) {
            auto hOpt = r.readNextHeader();
            if (!hOpt.has_value()) {
                break;
            }
            TlvHeader h = hOpt.value();
            auto valueBytes = r.readValueBytes(h.byteLen);
            if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                payload["lifecycle"] = readTlvString(valueBytes);
            } else if (h.fieldId == 5 && h.fieldType == TLV_U32) {
                payload["sampleRate"] = readTlvU32(valueBytes);
            } else if (h.fieldId == 6 && h.fieldType == TLV_U32) {
                payload["bufferSize"] = readTlvU32(valueBytes);
            } else if (h.fieldId == 7 && h.fieldType == TLV_U32) {
                payload["numOutputChannels"] = readTlvU32(valueBytes);
            } else if (h.fieldId == 9 && h.fieldType == TLV_STRING) {
                payload["outputDeviceName"] = readTlvString(valueBytes);
            } else if (h.fieldId == 4 && h.fieldType == TLV_STRING) {
                payload["lastError"] = readTlvString(valueBytes);
            }
        }

        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeEngineState(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("lifecycle") && payload["lifecycle"].is_string()) {
            w.writeString(2, payload["lifecycle"].get<std::string>());
        }

        if (payload.contains("lastError") && payload["lastError"].is_string()) {
            w.writeString(4, payload["lastError"].get<std::string>());
        }

        if (payload.contains("sampleRate") && payload["sampleRate"].is_number()) {
            w.writeU32(5, static_cast<std::uint32_t>(payload["sampleRate"].get<double>()));
        }

        if (payload.contains("blockSize") && payload["blockSize"].is_number()) {
            w.writeU32(6, static_cast<std::uint32_t>(payload["blockSize"].get<std::uint64_t>()));
        }

        if (payload.contains("numOutputChannels") && payload["numOutputChannels"].is_number()) {
            w.writeU32(7, static_cast<std::uint32_t>(payload["numOutputChannels"].get<std::uint64_t>()));
        }

        if (payload.contains("outputDeviceName") && payload["outputDeviceName"].is_string()) {
            w.writeString(9, payload["outputDeviceName"].get<std::string>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
