#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/IpcEnvelopeCodec.hpp"
#include "ipc/TcpClientSession.hpp"
#include <asio/ip/tcp.hpp>
#include <algorithm>
#include <chrono>
#include <functional>
#include <memory>
#include <mutex>
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

namespace loophole::signal::ipc {

/// TCP server for IPC envelope communication
class TcpServer {
public:
    using EnvelopeHandler = TcpClientSession::EnvelopeHandler;

    TcpServer(
        asio::io_context& io,
        const std::string& host,
        uint16_t port,
        EnvelopeHandler handler
    );

    void start();
    void stop();

    // Set callback to be called when a client connects
    // The callback receives the newly connected session
    using ClientConnectedCallback = std::function<void(const std::shared_ptr<TcpClientSession>&)>;
    void setClientConnectedCallback(ClientConnectedCallback callback);

    // Broadcast metering update event to all connected clients
    template<typename MeteringServiceType>
    void broadcastMetering(MeteringServiceType* meteringService) {
        std::lock_guard<std::mutex> lock(clientsMutex_);

        // Remove expired weak pointers
        clients_.erase(
            std::remove_if(
                clients_.begin(),
                clients_.end(),
                [](const std::weak_ptr<TcpClientSession>& wp) {
                    return wp.expired();
                }
            ),
            clients_.end()
        );

        if (!meteringService) {
            return;
        }

        // Get metering snapshot
        auto snapshot = meteringService->snapshotAll();
        if (snapshot.empty()) {
            return; // No channels to meter
        }

        // Create metering update envelope
        IpcEnvelope meteringEvent;
        meteringEvent.version = 1;
        meteringEvent.id = "metering-update-" + std::to_string(std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch()).count());
        meteringEvent.correlationId = std::nullopt;
        meteringEvent.timestamp = currentTimestamp();
        meteringEvent.origin = IpcOrigin::Signal;
        meteringEvent.target = IpcTarget::Pulse;
        meteringEvent.domain = "metering";
        meteringEvent.kind = IpcKind::Event;
        meteringEvent.name = "update";
        meteringEvent.priority = IpcPriority::Normal;

        // Build payload with channel metering data
        nlohmann::json payload;
        nlohmann::json channels = nlohmann::json::array();
        for (const auto& metering : snapshot) {
            nlohmann::json channel;
            channel["channelId"] = metering.channelId;
            channel["peak"] = metering.peak;
            channel["rms"] = metering.rms;
            channel["timestamp"] = metering.timestamp;
            channels.push_back(channel);
        }
        payload["channels"] = channels;
        meteringEvent.payload = payload;

        // Send to all active clients
        for (auto& weak_session : clients_) {
            if (auto session = weak_session.lock()) {
                session->send(meteringEvent);
            }
        }
    }

    // Send current engine state to a specific client session
    template<typename EngineHostType>
    void sendEngineState(EngineHostType* engineHost, const std::shared_ptr<TcpClientSession>& session) {
        if (!engineHost || !session) {
            return;
        }

        // Create engine.state event
        IpcEnvelope stateEvent;
        stateEvent.version = 1;
        stateEvent.id = "engine-state-initial-" + std::to_string(std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch()).count());
        stateEvent.correlationId = std::nullopt;
        stateEvent.timestamp = currentTimestamp();
        stateEvent.origin = IpcOrigin::Signal;
        stateEvent.target = IpcTarget::Pulse;
        stateEvent.domain = "engine";
        stateEvent.kind = IpcKind::Event;
        stateEvent.name = "state";
        stateEvent.priority = IpcPriority::Normal;

        // Get current engine state
        std::string lifecycle = "stopped";
        std::optional<std::string> lastError;
        switch (engineHost->state()) {
        case EngineHostType::State::Stopped:
            lifecycle = "stopped";
            break;
        case EngineHostType::State::Starting:
            lifecycle = "starting";
            break;
        case EngineHostType::State::Running:
            lifecycle = "running";
            break;
        case EngineHostType::State::Error:
            lifecycle = "error";
            lastError = engineHost->lastError();
            break;
        }

        nlohmann::json payload;
        payload["lifecycle"] = lifecycle;
        if (lastError.has_value()) {
            payload["lastError"] = lastError.value();
        } else {
            payload["lastError"] = nullptr;
        }

