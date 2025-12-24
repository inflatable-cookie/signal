#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {
std::optional<std::vector<std::uint8_t>> encodeTransportPositionUpdate(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        if (!payload.contains("positionSamples") || !payload["positionSamples"].is_number_unsigned()) {
            error = "transport.positionUpdate missing positionSamples";
            return std::nullopt;
        }

        TlvWriter w;
        w.writeU32(1, 1);
        w.writeU64(2, payload["positionSamples"].get<std::uint64_t>());

        if (payload.contains("sampleRate") && payload["sampleRate"].is_number()) {
            w.writeF64(3, payload["sampleRate"].get<double>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
