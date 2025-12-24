#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
namespace {

} // namespace

std::optional<nlohmann::json> decodeSetParameter(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader r(payloadBytes);
        std::optional<std::string> nodeId;
        std::optional<std::string> parameterId;
        std::optional<double> value;
        std::optional<bool> valueBool;

        while (true) {
            auto hOpt = r.readNextHeader();
            if (!hOpt.has_value()) {
                break;
            }

            TlvHeader h = hOpt.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                nodeId = readTlvString(valueBytes);
            } else if (h.fieldId == 3 && h.fieldType == TLV_STRING) {
                parameterId = readTlvString(valueBytes);
            } else if (h.fieldId == 4 && h.fieldType == TLV_F64) {
                value = readTlvF64(valueBytes);
            } else if (h.fieldId == 5 && h.fieldType == TLV_BOOL) {
                valueBool = readTlvBool(valueBytes);
            }
        }

        nlohmann::json payload = nlohmann::json::object();
        payload["nodeId"] = nodeId.value_or("");
        payload["parameterId"] = parameterId.value_or("");

        if (valueBool.has_value()) {
            payload["value"] = valueBool.value();
        } else {
            payload["value"] = value.value_or(0.0);
        }

        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
