#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include "ipc/TcpClientSession.hpp"
#include "ipc/Router.hpp"
#include <memory>
#include <unordered_map>
#include <string>

class EngineHost;
class MeteringService;

namespace loophole::signal::ipc {

/// Central dispatcher that routes envelopes to domain handlers
/// Simple registry-based forwarding - no domain-specific logic
class DomainDispatcher {
public:
    DomainDispatcher(IpcRouter* router, EngineHost* engineHost, MeteringService* meteringService);

    void handleEnvelope(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    );

private:
    IpcRouter* router_;
    EngineHost* engineHost_;
    std::unordered_map<std::string, std::unique_ptr<IpcDomainHandler>> domains_;
};

} // namespace loophole::signal::ipc

