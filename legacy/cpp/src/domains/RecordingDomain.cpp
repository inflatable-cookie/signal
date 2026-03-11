#include "domains/RecordingDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/RecordingCapture.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <unordered_map>
#include <nlohmann/json.hpp>
#include <sstream>

namespace {

struct AggregatedAudioChunk {
    std::string laneId;
    std::string assetId;
    int numChannels = 0;
    int sampleRate = 0;
    std::uint64_t startSample = 0;
    std::uint64_t endSample = 0;
    std::vector<float> interleaved;
};

struct AggregatedMidiChunk {
    std::string laneId;
    std::uint64_t startSample = 0;
    std::uint64_t endSample = 0;
    std::vector<RecordedMidiEvent> events;
};

std::string makeRecordingAssetId(const std::string& recordId, const std::string& laneId) {
    return "recording-" + recordId + "-" + laneId;
}

std::vector<AggregatedAudioChunk> aggregateAudioChunks(
    const std::string& recordId,
    const std::vector<RecordedAudioChunk>& chunks
) {
    std::vector<RecordedAudioChunk> sorted = chunks;
    std::sort(
        sorted.begin(),
        sorted.end(),
        [](const RecordedAudioChunk& left, const RecordedAudioChunk& right) {
            if (left.laneId != right.laneId) {
                return left.laneId < right.laneId;
            }
            return left.startSample < right.startSample;
        }
    );

    std::vector<AggregatedAudioChunk> aggregated;
    for (const auto& chunk : sorted) {
        if (chunk.laneId.empty() || chunk.numChannels <= 0 || chunk.sampleRate <= 0) {
            continue;
        }

        const auto frameCount = static_cast<std::uint64_t>(
            chunk.interleaved.size() / static_cast<std::size_t>(chunk.numChannels)
        );
        if (frameCount == 0) {
            continue;
        }

        if (
            !aggregated.empty() &&
            aggregated.back().laneId == chunk.laneId &&
            aggregated.back().numChannels == chunk.numChannels &&
            aggregated.back().sampleRate == chunk.sampleRate
        ) {
            aggregated.back().interleaved.insert(
                aggregated.back().interleaved.end(),
                chunk.interleaved.begin(),
                chunk.interleaved.end()
            );
            aggregated.back().endSample = std::max(
                aggregated.back().endSample,
                chunk.startSample + frameCount
            );
            continue;
        }

        AggregatedAudioChunk next;
        next.laneId = chunk.laneId;
        next.assetId = makeRecordingAssetId(recordId, chunk.laneId);
        next.numChannels = chunk.numChannels;
        next.sampleRate = chunk.sampleRate;
        next.startSample = chunk.startSample;
        next.endSample = chunk.startSample + frameCount;
        next.interleaved = chunk.interleaved;
        aggregated.push_back(std::move(next));
    }

    return aggregated;
}

std::vector<AggregatedMidiChunk> aggregateMidiChunks(
    const std::vector<RecordedMidiChunk>& chunks,
    std::uint64_t fallbackEndSample
) {
    std::vector<RecordedMidiChunk> sorted = chunks;
    std::sort(
        sorted.begin(),
        sorted.end(),
        [](const RecordedMidiChunk& left, const RecordedMidiChunk& right) {
            if (left.laneId != right.laneId) {
                return left.laneId < right.laneId;
            }
            return left.startSample < right.startSample;
        }
    );

    std::vector<AggregatedMidiChunk> aggregated;
    for (const auto& chunk : sorted) {
        if (chunk.laneId.empty()) {
            continue;
        }

        std::uint64_t chunkEndSample = fallbackEndSample;
        for (const auto& event : chunk.events) {
            chunkEndSample = std::max(chunkEndSample, event.timeSamples);
        }

        if (!aggregated.empty() && aggregated.back().laneId == chunk.laneId) {
            aggregated.back().events.insert(
                aggregated.back().events.end(),
                chunk.events.begin(),
                chunk.events.end()
            );
            aggregated.back().endSample = std::max(aggregated.back().endSample, chunkEndSample);
            continue;
        }

        AggregatedMidiChunk next;
        next.laneId = chunk.laneId;
        next.startSample = chunk.startSample;
        next.endSample = chunkEndSample;
        next.events = chunk.events;
        aggregated.push_back(std::move(next));
    }

    return aggregated;
}

} // namespace

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
    _engineHost->recordingSession().startRecording(playheadSamples);
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
    emitRecordingFinishedEvent(env, session, recordId, playheadSamples);

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

void RecordingDomain::emitRecordingFinishedEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    const std::string& recordId,
    std::uint64_t endSample
) {
    using namespace loophole::signal::ipc;

    std::vector<RecordedAudioChunk> audioChunks;
    std::vector<RecordedMidiChunk> midiChunks;
    _engineHost->recordingSession().consumeAudioChunks(audioChunks);
    _engineHost->recordingSession().consumeMidiChunks(midiChunks);

    auto aggregatedAudio = aggregateAudioChunks(recordId, audioChunks);
    auto aggregatedMidi = aggregateMidiChunks(midiChunks, endSample);

    IpcEnvelope finishedEvent;
    finishedEvent.version = 1;
    finishedEvent.id = "recording-finished-" + commandEnv.id;
    finishedEvent.correlationId = commandEnv.id;
    finishedEvent.timestamp = currentTimestamp();
    finishedEvent.origin = IpcOrigin::Signal;

    switch (commandEnv.origin) {
    case IpcOrigin::Aura: finishedEvent.target = IpcTarget::Aura; break;
    case IpcOrigin::Pulse: finishedEvent.target = IpcTarget::Pulse; break;
    case IpcOrigin::Signal: finishedEvent.target = IpcTarget::Signal; break;
    case IpcOrigin::Composer: finishedEvent.target = IpcTarget::Composer; break;
    }

    nlohmann::json audioPayload = nlohmann::json::array();
    for (const auto& chunk : aggregatedAudio) {
        audioPayload.push_back({
            {"laneId", chunk.laneId},
            {"assetId", chunk.assetId},
            {"numChannels", chunk.numChannels},
            {"sampleRate", chunk.sampleRate},
            {"startSample", chunk.startSample},
            {"endSample", chunk.endSample},
            {"audioData", chunk.interleaved},
        });
    }

    nlohmann::json midiPayload = nlohmann::json::array();
    for (const auto& chunk : aggregatedMidi) {
        nlohmann::json events = nlohmann::json::array();
        for (const auto& event : chunk.events) {
            events.push_back({
                {"timeSamples", event.timeSamples},
                {"status", event.status},
                {"data1", event.data1},
                {"data2", event.data2},
                {"channel", event.channel},
            });
        }
        midiPayload.push_back({
            {"laneId", chunk.laneId},
            {"startSample", chunk.startSample},
            {"endSample", chunk.endSample},
            {"events", events},
        });
    }

    finishedEvent.domain = "recording";
    finishedEvent.kind = IpcKind::Event;
    finishedEvent.name = "recordingFinished";
    finishedEvent.priority = commandEnv.priority;
    finishedEvent.payload = {
        {"recordId", recordId},
        {"audioChunks", audioPayload},
        {"midiChunks", midiPayload},
        {"startSample", _engineHost->recordingSession().getRecordingStartSample()},
        {"endSample", endSample},
        {"sampleRate", static_cast<std::uint32_t>(_engineHost->getSampleRate())},
    };

    session->send(finishedEvent);
}
