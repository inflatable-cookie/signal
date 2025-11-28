#include "domains/MeteringDomain.hpp"
#include "core/MeteringService.hpp"
#include "core/EngineHost.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <sstream>

MeteringDomain::MeteringDomain(MeteringService* meteringService, EngineHost* engineHost)
    : _meteringService(meteringService)
    , _engineHost(engineHost)
{
}

void MeteringDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "metering") {
        LOG_DEBUG({"MeteringDomain"}, "Received envelope for different domain");
        return;
    }

    // For now, metering domain only publishes events (no commands)
    // Future: could support commands like "enableMetering" or "setMeteringRate"
    if (env.kind == loophole::signal::ipc::IpcKind::Command) {
        std::ostringstream msg;
        msg << "Received metering command: " << env.name;
        LOG_DEBUG({"MeteringDomain"}, msg.str());
    }
}


