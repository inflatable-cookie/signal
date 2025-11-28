#pragma once

#include "ipc/Router.hpp"
#include "ipc/IpcDomainHandler.hpp"
#include <memory>
#include <optional>
#include <string>
#include <nlohmann/json.hpp>

class EngineHost;

class HardwareDomain : public DomainHandler, public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit HardwareDomain(IpcRouter* router, EngineHost* engineHost);
    ~HardwareDomain() override = default;

    // Legacy DomainHandler interface (for router)
    void handle(const Envelope& env) override;

    // New IpcDomainHandler interface
    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void sendListOutputDevicesResponse(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void sendSelectOutputDeviceResponse(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        const std::string& deviceId
    );

    IpcRouter* _router;
    EngineHost* _engineHost;
};

