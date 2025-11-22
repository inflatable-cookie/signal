#include "ipc/IpcEnvelope.hpp"
#include <chrono>
#include <iomanip>
#include <sstream>

namespace loophole::signal::ipc {

std::string originToString(IpcOrigin origin) {
    switch (origin) {
    case IpcOrigin::Aura:
        return "aura";
    case IpcOrigin::Pulse:
        return "pulse";
    case IpcOrigin::Signal:
        return "signal";
    case IpcOrigin::Composer:
        return "composer";
    }
    return "unknown";
}

std::optional<IpcOrigin> originFromString(std::string_view str) {
    if (str == "aura") return IpcOrigin::Aura;
    if (str == "pulse") return IpcOrigin::Pulse;
    if (str == "signal") return IpcOrigin::Signal;
    if (str == "composer") return IpcOrigin::Composer;
    return std::nullopt;
}

std::string targetToString(IpcTarget target) {
    switch (target) {
    case IpcTarget::Aura:
        return "aura";
    case IpcTarget::Pulse:
        return "pulse";
    case IpcTarget::Signal:
        return "signal";
    case IpcTarget::Composer:
        return "composer";
    }
    return "unknown";
}

std::optional<IpcTarget> targetFromString(std::string_view str) {
    if (str == "aura") return IpcTarget::Aura;
    if (str == "pulse") return IpcTarget::Pulse;
    if (str == "signal") return IpcTarget::Signal;
    if (str == "composer") return IpcTarget::Composer;
    return std::nullopt;
}

std::string kindToString(IpcKind kind) {
    switch (kind) {
    case IpcKind::Command:
        return "command";
    case IpcKind::Event:
        return "event";
    case IpcKind::Snapshot:
        return "snapshot";
    case IpcKind::Error:
        return "error";
    }
    return "unknown";
}

std::optional<IpcKind> kindFromString(std::string_view str) {
    if (str == "command") return IpcKind::Command;
    if (str == "event") return IpcKind::Event;
    if (str == "snapshot") return IpcKind::Snapshot;
    if (str == "error") return IpcKind::Error;
    return std::nullopt;
}

std::string priorityToString(IpcPriority priority) {
    switch (priority) {
    case IpcPriority::Realtime:
        return "realtime";
    case IpcPriority::High:
        return "high";
    case IpcPriority::Normal:
        return "normal";
    case IpcPriority::Low:
        return "low";
    }
    return "unknown";
}

std::optional<IpcPriority> priorityFromString(std::string_view str) {
    if (str == "realtime") return IpcPriority::Realtime;
    if (str == "high") return IpcPriority::High;
    if (str == "normal") return IpcPriority::Normal;
    if (str == "low") return IpcPriority::Low;
    return std::nullopt;
}

std::string currentTimestamp() {
    auto now = std::chrono::system_clock::now();
    auto time_t = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()
    ) % 1000;

    std::stringstream ss;
    ss << std::put_time(std::gmtime(&time_t), "%Y-%m-%dT%H:%M:%S");
    ss << '.' << std::setfill('0') << std::setw(3) << ms.count();
    ss << "Z";
    return ss.str();
}

} // namespace loophole::signal::ipc

