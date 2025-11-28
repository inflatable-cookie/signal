#pragma once

/// MixerDomain - IPC domain handler for mixer updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (mixer.updateChannel)
///   - Updates MixerService state
///   - Changes are applied in real-time by audio thread

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class MixerDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit MixerDomain(EngineHost* engineHost);
    ~MixerDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handleUpdateChannel(const nlohmann::json& payload);

    EngineHost* _engineHost;
};

