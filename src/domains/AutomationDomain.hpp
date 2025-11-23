#pragma once

/// AutomationDomain - IPC domain handler for automation updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by IpcRouter
/// Communication:
///   - Receives commands from Pulse (automation.setCurvesForSession)
///   - Updates AutomationService state
///   - Changes are applied in real-time by audio thread

#include "ipc/Router.hpp"
#include "ipc/Envelope.hpp"
#include <memory>

class EngineHost;

class AutomationDomain : public DomainHandler {
public:
    explicit AutomationDomain(EngineHost* engineHost);
    ~AutomationDomain() override = default;

    void handle(const Envelope& env) override;

private:
    EngineHost* _engineHost;
};

