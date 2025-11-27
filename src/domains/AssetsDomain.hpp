#pragma once

#include "ipc/Router.hpp"
#include <string>
#include <memory>

class EngineHost;

/// Assets domain handler for Signal
///
/// Handles asset registration commands from Pulse
class AssetsDomain : public DomainHandler {
public:
    explicit AssetsDomain(EngineHost* engineHost);
    ~AssetsDomain() override = default;

    void handle(const Envelope& env) override;

private:
    EngineHost* _engineHost;
};

