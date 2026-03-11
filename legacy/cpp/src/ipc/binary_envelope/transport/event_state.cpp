#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {
std::optional<std::vector<std::uint8_t>> encodeTransportState(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        if (!payload.contains("isPlaying") || !payload["isPlaying"].is_boolean()) {
            error = "transport.state missing isPlaying";
            return std::nullopt;
        }
        if (!payload.contains("positionBeats") || !payload["positionBeats"].is_number()) {
            error = "transport.state missing positionBeats";
            return std::nullopt;
        }
        if (!payload.contains("loopEnabled") || !payload["loopEnabled"].is_boolean()) {
            error = "transport.state missing loopEnabled";
            return std::nullopt;
        }

        TlvWriter w;
        w.writeU32(1, 1);
        w.writeBool(2, payload["isPlaying"].get<bool>());
        w.writeF64(3, payload["positionBeats"].get<double>());
        w.writeBool(4, payload["loopEnabled"].get<bool>());

        if (payload.contains("loopRegion") && payload["loopRegion"].is_object()) {
            const auto& lr = payload["loopRegion"];
            if (lr.contains("startBeats") && lr.contains("endBeats") && lr["startBeats"].is_number() && lr["endBeats"].is_number()) {
                w.writeObject(5, [&](TlvWriter& ww) {
                    ww.writeU32(1, 1);
                    ww.writeF64(2, lr["startBeats"].get<double>());
                    ww.writeF64(3, lr["endBeats"].get<double>());
                });
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
