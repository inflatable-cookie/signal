#pragma once

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class EngineDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit EngineDomain(IpcRouter* router, EngineHost* engineHost);
    ~EngineDomain() override = default;

    // Legacy DomainHandler interface (for router)
    void handle(const Envelope& env) override;

    // New IpcDomainHandler interface
    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void emitEngineStateEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitHeartbeatEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    IpcRouter* _router;
    EngineHost* _engineHost;
};

