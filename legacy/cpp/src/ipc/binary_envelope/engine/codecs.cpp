#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeGraphSnapshot(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodePlaybackScheduleSnapshot(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeSchemaOnly(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeEngineState(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeEngineState(
    const nlohmann::json& payload,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeEngineSelfTestResult(
    const nlohmann::json& payload,
    std::string& error
);

void appendEnginePayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "engine",
        .name = "graphSnapshot",
        .kind = IpcKind::Command,
        .decode = &decodeGraphSnapshot,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "engine",
        .name = "playbackScheduleSnapshot",
        .kind = IpcKind::Command,
        .decode = &decodePlaybackScheduleSnapshot,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "engine",
        .name = "state",
        .kind = IpcKind::Event,
        .decode = &decodeEngineState,
        .encode = &encodeEngineState,
    });

    out.push_back(PayloadCodec{
        .domain = "engine",
        .name = "selfTestResult",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeEngineSelfTestResult,
    });

    for (auto schemaOnlyName : {"start", "stop", "reset", "shutdown", "heartbeat", "selfTest"}) {
        out.push_back(PayloadCodec{
            .domain = "engine",
            .name = schemaOnlyName,
            .kind = IpcKind::Command,
            .decode = &decodeSchemaOnly,
            .encode = nullptr,
        });
    }
}

} // namespace loophole::signal::ipc::binary_envelope
