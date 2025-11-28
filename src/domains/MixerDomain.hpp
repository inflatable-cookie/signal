#pragma once

/// MixerDomain - IPC domain handler for mixer updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (mixer.updateChannel)
///   - Updates MixerService state
///   - Changes are applied in real-time by audio thread

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class MixerDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit MixerDomain(IpcRouter* router, EngineHost* engineHost);
    ~MixerDomain() override = default;

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

