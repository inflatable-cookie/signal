#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
std::optional<nlohmann::json> decodeSelectOutputDevice(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader r(payloadBytes);
        std::optional<std::string> id;

        while (true) {
            auto hOpt = r.readNextHeader();
            if (!hOpt.has_value()) {
                break;
            }

            TlvHeader h = hOpt.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                id = readTlvString(valueBytes);
            }
        }

        nlohmann::json payload = nlohmann::json::object();
        payload["id"] = id.value_or("");
        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
