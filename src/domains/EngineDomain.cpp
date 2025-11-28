#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/StreamScheduler.hpp"
#include "core/ScheduleData.hpp"
#include "core/GraphSnapshot.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

EngineDomain::EngineDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void EngineDomain::handleStart() {
    _engineHost->start();
}

void EngineDomain::handleStop() {
    _engineHost->stop();
}

void EngineDomain::handleReset() {
    _engineHost->reset();
}

void EngineDomain::handleShutdown() {
    LOG_INFO({"EngineDomain"}, "Shutdown requested");
    _engineHost->shutdown();
}

void EngineDomain::handleScheduleSession(const nlohmann::json& payload) {
    // Handle stream-based schedule from Pulse
    // Architecture: Pulse sends PlaybackScheduleSnapshot with streams, audioSegments, midiEvents
    // Signal parses via ScheduleData::fromJson and applies to StreamScheduler
    double sampleRate = _engineHost->getSampleRate();
    double defaultTempo = _engineHost->transport().tempo;

    auto scheduleOpt = ScheduleData::fromJson(payload, sampleRate, defaultTempo);
    if (!scheduleOpt) {
        LOG_WARN({"EngineDomain", "Schedule"}, "Failed to parse schedule snapshot; leaving existing schedule in place");
        return;
    }

    const auto& schedule = *scheduleOpt;

    // Warn if schedule is empty (but still apply it)
    if (schedule.audioSegments.empty() && schedule.midiEvents.empty()) {
        LOG_WARN({"EngineDomain", "Schedule"}, "Schedule contains no audio segments or MIDI events");
    }

    // Apply schedule to StreamScheduler
    _engineHost->streamScheduler().setSchedule(schedule);
}

void EngineDomain::handleGraphSnapshot(const nlohmann::json& payload) {
    // Handle GraphSnapshot from Pulse
    // Architecture: Pulse sends GraphSnapshot with nodes and connections
    // Signal parses via GraphSnapshot::fromJson and applies to GraphEngine
    auto graphOpt = GraphSnapshot::fromJson(payload);
    if (!graphOpt) {
        LOG_WARN({"EngineDomain", "Graph"}, "Failed to parse graph snapshot; leaving existing graph in place");
        return;
    }

    const auto& snapshot = *graphOpt;

    // Load snapshot into EngineHost
    _engineHost->loadGraphSnapshot(snapshot);

    // Prepare graph if engine is already running
    if (_engineHost->state() == EngineHost::State::Running) {
        _engineHost->prepareEngine(
            static_cast<int>(_engineHost->getSampleRate()),
            static_cast<size_t>(_engineHost->getBlockSize())
        );
    }
}

void EngineDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "engine") {
        LOG_DEBUG({"EngineDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"EngineDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"EngineDomain"}, "EngineHost is null");
        return;
    }

    // Handle commands directly
    if (env.name == "start") {
        handleStart();
    } else if (env.name == "stop") {
        handleStop();
    } else if (env.name == "reset") {
        handleReset();
    } else if (env.name == "shutdown") {
        handleShutdown();
    } else if (env.name == "heartbeat") {
        // Heartbeat command - just emit response
        emitHeartbeatEvent(env, session);
        return;
    } else if (env.name == "scheduleSession" || env.name == "playbackScheduleSnapshot") {
        handleScheduleSession(env.payload);
    } else if (env.name == "graphSnapshot" || env.name == "applyGraphSnapshot") {
        handleGraphSnapshot(env.payload);
    } else {
        LOG_WARN({"EngineDomain"}, std::string("Unknown command: ") + env.name);
    }

    // Emit state events after processing commands (except heartbeat which already emitted)
    if (env.name != "heartbeat") {
        emitEngineStateEvent(env, session);
    }
}

void EngineDomain::emitEngineStateEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    IpcEnvelope stateEvent;
    stateEvent.version = 1;
    stateEvent.id = "engine-state-" + commandEnv.id;
    stateEvent.correlationId = commandEnv.id;
    stateEvent.timestamp = currentTimestamp();
    stateEvent.origin = IpcOrigin::Signal;

    // Convert origin to target for event
    switch (commandEnv.origin) {
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
    stateEvent.priority = commandEnv.priority;

    // Get current engine state and create payload
    std::string lifecycle = "stopped";
    std::optional<std::string> lastError;
    if (_engineHost) {
        switch (_engineHost->state()) {
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
            lastError = _engineHost->lastError();
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
    if (_engineHost) {
        payload["sampleRate"] = _engineHost->getSampleRate();
        payload["blockSize"] = _engineHost->getBlockSize();
        payload["outputDeviceName"] = _engineHost->getOutputDeviceName();
        payload["numOutputChannels"] = _engineHost->getNumOutputChannels();
    }

    stateEvent.payload = payload;

    session->send(stateEvent);
}

void EngineDomain::emitHeartbeatEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    IpcEnvelope heartbeatEvent;
    heartbeatEvent.version = 1;
    heartbeatEvent.id = "engine-heartbeat-" + commandEnv.id;
    heartbeatEvent.correlationId = commandEnv.id;
    heartbeatEvent.timestamp = currentTimestamp();
    heartbeatEvent.origin = IpcOrigin::Signal;

    switch (commandEnv.origin) {
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
    heartbeatEvent.priority = commandEnv.priority;

    std::string lifecycle = "stopped";
    if (_engineHost) {
        switch (_engineHost->state()) {
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

