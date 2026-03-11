#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
std::optional<nlohmann::json> decodeSetLoopRegion(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader r(payloadBytes);
        std::optional<double> startBeats;
        std::optional<double> endBeats;

        while (true) {
            auto hOpt = r.readNextHeader();
            if (!hOpt.has_value()) {
                break;
            }

            TlvHeader h = hOpt.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_F64) {
                startBeats = readTlvF64(valueBytes);
            } else if (h.fieldId == 3 && h.fieldType == TLV_F64) {
                endBeats = readTlvF64(valueBytes);
            }
        }

        if (!startBeats.has_value()) {
            error = "transport.setLoopRegion missing startBeats";
            return std::nullopt;
        }

        if (!endBeats.has_value()) {
            error = "transport.setLoopRegion missing endBeats";
            return std::nullopt;
        }

        nlohmann::json payload = nlohmann::json::object();
        payload["startBeats"] = startBeats.value();
        payload["endBeats"] = endBeats.value();
        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
