#pragma once

#include "ipc/IpcEnvelope.hpp"
#include <optional>
#include <string>
#include <string_view>

namespace loophole::signal::ipc {

/// Serialise an IPC envelope to a JSON line string
std::string serialiseEnvelope(const IpcEnvelope& env);

/// Deserialise an IPC envelope from a JSON line string
/// Returns nullopt if the line is invalid or missing required fields
std::optional<IpcEnvelope> deserialiseEnvelope(std::string_view line);

} // namespace loophole::signal::ipc

