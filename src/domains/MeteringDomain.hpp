#pragma once

/// MeteringDomain - IPC domain handler for metering updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (future: enable/disable metering per channel)
///   - Publishes metering events to Pulse via TcpClientSession

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class MeteringService;
class EngineHost;

class MeteringDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    MeteringDomain(IpcRouter* router, MeteringService* meteringService, EngineHost* engineHost);
    ~MeteringDomain() override = default;

    // Legacy DomainHandler interface (for router)
    void handle(const Envelope& env) override;

    // New IpcDomainHandler interface
    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    IpcRouter* _router;
    MeteringService* _meteringService;
    EngineHost* _engineHost;
};


