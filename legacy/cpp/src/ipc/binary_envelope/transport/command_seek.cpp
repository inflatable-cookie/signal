#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
std::optional<nlohmann::json> decodeSeek(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader r(payloadBytes);
        std::optional<double> positionBeats;

        while (true) {
            auto hOpt = r.readNextHeader();
            if (!hOpt.has_value()) {
                break;
            }

            TlvHeader h = hOpt.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_F64) {
                positionBeats = readTlvF64(valueBytes);
            }
        }

        if (!positionBeats.has_value()) {
            error = "transport.seek missing positionBeats";
            return std::nullopt;
        }

        nlohmann::json payload = nlohmann::json::object();
        payload["positionBeats"] = positionBeats.value();
        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