        // Include runtime configuration (device is initialized even when stopped)
        payload["sampleRate"] = engineHost->getSampleRate();
        payload["blockSize"] = engineHost->getBlockSize();
        payload["outputDeviceName"] = engineHost->getOutputDeviceName();
        payload["numOutputChannels"] = engineHost->getNumOutputChannels();

        stateEvent.payload = payload;

        session->send(stateEvent);
    }

    // Broadcast engine state to all connected clients
    template<typename EngineHostType>
    void broadcastEngineState(EngineHostType* engineHost) {
        std::lock_guard<std::mutex> lock(clientsMutex_);

        // Remove expired weak pointers
        clients_.erase(
            std::remove_if(
                clients_.begin(),
                clients_.end(),
                [](const std::weak_ptr<TcpClientSession>& wp) {
                    return wp.expired();
                }
            ),
            clients_.end()
        );

        if (!engineHost) {
            return;
        }

        // Create engine.state event
        IpcEnvelope stateEvent;
        stateEvent.version = 1;
        stateEvent.id = "engine-state-broadcast-" + std::to_string(std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch()).count());
        stateEvent.correlationId = std::nullopt;
        stateEvent.timestamp = currentTimestamp();
        stateEvent.origin = IpcOrigin::Signal;
        stateEvent.target = IpcTarget::Pulse;
        stateEvent.domain = "engine";
        stateEvent.kind = IpcKind::Event;
        stateEvent.name = "state";
        stateEvent.priority = IpcPriority::Normal;

        // Get current engine state
        std::string lifecycle = "stopped";
        std::optional<std::string> lastError;
        switch (engineHost->state()) {
        case EngineHostType::State::Stopped:
            lifecycle = "stopped";
            break;
        case EngineHostType::State::Starting:
            lifecycle = "starting";
            break;
        case EngineHostType::State::Running:
            lifecycle = "running";
            break;
        case EngineHostType::State::Error:
            lifecycle = "error";
            lastError = engineHost->lastError();
            break;
        }

        nlohmann::json payload;
        payload["lifecycle"] = lifecycle;
        if (lastError.has_value()) {
            payload["lastError"] = lastError.value();
        } else {
            payload["lastError"] = nullptr;
        }

        // Include runtime configuration
        payload["sampleRate"] = engineHost->getSampleRate();
        payload["blockSize"] = engineHost->getBlockSize();
        payload["outputDeviceName"] = engineHost->getOutputDeviceName();
        payload["numOutputChannels"] = engineHost->getNumOutputChannels();

        stateEvent.payload = payload;

        // Send to all active clients
        for (auto& weak_session : clients_) {
            if (auto session = weak_session.lock()) {
                session->send(stateEvent);
            }
        }
    }

    // Broadcast transport position update to all connected clients (throttled)
    // Updates are sent at ~1Hz while playing, plus immediately on state changes
    template<typename EngineHostType>
    void broadcastTransportPosition(EngineHostType* engineHost, bool forceImmediate = false) {
        std::lock_guard<std::mutex> lock(clientsMutex_);

        // Remove expired weak pointers
        clients_.erase(
            std::remove_if(
                clients_.begin(),
                clients_.end(),
                [](const std::weak_ptr<TcpClientSession>& wp) {
                    return wp.expired();
                }
            ),
            clients_.end()
        );

        if (!engineHost) {
            return;
        }

        // Get current playhead and transport state
        uint64_t playheadSamples = engineHost->getPlayheadSamples();
        const auto& transport = engineHost->transport();
        double sampleRate = engineHost->getSampleRate();

        // Throttling: Only send periodic updates when playing
        // Always send immediate updates on state changes (play/stop/seek)
        if (!forceImmediate && !transport.isPlaying) {
            return; // Don't send periodic updates when stopped
        }

        // Check throttling (only for periodic updates, not forced immediate)
        if (!forceImmediate && transport.isPlaying) {
            auto now = std::chrono::steady_clock::now();
            auto timeSinceLastReport = std::chrono::duration_cast<std::chrono::milliseconds>(
                now - lastPositionReportTime_).count();

            // Throttle to ~1Hz (1000ms interval) for periodic updates
            constexpr int64_t POSITION_UPDATE_INTERVAL_MS = 1000;
            if (timeSinceLastReport < POSITION_UPDATE_INTERVAL_MS) {
                return; // Too soon, skip this update (throttling working correctly)
            }
        }

        // If we get here, either:
        // - forceImmediate is true (state change or seek), OR
        // - Enough time has passed since last report (throttling passed)

        // Update throttling state
        lastPositionReportTime_ = std::chrono::steady_clock::now();
        lastReportedPositionSamples_ = playheadSamples;

        // Create transport.positionUpdate event
        IpcEnvelope positionEvent;
        positionEvent.version = 1;
        positionEvent.id = "transport-position-update-" + std::to_string(std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch()).count());
        positionEvent.correlationId = std::nullopt;
        positionEvent.timestamp = currentTimestamp();
        positionEvent.origin = IpcOrigin::Signal;
        positionEvent.target = IpcTarget::Pulse;
        positionEvent.domain = "transport";
        positionEvent.kind = IpcKind::Event;
        positionEvent.name = "positionUpdate";
        positionEvent.priority = IpcPriority::Normal;

        nlohmann::json payload;
        payload["state"] = transport.isPlaying ? "playing" : "stopped";
        payload["positionSamples"] = playheadSamples;
        payload["positionSeconds"] = static_cast<double>(playheadSamples) / sampleRate;
        payload["sampleRate"] = sampleRate;

        positionEvent.payload = payload;

        // Send to all active clients
        for (auto& weak_session : clients_) {
            if (auto session = weak_session.lock()) {
                session->send(positionEvent);
            }
        }
    }

    // Broadcast diagnostics event to all connected clients
    template<typename EngineHostType>
    void broadcastDiagnostics(EngineHostType* engineHost) {
        std::lock_guard<std::mutex> lock(clientsMutex_);

        // Remove expired weak pointers
        clients_.erase(
            std::remove_if(
                clients_.begin(),
                clients_.end(),
                [](const std::weak_ptr<TcpClientSession>& wp) {
                    return wp.expired();
                }
            ),
            clients_.end()
        );

        // Create diagnostics envelope
        IpcEnvelope diagEvent;
        diagEvent.version = 1;
        diagEvent.id = "engine-diagnostics-" + std::to_string(std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch()).count());
        diagEvent.correlationId = std::nullopt; // Diagnostics are unsolicited
        diagEvent.timestamp = currentTimestamp();
        diagEvent.origin = IpcOrigin::Signal;
        diagEvent.target = IpcTarget::Pulse; // Diagnostics go to Pulse
        diagEvent.domain = "engine";
        diagEvent.kind = IpcKind::Event;
        diagEvent.name = "diagnostics";
        diagEvent.priority = IpcPriority::Normal;

        nlohmann::json payload;
        if (engineHost) {
            payload["cpuLoad"] = engineHost->getCpuLoad();
            payload["xruns"] = engineHost->getXruns();

            std::string lifecycle = "stopped";
            switch (engineHost->state()) {
            case EngineHostType::State::Stopped:
                lifecycle = "stopped";
                break;
            case EngineHostType::State::Starting:
                lifecycle = "starting";
                break;
            case EngineHostType::State::Running:
                lifecycle = "running";
                break;
            case EngineHostType::State::Error:
                lifecycle = "error";
                break;
            }
            payload["engineState"] = lifecycle;
            payload["sampleRate"] = engineHost->getSampleRate();
            payload["blockSize"] = engineHost->getBlockSize();

            const auto& transport = engineHost->transport();
            payload["transportState"] = transport.isPlaying ? "playing" : "stopped";
        } else {
            payload["cpuLoad"] = 0.0;
            payload["xruns"] = 0;
            payload["engineState"] = "stopped";
            payload["sampleRate"] = 44100.0;
            payload["blockSize"] = 512;
            payload["transportState"] = "stopped";
        }

        diagEvent.payload = payload;

        // Send to all active clients
        for (auto& weak_session : clients_) {
            if (auto session = weak_session.lock()) {
                session->send(diagEvent);
            }
        }
    }

private:
    void doAccept();

    asio::io_context& io_;
    asio::ip::tcp::acceptor acceptor_;
    EnvelopeHandler handler_;
    std::mutex clientsMutex_;
    std::vector<std::weak_ptr<TcpClientSession>> clients_;
    ClientConnectedCallback clientConnectedCallback_;

    // Throttling state for transport position updates
    std::chrono::steady_clock::time_point lastPositionReportTime_;
    uint64_t lastReportedPositionSamples_;
};

} // namespace loophole::signal::ipc

