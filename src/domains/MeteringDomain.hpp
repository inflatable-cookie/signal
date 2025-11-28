#pragma once

/// MeteringDomain - IPC domain handler for metering updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (future: enable/disable metering per channel)
///   - Publishes metering events to Pulse via TcpClientSession

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class MeteringService;
class EngineHost;

class MeteringDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    MeteringDomain(MeteringService* meteringService, EngineHost* engineHost);
    ~MeteringDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    MeteringService* _meteringService;
    EngineHost* _engineHost;
};


