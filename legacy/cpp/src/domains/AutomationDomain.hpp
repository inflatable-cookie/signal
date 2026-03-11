#pragma once

/// AutomationDomain - IPC domain handler for automation updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by DomainDispatcher
/// Communication:
///   - Receives commands from Pulse (automation.setCurvesForSession)
///   - Updates AutomationService state
///   - Changes are applied in real-time by audio thread

#include "ipc/IpcDomainHandler.hpp"
#include <memory>

class EngineHost;

class AutomationDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit AutomationDomain(EngineHost* engineHost);
    ~AutomationDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handleSetCurvesForSession(const nlohmann::json& payload);
    void handleAutomationSnapshot(const nlohmann::json& payload);
    void handleUpdateCurve(const nlohmann::json& payload);

    EngineHost* _engineHost;
};

