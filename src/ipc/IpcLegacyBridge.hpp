#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/Envelope.hpp"

namespace loophole::signal::ipc {

/// Convert IpcEnvelope to legacy Envelope format
/// Used by domains that still need to route through IpcRouter
inline Envelope toLegacyEnvelope(const IpcEnvelope& env) {
    Envelope oldEnv;
    oldEnv.v = env.version;
    oldEnv.id = env.id;
    oldEnv.cid = env.correlationId.value_or("");
    oldEnv.ts = env.timestamp;
    oldEnv.origin = originToString(env.origin);
    oldEnv.target = targetToString(env.target);
    oldEnv.domain = env.domain;
    oldEnv.kind = kindToString(env.kind);
    oldEnv.name = env.name;
    oldEnv.priority = priorityToString(env.priority);
    oldEnv.payload = env.payload.dump();
    return oldEnv;
}

} // namespace loophole::signal::ipc

