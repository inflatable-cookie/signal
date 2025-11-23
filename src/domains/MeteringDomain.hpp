#pragma once

/// MeteringDomain - IPC domain handler for metering updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by IpcRouter
/// Communication:
///   - Receives commands from Pulse (future: enable/disable metering per channel)
///   - Publishes metering events to Pulse via TcpClientSession

#include "ipc/Router.hpp"
#include "ipc/Envelope.hpp"
#include <memory>

class MeteringService;
class EngineHost;

class MeteringDomain : public DomainHandler {
public:
    MeteringDomain(MeteringService* meteringService, EngineHost* engineHost);
    ~MeteringDomain() override = default;

    void handle(const Envelope& env) override;

private:
    MeteringService* _meteringService;
    EngineHost* _engineHost;
};


