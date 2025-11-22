#include "ipc/Router.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>

void IpcRouter::registerHandler(
    const std::string& domain,
    std::shared_ptr<DomainHandler> handler
) {
    _handlers[domain].push_back(handler);
    std::cout << "[IpcRouter] Registered handler for domain: " << domain << std::endl;
}

void IpcRouter::dispatch(const Envelope& env) const {
    auto it = _handlers.find(env.domain);
    if (it == _handlers.end()) {
        std::cout << "[IpcRouter] No handler for domain: " << env.domain << std::endl;
        return;
    }

    std::cout << "[IpcRouter] Dispatching to domain: " << env.domain
              << ", name: " << env.name << std::endl;

    for (const auto& handler : it->second) {
        handler->handle(env);
    }
}

