#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeControlEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("timestampMs") && payload["timestampMs"].is_number_unsigned()) {
            w.writeU64(2, payload["timestampMs"].get<std::uint64_t>());
        }
        if (payload.contains("deviceId") && payload["deviceId"].is_string()) {
            w.writeString(3, payload["deviceId"].get<std::string>());
        }
        if (payload.contains("controlKey") && payload["controlKey"].is_string()) {
            w.writeString(4, payload["controlKey"].get<std::string>());
        }
        if (payload.contains("controlId") && payload["controlId"].is_string()) {
            w.writeString(5, payload["controlId"].get<std::string>());
        }
        if (payload.contains("action") && payload["action"].is_string()) {
            w.writeString(6, payload["action"].get<std::string>());
        }
        if (payload.contains("value") && payload["value"].is_number()) {
            w.writeF64(7, payload["value"].get<double>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
