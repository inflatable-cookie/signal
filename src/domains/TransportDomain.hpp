#pragma once

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class TransportDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit TransportDomain(IpcRouter* router, EngineHost* engineHost);
    ~TransportDomain() override = default;

    // Legacy DomainHandler interface (for router)
    void handle(const Envelope& env) override;

    // New IpcDomainHandler interface
    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void emitTransportStateEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    IpcRouter* _router;
    EngineHost* _engineHost;
};

