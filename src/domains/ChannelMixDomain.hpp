#pragma once

/// ChannelMixDomain - IPC domain handler for channel‑mix updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (channelMix.updateChannel)
///   - Updates ChannelMixService state (gain, pan, mute, solo, effective mute)
///   - Changes are applied in real-time by audio thread
///
/// Phase 9 note:
///   - UI-facing channel control now flows through Pulse via the `node` (Fader
///     parameters) and `console` (Channel mute/solo) domains.
///   - ChannelMixDomain remains as a Signal-facing bridge that applies consolidated
///     channel‑mix state to ChannelMixService only.
///   - FaderNode parameters are owned by the Node domain (`node.setParameter`)
///     and ChannelMixDomain no longer writes directly to FaderNode instances.
///   - New IPC shaping should prefer the Node and Console domains; ChannelMixDomain
///     is not intended for new UI traffic and will be retired once Signal is
///     fully aligned.

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class ChannelMixDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit ChannelMixDomain(EngineHost* engineHost);
    ~ChannelMixDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handleUpdateChannel(const nlohmann::json& payload);

    EngineHost* _engineHost;
};
