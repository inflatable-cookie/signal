#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class EngineDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit EngineDomain(EngineHost* engineHost);
    ~EngineDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handleStart();
    void handleStop();
    void handleReset();
    void handleShutdown();
    void handleScheduleSession(const nlohmann::json& payload);
    void handleGraphSnapshot(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleSelfTest(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitEngineStateEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitHeartbeatEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitPluginUnavailableDiagnosticsEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    EngineHost* _engineHost;
};
