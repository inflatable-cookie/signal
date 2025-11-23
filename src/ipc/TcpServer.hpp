#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/IpcEnvelopeCodec.hpp"
#include "ipc/TcpClientSession.hpp"
#include <asio/ip/tcp.hpp>
#include <algorithm>
#include <chrono>
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
};

} // namespace loophole::signal::ipc

