#pragma once

/// MixerDomain - IPC domain handler for mixer updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (mixer.updateChannel)
///   - Updates MixerService state (gain, pan, mute, solo, effective mute)
///   - Changes are applied in real-time by audio thread
///
/// Phase 9 note:
///   - UI-facing mixer control now flows through Pulse via the `node` (Fader
///     parameters) and `console` (Channel mute/solo) domains.
///   - MixerDomain remains as a Signal-facing bridge that applies consolidated
///     mixer state to MixerService only.
///   - FaderNode parameters are owned by the Node domain (`node.setParameter`)
///     and MixerDomain no longer writes directly to FaderNode instances.
///   - New IPC shaping should prefer the Node and Console domains; MixerDomain
///     is not intended for new UI traffic and will be retired once Signal is
///     fully aligned.

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
