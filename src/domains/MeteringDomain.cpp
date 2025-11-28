#include "domains/MeteringDomain.hpp"
#include "core/MeteringService.hpp"
#include "core/EngineHost.hpp"
#include "ipc/Envelope.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/IpcLegacyBridge.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <sstream>

MeteringDomain::MeteringDomain(IpcRouter* router, MeteringService* meteringService, EngineHost* engineHost)
    : _router(router)
    , _meteringService(meteringService)
    , _engineHost(engineHost)
{
}

void MeteringDomain::handle(const Envelope& env) {
    // For now, metering domain only publishes events (no commands)
    // Future: could support commands like "enableMetering" or "setMeteringRate"
    std::ostringstream msg;
    msg << "Received envelope: " << env.domain << "." << env.name;
    LOG_DEBUG({"MeteringDomain"}, msg.str());
}

void MeteringDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "metering") {
        LOG_DEBUG({"MeteringDomain"}, "Received envelope for different domain");
        return;
    }

    // Convert to legacy envelope and route through router
    auto oldEnv = loophole::signal::ipc::toLegacyEnvelope(env);
    if (_router) {
        _router->dispatch(oldEnv);
    }
}


