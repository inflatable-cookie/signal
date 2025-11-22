#pragma once

#include <optional>
#include <string>
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc {

/// Origin of an IPC envelope
enum class IpcOrigin {
    Aura,
    Pulse,
    Signal,
    Composer,
};

/// Target of an IPC envelope
enum class IpcTarget {
    Aura,
    Pulse,
    Signal,
    Composer,
};

/// Kind of IPC envelope
enum class IpcKind {
    Command,
    Event,
    Snapshot,
    Error,
};

/// Priority hint for IPC envelope
enum class IpcPriority {
    Realtime,
    High,
    Normal,
    Low,
};

/// Convert IpcOrigin to string
std::string originToString(IpcOrigin origin);

/// Convert string to IpcOrigin
std::optional<IpcOrigin> originFromString(std::string_view str);

/// Convert IpcTarget to string
std::string targetToString(IpcTarget target);

/// Convert string to IpcTarget
std::optional<IpcTarget> targetFromString(std::string_view str);

/// Convert IpcKind to string
std::string kindToString(IpcKind kind);

/// Convert string to IpcKind
std::optional<IpcKind> kindFromString(std::string_view str);

/// Convert IpcPriority to string
std::string priorityToString(IpcPriority priority);

/// Convert string to IpcPriority
std::optional<IpcPriority> priorityFromString(std::string_view str);

/// Error information structure
struct IpcErrorInfo {
    std::string code;
    std::string message;
    nlohmann::json details;
};

/// IPC Envelope structure matching the Chorus IPC envelope specification
struct IpcEnvelope {
    int version = 1;
    std::string id;
    std::optional<std::string> correlationId;
    std::string timestamp; // ISO 8601, required

    IpcOrigin origin;
    IpcTarget target;
    std::string domain;
    IpcKind kind;
    std::string name;
    IpcPriority priority;

    nlohmann::json payload;
    std::optional<IpcErrorInfo> error;
};

/// Generate an ISO 8601 timestamp string for the current time
std::string currentTimestamp();

} // namespace loophole::signal::ipc

