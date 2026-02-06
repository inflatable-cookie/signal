#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <memory>
#include <optional>
#include <string>

class EngineHost;
enum class PluginFormat;

class PluginDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit PluginDomain(EngineHost* engineHost);
    ~PluginDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    static std::optional<loophole::signal::ipc::IpcTarget> envelopeTargetForOrigin(
        loophole::signal::ipc::IpcOrigin origin
    );
    static const char* formatTag(PluginFormat format) noexcept;

    void handleList(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );

    void emitError(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        const std::string& code,
        const std::string& message
    ) const;

    EngineHost* _engineHost;
};
