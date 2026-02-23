#pragma once

#include "ipc/IpcDomainHandler.hpp"
#include <nlohmann/json.hpp>
#include <memory>
#include <mutex>
#include <optional>
#include <cstdint>
#include <string>
#include <thread>
#include <vector>

#include "core/PluginInstance.hpp"

class EngineHost;
enum class PluginFormat;

class PluginDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit PluginDomain(EngineHost* engineHost);
    ~PluginDomain() override = default;

    struct LightCacheEntry {
        std::optional<std::string> pluginId;
        std::string binaryPath;
        std::optional<std::uint64_t> fileMtimeUnix;
        std::optional<std::uint64_t> fileSizeBytes;
    };

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    struct ScanState {
        std::string scanId;
        std::string scanLevel{"catalog"};
        loophole::signal::ipc::IpcTarget target{loophole::signal::ipc::IpcTarget::Pulse};
        loophole::signal::ipc::IpcPriority priority{loophole::signal::ipc::IpcPriority::Normal};
    };

    static std::optional<loophole::signal::ipc::IpcTarget> envelopeTargetForOrigin(
        loophole::signal::ipc::IpcOrigin origin
    );
    static const char* formatTag(PluginFormat format) noexcept;

    void handleList(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleRescan(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleCancelScan(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleScanStatus(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void runScan(
        std::string scanId,
        std::string scanLevel,
        std::vector<LightCacheEntry> lightCacheEntries,
        std::optional<loophole::signal::ipc::IpcTarget> target,
        loophole::signal::ipc::IpcPriority priority,
        std::weak_ptr<loophole::signal::ipc::TcpClientSession> weakSession,
        std::stop_token stopToken
    );
    void emitEvent(
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        std::optional<loophole::signal::ipc::IpcTarget> target,
        loophole::signal::ipc::IpcPriority priority,
        std::optional<std::string> correlationId,
        const std::string& name,
        nlohmann::json payload
    ) const;

    void emitError(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        const std::string& code,
        const std::string& message
    ) const;

    EngineHost* _engineHost;
    mutable std::mutex scanMutex_;
    std::optional<ScanState> activeScan_;
    std::vector<PluginDescriptor> lastEmittedCatalog_;
    std::jthread scanThread_;
};
