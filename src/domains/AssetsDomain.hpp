#pragma once

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <string>
#include <memory>

class EngineHost;

/// Assets domain handler for Signal
///
/// Handles asset registration commands from Pulse
class AssetsDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit AssetsDomain(IpcRouter* router, EngineHost* engineHost);
    ~AssetsDomain() override = default;

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

