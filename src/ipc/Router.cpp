#include "ipc/Router.hpp"
#include "ipc/Envelope.hpp"
#include "logging/Logging.hpp"
#include <sstream>

void IpcRouter::registerHandler(
    const std::string& domain,
    std::shared_ptr<DomainHandler> handler
) {
    _handlers[domain].push_back(handler);
    LOG_INFO({"IpcRouter"}, std::string("Registered handler for domain: ") + domain);
}

void IpcRouter::dispatch(const Envelope& env) const {
    auto it = _handlers.find(env.domain);
    if (it == _handlers.end()) {
        LOG_WARN({"IpcRouter"}, std::string("No handler for domain: ") + env.domain);
        return;
    }

    for (const auto& handler : it->second) {
        handler->handle(env);
    }
}

