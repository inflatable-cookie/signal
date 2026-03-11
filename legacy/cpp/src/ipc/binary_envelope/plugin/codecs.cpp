#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodePluginListCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);
std::optional<nlohmann::json> decodePluginRescanCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);
std::optional<nlohmann::json> decodePluginCancelScanCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);
std::optional<nlohmann::json> decodePluginScanStatusCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodePluginListEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginRescanEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginScanStartedEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginAddedEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginRemovedOrUpdatedEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginScanCompletedEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginScanFailedEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginScanStatusEvent(
    const nlohmann::json& payload,
    std::string& error
);
std::optional<std::vector<std::uint8_t>> encodePluginCancelScanEvent(
    const nlohmann::json& payload,
    std::string& error
);

void appendPluginPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "list",
        .kind = IpcKind::Command,
        .decode = &decodePluginListCommand,
        .encode = nullptr,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "rescan",
        .kind = IpcKind::Command,
        .decode = &decodePluginRescanCommand,
        .encode = nullptr,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "cancelScan",
        .kind = IpcKind::Command,
        .decode = &decodePluginCancelScanCommand,
        .encode = nullptr,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "scanStatus",
        .kind = IpcKind::Command,
        .decode = &decodePluginScanStatusCommand,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "list",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginListEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "rescan",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginRescanEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "scanStarted",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginScanStartedEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "added",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginAddedEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "removed",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginRemovedOrUpdatedEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "updated",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginRemovedOrUpdatedEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "scanCompleted",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginScanCompletedEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "scanFailed",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginScanFailedEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "scanStatus",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginScanStatusEvent,
    });
    out.push_back(PayloadCodec{
        .domain = "plugin",
        .name = "cancelScan",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodePluginCancelScanEvent,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
