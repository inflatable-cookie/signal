#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class TransportDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit TransportDomain(EngineHost* engineHost);
    ~TransportDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handlePlay();
    void handleStop(const nlohmann::json& payload);
    void handleSeek(const nlohmann::json& payload);
    void handleSetLoopEnabled(const nlohmann::json& payload);
    void handleSetLoopRegion(const nlohmann::json& payload);
    void handleSetTempo(const nlohmann::json& payload);

    void emitTransportStateEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitTransportPositionUpdate(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    EngineHost* _engineHost;
};

