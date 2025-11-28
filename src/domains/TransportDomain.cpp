#include "domains/TransportDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/TransportState.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

TransportDomain::TransportDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void TransportDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "transport") {
        LOG_DEBUG({"TransportDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"TransportDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"TransportDomain"}, "EngineHost is null");
        return;
    }

    // Handle commands directly
    if (env.name == "play") {
        handlePlay();
    } else if (env.name == "stop") {
        handleStop(env.payload);
    } else if (env.name == "seek") {
        handleSeek(env.payload);
    } else if (env.name == "setLoopEnabled") {
        handleSetLoopEnabled(env.payload);
    } else if (env.name == "setLoopRegion") {
        handleSetLoopRegion(env.payload);
    } else if (env.name == "setTempo") {
        handleSetTempo(env.payload);
    } else {
        LOG_WARN({"TransportDomain"}, std::string("Unknown command: ") + env.name);
    }

    // Emit state events after processing commands
    emitTransportStateEvent(env, session);
}

void TransportDomain::handlePlay() {
    auto& transport = _engineHost->transport();
    transport.isPlaying = true;
    // Get current playhead from engine (source of truth) - this preserves position from stop/seek
    double sampleRate = _engineHost->getSampleRate();
    uint64_t playheadSamples = _engineHost->getPlayheadSamples();
    double positionSeconds = static_cast<double>(playheadSamples) / sampleRate;
    // Sync transport.positionSeconds with actual playhead
    transport.positionSeconds = positionSeconds;
    _engineHost->commitTransportUpdate();  // Commit snapshot swap
    std::ostringstream msg;
    msg << "Play command received, playhead: " << playheadSamples << " samples (positionSeconds: " << positionSeconds << "s)";
    LOG_INFO({"TransportDomain"}, msg.str());
}

void TransportDomain::handleStop(const nlohmann::json& payload) {
    // If payload contains a position, seek to it FIRST (from Aura's simulation)
    // This ensures we stop at the exact position Aura requested, not where the playhead happens to be
    auto& transport = _engineHost->transport();
    double sampleRate = _engineHost->getSampleRate();
    double tempo = transport.tempo; // Get tempo from transport state

    try {
        double positionSeconds = 0.0;
        bool hasPosition = false;

        // Debug: log payload contents
        std::ostringstream debugMsg;
        debugMsg << "Stop payload: " << payload.dump();
        LOG_DEBUG({"TransportDomain"}, debugMsg.str());

        if (payload.contains("positionSamples")) {
            // Direct sample position (preferred)
            uint64_t samples = payload["positionSamples"].get<uint64_t>();
            positionSeconds = static_cast<double>(samples) / sampleRate;
            hasPosition = true;
            LOG_DEBUG({"TransportDomain"}, "Using positionSamples from payload");
        } else if (payload.contains("seconds")) {
            positionSeconds = payload["seconds"].get<double>();
            hasPosition = true;
            LOG_DEBUG({"TransportDomain"}, "Using seconds from payload");
        } else if (payload.contains("positionBeats")) {
            // Convert beats to seconds using real tempo
            double beats = payload["positionBeats"].get<double>();
            positionSeconds = (beats / tempo) * 60.0;
            hasPosition = true;
            LOG_DEBUG({"TransportDomain"}, "Using positionBeats from payload");
        }

        if (hasPosition) {
            // Seek to the requested position FIRST, then stop
            // This ensures we stop at the exact position Aura requested
            transport.positionSeconds = positionSeconds;
            uint64_t playheadSamples = static_cast<uint64_t>(positionSeconds * sampleRate);
            _engineHost->setPlayheadSamples(playheadSamples);
        } else {
            // No position in payload - use current playhead
            uint64_t playheadSamples = _engineHost->getPlayheadSamples();
            transport.positionSeconds = static_cast<double>(playheadSamples) / sampleRate;
        }
    } catch (const std::exception& e) {
        // If payload parsing fails, fall back to current playhead
        uint64_t playheadSamples = _engineHost->getPlayheadSamples();
        transport.positionSeconds = static_cast<double>(playheadSamples) / sampleRate;
        std::ostringstream msg;
        msg << "Stop command received, payload parse failed: " << e.what() << ", using current: " << transport.positionSeconds << "s";
        LOG_WARN({"TransportDomain"}, msg.str());
    }

    // Now stop playback (position already set above)
    transport.isPlaying = false;
    _engineHost->commitTransportUpdate();  // Commit stop + position in one update

    // Log final state
    uint64_t finalPlayheadSamples = _engineHost->getPlayheadSamples();
    double finalPositionSeconds = static_cast<double>(finalPlayheadSamples) / sampleRate;
    std::ostringstream msg;
    msg << "Stop command received, stopped at position: " << finalPositionSeconds << "s (" << finalPlayheadSamples << " samples)";
    LOG_INFO({"TransportDomain"}, msg.str());
}

