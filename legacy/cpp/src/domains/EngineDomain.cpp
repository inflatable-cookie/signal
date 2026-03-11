#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/StreamScheduler.hpp"
#include "core/ScheduleData.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/EngineSelfTest.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

namespace {
std::string pluginFormatToString(std::optional<PluginFormat> format) {
    if (!format.has_value()) {
        return "unknown";
    }

    switch (format.value()) {
        case PluginFormat::Clap:
            return "clap";
        case PluginFormat::Vst3:
            return "vst3";
        case PluginFormat::Au:
            return "au";
        case PluginFormat::Lv2:
            return "lv2";
        case PluginFormat::Native:
            return "native";
    }

    return "unknown";
}
} // namespace

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

void EngineDomain::handleGraphSnapshot(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    // Handle GraphSnapshot from Pulse
    // Architecture: Pulse sends GraphSnapshot with nodes and connections
    // Signal parses via GraphSnapshot::fromJson and applies to GraphEngine
    auto graphOpt = GraphSnapshot::fromJson(commandEnv.payload);
    if (!graphOpt) {
        LOG_WARN({"EngineDomain", "Graph"}, "Failed to parse graph snapshot; leaving existing graph in place");
        return;
    }

    const auto& snapshot = *graphOpt;

    // Load snapshot into EngineHost
    _engineHost->loadGraphSnapshot(snapshot);

    if (!_engineHost->graphEngine().getUnavailablePluginNodes().empty()) {
        emitPluginUnavailableDiagnosticsEvent(commandEnv, session);
    }

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
        handleGraphSnapshot(env, session);
    } else if (env.name == "selfTest") {
        handleSelfTest(env, session);
        return; // Self-test emits its own event, don't emit state event
    } else {
        LOG_WARN({"EngineDomain"}, std::string("Unknown command: ") + env.name);
    }

    // Emit state events after processing commands (except heartbeat which already emitted)
    if (env.name != "heartbeat") {
        emitEngineStateEvent(env, session);
    }
}

void EngineDomain::emitPluginUnavailableDiagnosticsEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    const auto& unavailable = _engineHost->graphEngine().getUnavailablePluginNodes();
    if (unavailable.empty()) {
        return;
    }

    IpcEnvelope diagnosticsEvent;
    diagnosticsEvent.version = 1;
    diagnosticsEvent.id = "diagnostics-error-" + commandEnv.id;
    diagnosticsEvent.correlationId = commandEnv.id;
    diagnosticsEvent.timestamp = currentTimestamp();
    diagnosticsEvent.origin = IpcOrigin::Signal;
    diagnosticsEvent.target = IpcTarget::Pulse;
    diagnosticsEvent.domain = "diagnostics";
    diagnosticsEvent.kind = IpcKind::Event;
    diagnosticsEvent.name = "error";
    diagnosticsEvent.priority = commandEnv.priority;

    nlohmann::json unavailablePlugins = nlohmann::json::array();
    for (const auto& item : unavailable) {
        unavailablePlugins.push_back({
            {"nodeId", item.nodeId},
            {"pluginId", item.pluginId},
            {"pluginFormat", pluginFormatToString(item.pluginFormat)},
            {"reason", item.reason},
        });
    }

    std::ostringstream message;
    message << "Graph loaded with " << unavailable.size()
            << " unavailable plugin node(s); affected nodes are bypassed.";

    diagnosticsEvent.payload = {
        {"kind", "engine.pluginUnavailableOnRestore"},
        {"message", message.str()},
        {"details",
            {
                {"code", "plugin_unavailable_on_restore"},
                {"unavailablePlugins", unavailablePlugins},
            }},
    };

    session->send(diagnosticsEvent);
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

void EngineDomain::handleSelfTest(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    LOG_INFO({"EngineDomain"}, "Running engine self-test");

    // Run self-test synchronously (should be fast - few ms)
    // This runs on the IPC thread, not the audio thread
    EngineSelfTestResult testResult;
    try {
        testResult = runEngineSelfTest();
    } catch (const std::exception& e) {
        LOG_ERROR({"EngineDomain"}, std::string("Self-test exception: ") + e.what());
        testResult.ok = false;
        // Add a single failed scenario to indicate error
        EngineSelfTestScenarioResult errorScenario;
        errorScenario.id = "error";
        errorScenario.ok = false;
        errorScenario.maxAbsSample = 0.0f;
        testResult.scenarios.push_back(errorScenario);
    } catch (...) {
        LOG_ERROR({"EngineDomain"}, "Self-test failed with unknown exception");
        testResult.ok = false;
        EngineSelfTestScenarioResult errorScenario;
        errorScenario.id = "error";
        errorScenario.ok = false;
        errorScenario.maxAbsSample = 0.0f;
        testResult.scenarios.push_back(errorScenario);
    }

    // Build JSON payload
    nlohmann::json payload;
    payload["ok"] = testResult.ok;
    payload["scenarioCount"] = static_cast<int>(testResult.scenarios.size());

    int failedCount = 0;
    for (const auto& scenario : testResult.scenarios) {
        if (!scenario.ok) {
            failedCount++;
        }
    }
    payload["failedScenarioCount"] = failedCount;

    nlohmann::json scenariosArray = nlohmann::json::array();
    for (const auto& scenario : testResult.scenarios) {
        nlohmann::json scenarioJson;
        scenarioJson["id"] = scenario.id;
        scenarioJson["ok"] = scenario.ok;
        scenarioJson["maxAbsSample"] = scenario.maxAbsSample;
        scenariosArray.push_back(scenarioJson);
    }
    payload["scenarios"] = scenariosArray;

    // Emit self-test result event
    IpcEnvelope resultEvent;
    resultEvent.version = 1;
    resultEvent.id = "engine-self-test-result-" + commandEnv.id;
    resultEvent.correlationId = commandEnv.id;
    resultEvent.timestamp = currentTimestamp();
    resultEvent.origin = IpcOrigin::Signal;

    // Convert origin to target for event
    switch (commandEnv.origin) {
    case IpcOrigin::Aura:
        resultEvent.target = IpcTarget::Aura;
        break;
    case IpcOrigin::Pulse:
        resultEvent.target = IpcTarget::Pulse;
        break;
    case IpcOrigin::Signal:
        resultEvent.target = IpcTarget::Signal;
        break;
    case IpcOrigin::Composer:
        resultEvent.target = IpcTarget::Composer;
        break;
    }

    resultEvent.domain = "engine";
    resultEvent.kind = IpcKind::Event;
    resultEvent.name = "selfTestResult";
    resultEvent.priority = commandEnv.priority;
    resultEvent.payload = payload;

    session->send(resultEvent);

    std::ostringstream msg;
    msg << "Engine self-test complete: " << (testResult.ok ? "PASS" : "FAIL")
        << " (" << (testResult.scenarios.size() - failedCount) << "/"
        << testResult.scenarios.size() << " scenarios passed)";
    LOG_INFO({"EngineDomain"}, msg.str());
}
