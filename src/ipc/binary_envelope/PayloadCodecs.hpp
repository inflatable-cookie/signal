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

/// Returns `nullopt` when no typed TLV payload codec exists for the message.
/// When decoding fails, returns `nullopt` and sets `error`.
std::optional<nlohmann::json> decodeTypedPayload(
    std::string_view domain,
    std::string_view name,
    IpcKind kind,
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

/// Returns `nullopt` when no typed TLV payload codec exists for the message.
/// When encoding fails, returns `nullopt` and sets `error`.
std::optional<std::vector<std::uint8_t>> encodeTypedPayload(
    std::string_view domain,
    std::string_view name,
    IpcKind kind,
    const nlohmann::json& payload,
    std::string& error
);

} // namespace loophole::signal::ipc::binary_envelope