void TransportDomain::handleSeek(const nlohmann::json& payload) {
    auto& transport = _engineHost->transport();
    double sampleRate = _engineHost->getSampleRate();
    double tempo = transport.tempo; // Get tempo from transport state

    try {
        double positionSeconds = 0.0;
        if (payload.contains("positionSamples")) {
            // Direct sample position (preferred)
            uint64_t samples = payload["positionSamples"].get<uint64_t>();
            positionSeconds = static_cast<double>(samples) / sampleRate; // Use engine sample rate
        } else if (payload.contains("seconds")) {
            positionSeconds = payload["seconds"].get<double>();
        } else if (payload.contains("positionBeats")) {
            // Convert beats to seconds using real tempo
            double beats = payload["positionBeats"].get<double>();
            positionSeconds = (beats / tempo) * 60.0;
        }
        transport.positionSeconds = positionSeconds;

        // Update playhead in samples
        uint64_t playheadSamples = static_cast<uint64_t>(positionSeconds * sampleRate);
        _engineHost->setPlayheadSamples(playheadSamples);
        _engineHost->commitTransportUpdate();  // Commit snapshot swap

        std::ostringstream msg;
        msg << "Seek command received, position: "
            << transport.positionSeconds << "s (" << playheadSamples << " samples)";
        LOG_INFO({"TransportDomain"}, msg.str());
    } catch (const std::exception& e) {
        LOG_ERROR({"TransportDomain"}, std::string("Failed to parse seek payload: ") + e.what());
    }
}

void TransportDomain::handleSetLoopEnabled(const nlohmann::json& payload) {
    auto& transport = _engineHost->transport();

    try {
        if (payload.contains("enabled")) {
            transport.loopEnabled = payload["enabled"].get<bool>();
            _engineHost->commitTransportUpdate();  // Commit snapshot swap
            std::ostringstream msg;
            msg << "Loop enabled: " << transport.loopEnabled;
            LOG_INFO({"TransportDomain"}, msg.str());
        }
    } catch (const std::exception& e) {
        LOG_ERROR({"TransportDomain"}, std::string("Failed to parse setLoopEnabled payload: ") + e.what());
    }
}

void TransportDomain::handleSetLoopRegion(const nlohmann::json& payload) {
    auto& transport = _engineHost->transport();
    double sampleRate = _engineHost->getSampleRate();
    double tempo = transport.tempo; // Get tempo from transport state

    try {
        LoopRegion region;
        bool hasRegion = false;

        // Prefer samples (per spec), fall back to seconds or beats
        if (payload.contains("startSamples") && payload.contains("endSamples")) {
            uint64_t startSamples = payload["startSamples"].get<uint64_t>();
            uint64_t endSamples = payload["endSamples"].get<uint64_t>();
            region.startSeconds = static_cast<double>(startSamples) / sampleRate;
            region.endSeconds = static_cast<double>(endSamples) / sampleRate;
            hasRegion = true;
        } else if (payload.contains("startSeconds") && payload.contains("endSeconds")) {
            region.startSeconds = payload["startSeconds"].get<double>();
            region.endSeconds = payload["endSeconds"].get<double>();
            hasRegion = true;
        } else if (payload.contains("startBeats") && payload.contains("endBeats")) {
            // Convert beats to seconds using real tempo
            double startBeats = payload["startBeats"].get<double>();
            double endBeats = payload["endBeats"].get<double>();
            region.startSeconds = (startBeats / tempo) * 60.0;
            region.endSeconds = (endBeats / tempo) * 60.0;
            hasRegion = true;
        }

        if (hasRegion) {
            transport.loopRegion = region;

            // Also store sample-based loop region for efficient audio thread access
            LoopRegionSamples loopSamples;
            loopSamples.startSamples = static_cast<uint64_t>(region.startSeconds * sampleRate);
            loopSamples.endSamples = static_cast<uint64_t>(region.endSeconds * sampleRate);
            transport.loopRegionSamples = loopSamples;

            _engineHost->commitTransportUpdate();  // Commit snapshot swap
            std::ostringstream msg;
            msg << "Loop region set: " << region.startSeconds
                << " - " << region.endSeconds << "s ("
                << loopSamples.startSamples << " - " << loopSamples.endSamples << " samples)";
            LOG_INFO({"TransportDomain"}, msg.str());
        } else {
            // Clear loop region if enabled is false
            if (payload.contains("enabled") && !payload["enabled"].get<bool>()) {
                transport.loopRegion = std::nullopt;
                transport.loopRegionSamples = std::nullopt;
                _engineHost->commitTransportUpdate();  // Commit snapshot swap
                LOG_INFO({"TransportDomain"}, "Loop region cleared");
            }
        }
    } catch (const std::exception& e) {
        LOG_ERROR({"TransportDomain"}, std::string("Failed to parse setLoopRegion payload: ") + e.what());
    }
}

void TransportDomain::handleSetTempo(const nlohmann::json& payload) {
    auto& transport = _engineHost->transport();

    try {
        if (payload.contains("tempo")) {
            transport.tempo = payload["tempo"].get<double>();
            _engineHost->commitTransportUpdate();  // Commit snapshot swap
            std::ostringstream msg;
            msg << "Tempo set to: " << transport.tempo << " BPM";
            LOG_INFO({"TransportDomain"}, msg.str());
        }
    } catch (const std::exception& e) {
        LOG_ERROR({"TransportDomain"}, std::string("Failed to parse setTempo payload: ") + e.what());
    }
}

void TransportDomain::emitTransportStateEvent(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    IpcEnvelope stateEvent;
    stateEvent.version = 1;
    stateEvent.id = "transport-state-" + commandEnv.id;
    stateEvent.correlationId = commandEnv.id;
    stateEvent.timestamp = currentTimestamp();
    stateEvent.origin = IpcOrigin::Signal;

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

    stateEvent.domain = "transport";
    stateEvent.kind = IpcKind::Event;
    stateEvent.name = "state";
    stateEvent.priority = commandEnv.priority;

    // Get current transport state and create payload
    nlohmann::json payload;
    if (_engineHost) {
        const auto& transport = _engineHost->transport();
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

