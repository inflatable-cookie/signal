#include "domains/TransportDomain.hpp"
#include "core/EngineHost.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>

TransportDomain::TransportDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void TransportDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        std::cout << "[TransportDomain] Ignoring non-command: " << env.kind << std::endl;
        return;
    }

    if (env.name == "play") {
        std::cout << "[TransportDomain] Play command received" << std::endl;
        // TODO: Implement transport play logic
    } else if (env.name == "stop") {
        std::cout << "[TransportDomain] Stop command received" << std::endl;
        // TODO: Implement transport stop logic
    } else if (env.name == "seek") {
        std::cout << "[TransportDomain] Seek command received" << std::endl;
        // TODO: Implement transport seek logic
    } else {
        std::cout << "[TransportDomain] Unknown command: " << env.name << std::endl;
    }
}

