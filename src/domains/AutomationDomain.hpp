#pragma once

/// AutomationDomain - IPC domain handler for automation updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (automation.setCurvesForSession)
///   - Updates AutomationService state
///   - Changes are applied in real-time by audio thread

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class AutomationDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit AutomationDomain(IpcRouter* router, EngineHost* engineHost);
    ~AutomationDomain() override = default;

    // Legacy DomainHandler interface (for router)
    void handle(const Envelope& env) override;

    // New IpcDomainHandler interface
    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    IpcRouter* _router;
    EngineHost* _engineHost;
};

