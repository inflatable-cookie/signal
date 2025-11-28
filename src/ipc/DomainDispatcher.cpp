/// DomainDispatcher - Routes IPC envelopes to domain handlers
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Local to SignalApp::run() (created in run method)
/// Communication:
///   - Receives envelopes from TcpClientSession handlers
///   - Dispatches to domain handlers synchronously
///   - Domain handlers update EngineHost/TransportState directly
///   - Sends events back to Pulse via TcpClientSession

#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelopeCodec.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/Envelope.hpp"
#include "core/EngineHost.hpp"
#include "domains/HardwareDomain.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc {

DomainDispatcher::DomainDispatcher(IpcRouter* router, EngineHost* engineHost)
    : router_(router)
    , engineHost_(engineHost)
    , hardwareDomain_(std::make_unique<HardwareDomain>(engineHost))
{
}

void DomainDispatcher::handleEnvelope(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    if (env.domain == "engine") {
        handleEngineDomain(env, session);
    } else if (env.domain == "transport") {
        handleTransportDomain(env, session);
    } else if (env.domain == "hardware") {
        handleHardwareDomain(env, session);
    } else {
        // Route other domains (automation, mixer, metering, etc.) through the router
        handleGenericDomain(env, session);
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

    // Dispatch to router (this will call EngineDomain::handle which updates EngineHost)
    router_->dispatch(old_env);

    // Send engine.state event after processing commands
    // Note: For shutdown command, we emit a final state event but don't stop the process
    // The process should be stopped by Pulse or via SIGINT/SIGTERM
    if (env.kind == IpcKind::Command && env.domain == "engine") {
        IpcEnvelope stateEvent;
        stateEvent.version = 1;
        stateEvent.id = "engine-state-" + env.id;
        stateEvent.correlationId = env.id;
        stateEvent.timestamp = currentTimestamp();
        stateEvent.origin = IpcOrigin::Signal;

        // Convert origin to target for event
        switch (env.origin) {
        case IpcOrigin::Aura:
            stateEvent.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            stateEvent.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            stateEvent.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            stateEvent.target = IpcTarget::Composer;
            break;
        }

        stateEvent.domain = "engine";
        stateEvent.kind = IpcKind::Event;
        stateEvent.name = "state";
        stateEvent.priority = env.priority;

        // Get current engine state and create payload
        // Pulse expects "lifecycle" field matching: "stopped", "starting", "running", "error"
        std::string lifecycle = "stopped";
        std::optional<std::string> lastError;
        if (engineHost_) {
            switch (engineHost_->state()) {
            case EngineHost::State::Stopped:
                lifecycle = "stopped";
                break;
            case EngineHost::State::Starting:
                lifecycle = "starting";
                break;
            case EngineHost::State::Running:
                lifecycle = "running";
                break;
            case EngineHost::State::Error:
                lifecycle = "error";
                lastError = engineHost_->lastError();
                break;
            }
        }

        nlohmann::json payload;
        payload["lifecycle"] = lifecycle;
        if (lastError.has_value()) {
            payload["lastError"] = lastError.value();
        } else {
            payload["lastError"] = nullptr;
        }
        // Include runtime configuration in state event
        if (engineHost_) {
            payload["sampleRate"] = engineHost_->getSampleRate();
            payload["blockSize"] = engineHost_->getBlockSize();
            payload["outputDeviceName"] = engineHost_->getOutputDeviceName();
            payload["numOutputChannels"] = engineHost_->getNumOutputChannels();
        }

        stateEvent.payload = payload;

        session->send(stateEvent);
    } else if (env.kind == IpcKind::Command && env.name == "heartbeat") {
        // Respond to heartbeat with current state
        IpcEnvelope heartbeatEvent;
        heartbeatEvent.version = 1;
        heartbeatEvent.id = "engine-heartbeat-" + env.id;
        heartbeatEvent.correlationId = env.id;
        heartbeatEvent.timestamp = currentTimestamp();
        heartbeatEvent.origin = IpcOrigin::Signal;

        switch (env.origin) {
        case IpcOrigin::Aura:
            heartbeatEvent.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            heartbeatEvent.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            heartbeatEvent.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            heartbeatEvent.target = IpcTarget::Composer;
            break;
        }

        heartbeatEvent.domain = "engine";
        heartbeatEvent.kind = IpcKind::Event;
        heartbeatEvent.name = "heartbeat";
        heartbeatEvent.priority = env.priority;

        std::string lifecycle = "stopped";
        if (engineHost_) {
            switch (engineHost_->state()) {
            case EngineHost::State::Stopped:
                lifecycle = "stopped";
                break;
            case EngineHost::State::Starting:
                lifecycle = "starting";
                break;
            case EngineHost::State::Running:
                lifecycle = "running";
                break;
            case EngineHost::State::Error:
                lifecycle = "error";
                break;
            }
        }

        nlohmann::json payload;
        payload["lifecycle"] = lifecycle;
        heartbeatEvent.payload = payload;

        session->send(heartbeatEvent);
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

    // Send transport.state event after processing commands
    if (env.kind == IpcKind::Command && env.domain == "transport") {
        IpcEnvelope stateEvent;
        stateEvent.version = 1;
        stateEvent.id = "transport-state-" + env.id;
        stateEvent.correlationId = env.id;
        stateEvent.timestamp = currentTimestamp();
        stateEvent.origin = IpcOrigin::Signal;

        switch (env.origin) {
        case IpcOrigin::Aura:
            stateEvent.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            stateEvent.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            stateEvent.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            stateEvent.target = IpcTarget::Composer;
            break;
        }

        stateEvent.domain = "transport";
        stateEvent.kind = IpcKind::Event;
        stateEvent.name = "state";
        stateEvent.priority = env.priority;

        // Get current transport state and create payload
        nlohmann::json payload;
        if (engineHost_) {
            const auto& transport = engineHost_->transport();
            payload["isPlaying"] = transport.isPlaying;
            // Convert seconds to beats using real tempo
            double tempo = transport.tempo;
            payload["positionBeats"] = (transport.positionSeconds / 60.0) * tempo;
            payload["loopEnabled"] = transport.loopEnabled;
            if (transport.loopRegion.has_value()) {
                nlohmann::json loopRegion;
                loopRegion["startBeats"] = (transport.loopRegion->startSeconds / 60.0) * tempo;
                loopRegion["endBeats"] = (transport.loopRegion->endSeconds / 60.0) * tempo;
                payload["loopRegion"] = loopRegion;
            } else {
                payload["loopRegion"] = nullptr;
            }
        } else {
            payload["isPlaying"] = false;
            payload["positionBeats"] = 0.0;
            payload["loopEnabled"] = false;
            payload["loopRegion"] = nullptr;
        }

        stateEvent.payload = payload;

        session->send(stateEvent);
    }
}

void DomainDispatcher::handleHardwareDomain(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    if (env.kind != IpcKind::Command) {
        std::cout << "[DomainDispatcher] Ignoring non-command hardware envelope" << std::endl;
        return;
    }

    if (!engineHost_ || !hardwareDomain_) {
        std::cerr << "[DomainDispatcher] EngineHost or HardwareDomain is null" << std::endl;
        return;
    }

    // Convert IpcEnvelope to old Envelope format and route through router for command processing
    Envelope old_env;
    old_env.v = env.version;
    old_env.id = env.id;
    old_env.cid = env.correlationId.value_or("");
    old_env.ts = env.timestamp;
    old_env.origin = originToString(env.origin);
    old_env.target = targetToString(env.target);
    old_env.domain = env.domain; // Use canonical domain "hardware"
    old_env.kind = kindToString(env.kind);
    old_env.name = env.name;
    old_env.priority = priorityToString(env.priority);
    old_env.payload = env.payload.dump();

    // Route command through router for processing
    router_->dispatch(old_env);

    // Helper to create response envelope
    auto createResponse = [&](const std::string& name, const nlohmann::json& payload) -> IpcEnvelope {
        IpcEnvelope response;
        response.version = 1;
        response.id = "hardware-" + name + "-" + env.id;
        response.correlationId = env.id;
        response.timestamp = currentTimestamp();
        response.origin = IpcOrigin::Signal;

        // Convert origin to target
        switch (env.origin) {
        case IpcOrigin::Aura:
            response.target = IpcTarget::Aura;
            break;
        case IpcOrigin::Pulse:
            response.target = IpcTarget::Pulse;
            break;
        case IpcOrigin::Signal:
            response.target = IpcTarget::Signal;
            break;
        case IpcOrigin::Composer:
            response.target = IpcTarget::Composer;
            break;
        }

        // Use canonical domain name hardware for responses
        response.domain = "hardware";
        response.kind = IpcKind::Event;
        response.name = name;
        response.priority = env.priority;
        response.payload = payload;
        return response;
    };

    // Get response data from HardwareDomain and send event
    if (env.name == "listOutputDevices" || env.name == "refreshOutputDevices") {
        auto response = hardwareDomain_->handleListOutputDevices();
        if (response.has_value()) {
            IpcEnvelope event = createResponse(response->eventName, response->payload);
            session->send(event);
        }
    } else if (env.name == "selectOutputDevice" || env.name == "setActiveOutputDevice") {
        // Parse device ID from payload
        std::string deviceId;
        try {
            if (env.payload.contains("id") && env.payload["id"].is_string()) {
                deviceId = env.payload["id"].get<std::string>();
            } else if (env.payload.contains("deviceId") && env.payload["deviceId"].is_string()) {
                deviceId = env.payload["deviceId"].get<std::string>();
            } else {
                std::cerr << "[DomainDispatcher] selectOutputDevice: missing or invalid 'id' or 'deviceId' field" << std::endl;
                nlohmann::json errorPayload;
                errorPayload["success"] = false;
                errorPayload["error"] = "Missing or invalid device ID";
                IpcEnvelope errorResponse = createResponse("outputDeviceSelected", errorPayload);
                session->send(errorResponse);
                return;
            }
        } catch (const std::exception& e) {
            std::cerr << "[DomainDispatcher] Failed to parse selectOutputDevice payload: " << e.what() << std::endl;
            nlohmann::json errorPayload;
            errorPayload["success"] = false;
            errorPayload["error"] = "Failed to parse payload: " + std::string(e.what());
            IpcEnvelope errorResponse = createResponse("outputDeviceSelected", errorPayload);
            session->send(errorResponse);
            return;
        }

        auto response = hardwareDomain_->handleSelectOutputDevice(deviceId);
        if (response.has_value()) {
            IpcEnvelope event = createResponse(response->eventName, response->payload);
            session->send(event);
        } else {
            // Fallback error response
            nlohmann::json errorPayload;
            errorPayload["success"] = false;
            errorPayload["error"] = "Failed to process device selection";
            IpcEnvelope errorResponse = createResponse("outputDeviceSelected", errorPayload);
            session->send(errorResponse);
        }
    } else {
        std::cout << "[DomainDispatcher] Unknown hardware command: " << env.name << std::endl;
    }
}

void DomainDispatcher::handleGenericDomain(
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

    // Dispatch to router (this will call the appropriate domain handler)
    router_->dispatch(old_env);
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

