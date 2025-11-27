#include "domains/MeteringDomain.hpp"
#include "core/MeteringService.hpp"
#include "core/EngineHost.hpp"
#include "ipc/Envelope.hpp"
#include "logging/Logging.hpp"
#include <sstream>

MeteringDomain::MeteringDomain(MeteringService* meteringService, EngineHost* engineHost)
    : _meteringService(meteringService)
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


