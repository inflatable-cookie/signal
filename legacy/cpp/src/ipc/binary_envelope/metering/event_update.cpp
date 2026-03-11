#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <limits>

namespace loophole::signal::ipc::binary_envelope {
std::optional<std::vector<std::uint8_t>> encodeMeteringUpdate(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        if (!payload.contains("channels") || !payload["channels"].is_array()) {
            error = "metering.update missing channels";
            return std::nullopt;
        }

        std::vector<std::vector<std::uint8_t>> channels;
        for (const auto& ch : payload["channels"]) {
            if (!ch.is_object()) {
                continue;
            }

            if (!ch.contains("channelId") || !ch["channelId"].is_string()) {
                error = "metering.update.channels[] missing channelId";
                return std::nullopt;
            }
            if (!ch.contains("peak") || !ch["peak"].is_number()) {
                error = "metering.update.channels[] missing peak";
                return std::nullopt;
            }
            if (!ch.contains("rms") || !ch["rms"].is_number()) {
                error = "metering.update.channels[] missing rms";
                return std::nullopt;
            }
            if (!ch.contains("timestamp") || !ch["timestamp"].is_number_unsigned()) {
                if (!ch.contains("timestamp") || !ch["timestamp"].is_number_integer()) {
                    error = "metering.update.channels[] missing timestamp";
                    return std::nullopt;
                }
                auto ts_signed = ch["timestamp"].get<std::int64_t>();
                if (ts_signed < 0) {
                    error = "metering.update.channels[] timestamp must be non-negative";
                    return std::nullopt;
                }
            }

            TlvWriter cw;
            cw.writeU32(1, 1);
            cw.writeString(2, ch["channelId"].get<std::string>());
            cw.writeF64(3, ch["peak"].get<double>());
            cw.writeF64(4, ch["rms"].get<double>());
            if (ch["timestamp"].is_number_unsigned()) {
                cw.writeU64(5, ch["timestamp"].get<std::uint64_t>());
            } else {
                cw.writeU64(5, static_cast<std::uint64_t>(ch["timestamp"].get<std::int64_t>()));
            }
            channels.push_back(cw.intoBytes());
        }

        TlvWriter w;
        w.writeU32(1, 1);
        w.writeObjectList(2, channels);
        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
