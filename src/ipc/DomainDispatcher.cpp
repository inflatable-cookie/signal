#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelopeCodec.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc {

DomainDispatcher::DomainDispatcher(IpcRouter* router) : router_(router) {
}

void DomainDispatcher::handleEnvelope(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    std::cout << "[DomainDispatcher] Received envelope: " << env.domain << "." << env.name
              << " (kind: " << kindToString(env.kind) << ")" << std::endl;

    if (env.domain == "engine") {
        handleEngineDomain(env, session);
    } else if (env.domain == "transport") {
        handleTransportDomain(env, session);
    } else {
        handleUnknownDomain(env, session);
    }
}

void DomainDispatcher::handleEngineDomain(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    // Convert new IpcEnvelope to old Envelope for existing router
    Envelope old_env;
    old_env.v = env.version;
    old_env.id = env.id;
    old_env.cid = env.correlationId.value_or("");
    old_env.ts = env.timestamp;
    old_env.origin = originToString(env.origin);
    old_env.target = targetToString(env.target);
    old_env.domain = env.domain;
    old_env.kind = kindToString(env.kind);
    old_env.name = env.name;
    old_env.priority = priorityToString(env.priority);
    old_env.payload = env.payload.dump();

    // Dispatch to router
    router_->dispatch(old_env);

    // For now, echo back an acknowledgement if this was a command
    if (env.kind == IpcKind::Command) {
        IpcEnvelope reply;
        reply.version = 1;
        reply.id = "reply-" + env.id;
        reply.correlationId = env.id;
        reply.timestamp = currentTimestamp();
        reply.origin = IpcOrigin::Signal;
        // Convert origin to target for reply
        switch (env.origin) {
        case IpcOrigin::Aura:
            reply.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            reply.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            reply.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            reply.target = IpcTarget::Composer;
            break;
        }
        reply.domain = env.domain;
        reply.kind = IpcKind::Event;
        reply.name = env.name; // Event name matches command name per spec
        reply.priority = env.priority;
        reply.payload = nlohmann::json::object();

        session->send(reply);
    }
}

void DomainDispatcher::handleTransportDomain(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    // Convert and dispatch similar to engine domain
    Envelope old_env;
    old_env.v = env.version;
    old_env.id = env.id;
    old_env.cid = env.correlationId.value_or("");
    old_env.ts = env.timestamp;
    old_env.origin = originToString(env.origin);
    old_env.target = targetToString(env.target);
    old_env.domain = env.domain;
    old_env.kind = kindToString(env.kind);
    old_env.name = env.name;
    old_env.priority = priorityToString(env.priority);
    old_env.payload = env.payload.dump();

    router_->dispatch(old_env);

    // Echo back acknowledgement if this was a command
    if (env.kind == IpcKind::Command) {
        IpcEnvelope reply;
        reply.version = 1;
        reply.id = "reply-" + env.id;
        reply.correlationId = env.id;
        reply.timestamp = currentTimestamp();
        reply.origin = IpcOrigin::Signal;
        // Convert origin to target for reply
        switch (env.origin) {
        case IpcOrigin::Aura:
            reply.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            reply.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            reply.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            reply.target = IpcTarget::Composer;
            break;
        }
        reply.domain = env.domain;
        reply.kind = IpcKind::Event;
        reply.name = env.name;
        reply.priority = env.priority;
        reply.payload = nlohmann::json::object();

        session->send(reply);
    }
}

void DomainDispatcher::handleUnknownDomain(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    std::cout << "[DomainDispatcher] Unknown domain: " << env.domain << std::endl;

    // Send error response for commands
    if (env.kind == IpcKind::Command) {
        IpcEnvelope error_reply;
        error_reply.version = 1;
        error_reply.id = "error-" + env.id;
        error_reply.correlationId = env.id;
        error_reply.timestamp = currentTimestamp();
        error_reply.origin = IpcOrigin::Signal;
        // Convert origin to target for error reply
        switch (env.origin) {
        case IpcOrigin::Aura:
            error_reply.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            error_reply.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            error_reply.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            error_reply.target = IpcTarget::Composer;
            break;
        }
        error_reply.domain = env.domain;
        error_reply.kind = IpcKind::Error;
        error_reply.name = env.name;
        error_reply.priority = env.priority;
        error_reply.payload = nlohmann::json::object();

        IpcErrorInfo error_info;
        error_info.code = "unknown_domain";
        error_info.message = "No handler registered for domain: " + env.domain;
        error_info.details = nlohmann::json::object();
        error_reply.error = std::make_optional(error_info);

        session->send(error_reply);
    }
}

} // namespace loophole::signal::ipc

