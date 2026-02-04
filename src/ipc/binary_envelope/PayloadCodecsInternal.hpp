#pragma once

#include "ipc/IpcEnvelope.hpp"
#include <cstdint>
#include <nlohmann/json.hpp>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace loophole::signal::ipc::binary_envelope {

using DecodeFn = std::optional<nlohmann::json>(*)(
    std::span<const std::uint8_t>,
    std::string&
);

using EncodeFn = std::optional<std::vector<std::uint8_t>>(*)(
    const nlohmann::json&,
    std::string&
);

struct PayloadCodec {
    std::string_view domain;
    std::string_view name;
    IpcKind kind;
    DecodeFn decode;
    EncodeFn encode;
};

void appendAssetsPayloadCodecs(std::vector<PayloadCodec>& out);
void appendAutomationPayloadCodecs(std::vector<PayloadCodec>& out);
void appendControlPayloadCodecs(std::vector<PayloadCodec>& out);
void appendDiagnosticsPayloadCodecs(std::vector<PayloadCodec>& out);
void appendEnginePayloadCodecs(std::vector<PayloadCodec>& out);
void appendHardwarePayloadCodecs(std::vector<PayloadCodec>& out);
void appendMeteringPayloadCodecs(std::vector<PayloadCodec>& out);
void appendNodePayloadCodecs(std::vector<PayloadCodec>& out);
void appendTransportPayloadCodecs(std::vector<PayloadCodec>& out);

} // namespace loophole::signal::ipc::binary_envelope
