#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <memory>
#include <optional>
#include <string>
#include <vector>

class EngineHost;
class PluginNode;

class ParameterDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit ParameterDomain(EngineHost* engineHost);
    ~ParameterDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    struct Scope {
        std::string nodeId;
        std::optional<std::string> pluginInstanceId;
    };

    std::optional<Scope> parseScope(const nlohmann::json& payload) const;
    PluginNode* resolvePluginNode(const Scope& scope) const;
    std::optional<loophole::signal::ipc::IpcTarget> envelopeTargetForOrigin(
        loophole::signal::ipc::IpcOrigin origin
    ) const;

    void handleRequestDescriptors(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleRequestValues(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleSetValue(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitError(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        const std::string& code,
        const std::string& message,
        const nlohmann::json& details
    ) const;

    EngineHost* _engineHost;
};
