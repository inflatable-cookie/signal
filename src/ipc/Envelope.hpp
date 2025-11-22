#pragma once

#include <string>

struct Envelope {
    int v = 1;
    std::string id;
    std::string cid;
    std::string ts;
    std::string origin;
    std::string target;
    std::string domain;
    std::string kind;    // "command", "event", "snapshot", "error"
    std::string name;    // e.g. "start", "state", etc.
    std::string priority;
    std::string payload; // For now: raw JSON string or placeholder
    std::string error;   // For now: empty if no error
};

// Helper function for tests
Envelope makeBasicEnvelope(
    const std::string& domain,
    const std::string& kind,
    const std::string& name
);

