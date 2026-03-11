#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <memory>
#include <optional>
#include <string>
#include <nlohmann/json.hpp>

class EngineHost;

class HardwareDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit HardwareDomain(EngineHost* engineHost);
    ~HardwareDomain() override = default;

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

    void sendControlDeviceInventory(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    EngineHost* _engineHost;
};
