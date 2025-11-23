#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>

EngineDomain::EngineDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void EngineDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        std::cout << "[EngineDomain] Ignoring non-command: " << env.kind << std::endl;
        return;
    }

    if (!_engineHost) {
        std::cerr << "[EngineDomain] EngineHost is null" << std::endl;
        return;
    }

    if (env.name == "start") {
        _engineHost->start();
    } else if (env.name == "stop") {
        _engineHost->stop();
    } else if (env.name == "reset") {
        _engineHost->reset();
    } else if (env.name == "shutdown") {
        std::cout << "[EngineDomain] Shutdown requested" << std::endl;
        _engineHost->shutdown();
    } else if (env.name == "heartbeat") {
        // Heartbeat command received - handled by DomainDispatcher to emit event
        std::cout << "[EngineDomain] Heartbeat command received" << std::endl;
    } else {
        std::cout << "[EngineDomain] Unknown command: " << env.name << std::endl;
    }
}

