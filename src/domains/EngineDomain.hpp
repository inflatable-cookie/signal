#pragma once

#include "ipc/Router.hpp"
#include <memory>

class EngineHost;

class EngineDomain : public DomainHandler {
public:
    explicit EngineDomain(EngineHost* engineHost);
    ~EngineDomain() override = default;

    void handle(const Envelope& env) override;

private:
    EngineHost* _engineHost;
};

