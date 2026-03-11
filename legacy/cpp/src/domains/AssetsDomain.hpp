#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <string>
#include <memory>

class EngineHost;

/// Assets domain handler for Signal
///
/// Handles asset registration commands from Pulse
class AssetsDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit AssetsDomain(EngineHost* engineHost);
    ~AssetsDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    void handleRegisterAudioAsset(const nlohmann::json& payload);

    EngineHost* _engineHost;
};

