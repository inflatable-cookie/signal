#include "domains/MeteringDomain.hpp"
#include "core/MeteringService.hpp"
#include "core/EngineHost.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>

MeteringDomain::MeteringDomain(MeteringService* meteringService, EngineHost* engineHost)
    : _meteringService(meteringService)
    , _engineHost(engineHost)
{
}

void MeteringDomain::handle(const Envelope& env) {
    // For now, metering domain only publishes events (no commands)
    // Future: could support commands like "enableMetering" or "setMeteringRate"
    std::cout << "[MeteringDomain] Received envelope: " << env.domain << "." << env.name << std::endl;
}


