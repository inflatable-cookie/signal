#include "ipc/Envelope.hpp"

Envelope makeBasicEnvelope(
    const std::string& domain,
    const std::string& kind,
    const std::string& name
) {
    Envelope env;
    env.v = 1;
    env.domain = domain;
    env.kind = kind;
    env.name = name;
    env.priority = "normal";
    // TODO: Generate proper id, ts, etc.
    return env;
}

