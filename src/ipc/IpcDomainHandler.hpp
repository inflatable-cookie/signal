#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include <memory>

namespace loophole::signal::ipc {

/// Interface for domain handlers that process IpcEnvelope directly
class IpcDomainHandler {
public:
    virtual ~IpcDomainHandler() = default;

    virtual void handle(
        const IpcEnvelope& env,
        const std::shared_ptr<TcpClientSession>& session
    ) = 0;
};

} // namespace loophole::signal::ipc

