#include "domains/RecordingDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/RecordingCapture.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <nlohmann/json.hpp>
#include <sstream>

RecordingDomain::RecordingDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void RecordingDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "recording") {
        LOG_DEBUG({"RecordingDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"RecordingDomain"}, "Ignoring non-command recording envelope");
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"RecordingDomain"}, "EngineHost is null");
        return;
    }

    if (env.name == "setArmState") {
        handleSetArmState(env.payload);
    } else if (env.name == "startRecording") {
        handleStartRecording(env, session);
    } else if (env.name == "stopRecording") {
        handleStopRecording(env, session);
    } else {
        LOG_WARN({"RecordingDomain"}, std::string("Unknown recording command: ") + env.name);
    }
}

void RecordingDomain::handleSetArmState(const nlohmann::json& payload) {
    try {
        std::string laneId = payload.value("laneId", "");
        bool armed = payload.value("armed", false);
        if (laneId.empty()) {
            LOG_WARN({"RecordingDomain"}, "setArmState missing laneId");
            return;
        }

        _engineHost->recordingSession().setArmState(laneId, armed);
    } catch (const std::exception& e) {
        LOG_ERROR({"RecordingDomain"}, std::string("Failed to parse setArmState payload: ") + e.what());
    }
}

void RecordingDomain::handleStartRecording(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    std::string recordId;
    std::vector<std::string> armedLanes;

    try {
        recordId = env.payload.value("recordId", "");
        if (env.payload.contains("armedLanes") && env.payload["armedLanes"].is_array()) {
            for (const auto& lane : env.payload["armedLanes"]) {
                if (lane.is_string()) {
                    const auto laneId = lane.get<std::string>();
                    if (!laneId.empty()) {
                        armedLanes.push_back(laneId);
                    }
                }
            }
        }
    } catch (const std::exception& e) {
        LOG_ERROR({"RecordingDomain"}, std::string("Failed to parse startRecording payload: ") + e.what());
        return;
    }

    if (!armedLanes.empty()) {
        _engineHost->recordingSession().replaceArmedLanes(armedLanes);
    }

    const auto playheadSamples = _engineHost->getPlayheadSamples();
    _engineHost->recordingSession().startRecording();
    emitRecordingStateEvent(env, session, true, recordId, std::nullopt);

    std::ostringstream msg;
    msg << "Recording started at playhead " << playheadSamples << " samples";
    if (!recordId.empty()) {
        msg << " (recordId=" << recordId << ")";
    }
    LOG_INFO({"RecordingDomain"}, msg.str());
}

void RecordingDomain::handleStopRecording(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    std::string recordId;
    try {
        recordId = env.payload.value("recordId", "");
    } catch (const std::exception& e) {
        LOG_ERROR({"RecordingDomain"}, std::string("Failed to parse stopRecording payload: ") + e.what());
        return;
    }

    const auto playheadSamples = _engineHost->getPlayheadSamples();
    _engineHost->recordingSession().stopRecording();
    emitRecordingStateEvent(env, session, false, recordId, playheadSamples);

    std::ostringstream msg;
    msg << "Recording stopped at playhead " << playheadSamples << " samples";
    if (!recordId.empty()) {
        msg << " (recordId=" << recordId << ")";
    }
    LOG_INFO({"RecordingDomain"}, msg.str());
}

void RecordingDomain::emitRecordingStateEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    bool isRecording,
    const std::string& recordId,
    std::optional<std::uint64_t> endSample
) {
    using namespace loophole::signal::ipc;

    auto armedLanes = _engineHost->recordingSession().getArmedLaneIds();
    IpcEnvelope stateEvent;
    stateEvent.version = 1;
    stateEvent.id = "recording-state-" + commandEnv.id;
    stateEvent.correlationId = commandEnv.id;
    stateEvent.timestamp = currentTimestamp();
    stateEvent.origin = IpcOrigin::Signal;

    switch (commandEnv.origin) {
    case IpcOrigin::Aura: stateEvent.target = IpcTarget::Aura; break;
    case IpcOrigin::Pulse: stateEvent.target = IpcTarget::Pulse; break;
    case IpcOrigin::Signal: stateEvent.target = IpcTarget::Signal; break;
    case IpcOrigin::Composer: stateEvent.target = IpcTarget::Composer; break;
    }

    stateEvent.domain = "recording";
    stateEvent.kind = IpcKind::Event;
    stateEvent.name = "state";
    stateEvent.priority = commandEnv.priority;
    stateEvent.payload = {
        {"recordId", recordId},
        {"isRecording", isRecording},
        {"playheadSamples", _engineHost->getPlayheadSamples()},
        {"armedLaneIds", armedLanes},
    };

    if (endSample.has_value()) {
        stateEvent.payload["endSample"] = endSample.value();
    }

    session->send(stateEvent);
}
