#pragma once

/// NodeDomain - IPC domain handler for node parameter updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (node.setParameter)
///   - Applies parameter changes to GraphNode instances (e.g. FaderNode)
///
/// Phase 9 note:
///   - UI-facing Fader parameters (gain/spatial.balance) are sent via `node.setParameter`
///     using Fader node IDs (e.g. "fader-<trackId>").
///   - Mute is applied via `node.setParameter` (`parameterId="muted"`) and
///     NodeDomain applies these parameters directly to nodes.

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class NodeDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit NodeDomain(EngineHost* engineHost);
    ~NodeDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handleSetParameter(const nlohmann::json& payload);

    EngineHost* _engineHost;
};
