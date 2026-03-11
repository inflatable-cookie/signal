#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodePlay(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeStop(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeSeek(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeSetLoopEnabled(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeSetLoopRegion(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeTransportState(
    const nlohmann::json& payload,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeTransportPositionUpdate(
    const nlohmann::json& payload,
    std::string& error
);

void appendTransportPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "play",
        .kind = IpcKind::Command,
        .decode = &decodePlay,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "stop",
        .kind = IpcKind::Command,
        .decode = &decodeStop,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "seek",
        .kind = IpcKind::Command,
        .decode = &decodeSeek,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "setLoopEnabled",
        .kind = IpcKind::Command,
        .decode = &decodeSetLoopEnabled,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "setLoopRegion",
        .kind = IpcKind::Command,
        .decode = &decodeSetLoopRegion,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "state",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeTransportState,
    });

    out.push_back(PayloadCodec{
        .domain = "transport",
        .name = "positionUpdate",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeTransportPositionUpdate,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
