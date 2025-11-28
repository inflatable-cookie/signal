#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "ipc/Router.hpp"
#include <memory>

class EngineHost;
class HardwareDomain;

namespace loophole::signal::ipc {

/// Central dispatcher that routes envelopes to domain handlers and can send replies
class DomainDispatcher {
public:
    DomainDispatcher(IpcRouter* router, EngineHost* engineHost);

    void handleEnvelope(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

private:
    void handleEngineDomain(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

    void handleTransportDomain(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

    void handleHardwareDomain(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

    void handleUnknownDomain(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

    void handleGenericDomain(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

    IpcRouter* router_;
    EngineHost* engineHost_;
    std::unique_ptr<HardwareDomain> hardwareDomain_;
};

} // namespace loophole::signal::ipc

