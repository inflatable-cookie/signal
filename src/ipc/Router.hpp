#pragma once

#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

struct Envelope;

class DomainHandler {
public:
    virtual ~DomainHandler() = default;
    virtual void handle(const Envelope& env) = 0;
};

class IpcRouter {
public:
    void registerHandler(
        const std::string& domain,
        std::shared_ptr<DomainHandler> handler
    );
    void dispatch(const Envelope& env) const;

private:
    std::unordered_map<std::string, std::vector<std::shared_ptr<DomainHandler>>> _handlers;
};

