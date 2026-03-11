#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeParameterSetValue(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader r(payloadBytes);
        nlohmann::json payload = nlohmann::json::object();

        while (true) {
            auto header = r.readNextHeader();
            if (!header.has_value()) {
                break;
            }

            auto h = header.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_OBJECT) {
                TlvReader scopeReader(valueBytes);
                nlohmann::json scope = nlohmann::json::object();

                while (true) {
                    auto scopeHeader = scopeReader.readNextHeader();
                    if (!scopeHeader.has_value()) {
                        break;
                    }

                    auto sh = scopeHeader.value();
                    auto scopeValueBytes = scopeReader.readValueBytes(sh.byteLen);

                    if (sh.fieldId == 2 && sh.fieldType == TLV_STRING) {
                        scope["nodeId"] = readTlvString(scopeValueBytes);
                    } else if (sh.fieldId == 3 && sh.fieldType == TLV_STRING) {
                        scope["pluginInstanceId"] = readTlvString(scopeValueBytes);
                    }
                }

                payload["scope"] = scope;
            } else if (h.fieldId == 3 && h.fieldType == TLV_STRING) {
                payload["paramId"] = readTlvString(valueBytes);
            } else if (h.fieldId == 4 && h.fieldType == TLV_F64) {
                payload["value"] = readTlvF64(valueBytes);
            }
        }

        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
