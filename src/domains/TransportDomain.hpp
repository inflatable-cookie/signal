#pragma once

#include "ipc/Router.hpp"
#include <memory>

class EngineHost;

class TransportDomain : public DomainHandler {
public:
    explicit TransportDomain(EngineHost* engineHost);
    ~TransportDomain() override = default;

    void handle(const Envelope& env) override;

private:
    EngineHost* _engineHost;
};

