#pragma once

#include "ipc/IpcEnvelope.hpp"
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <vector>

namespace loophole::signal::ipc {

/// Decode a `binary-envelope-v2` payload (not including the outer LPF1 framing) into an `IpcEnvelope`.
///
/// Notes:
/// - This currently only decodes TLV payloads for a small subset of messages (pilot scope).
/// - Unknown payloads are decoded as an empty JSON object.
std::optional<IpcEnvelope> decodeBinaryEnvelopeV2(
    std::span<const std::uint8_t> bytes,
    std::string& error
);

/// Encode an `IpcEnvelope` into `binary-envelope-v2` payload bytes (not including LPF1 framing).
///
/// `payloadTlvBytes` must already be encoded as the per-message TLV payload.
std::optional<std::vector<std::uint8_t>> encodeBinaryEnvelopeV2(
    const IpcEnvelope& envelope,
    std::span<const std::uint8_t> payloadTlvBytes,
    std::string& error
);

/// Best-effort: encode a framed-binary envelope for messages that have TLV payload codecs in Signal.
///
/// Returns `nullopt` if the envelope is not supported yet (caller should fall back to JSON).
std::optional<std::vector<std::uint8_t>> tryEncodeBinaryEnvelopeV2(
    const IpcEnvelope& envelope,
    std::string& error
);

} // namespace loophole::signal::ipc
