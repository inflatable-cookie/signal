#include "domains/PluginDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/PluginHost.hpp"
#include "clap/ClapRegistry.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "vst3/Vst3Backend.hpp"
#include <nlohmann/json.hpp>
#include <unordered_map>

namespace {
const char* formatTagForCatalog(PluginFormat format) noexcept {
    switch (format) {
    case PluginFormat::Clap:
        return "clap";
    case PluginFormat::Vst3:
        return "vst3";
    case PluginFormat::Au:
        return "au";
    case PluginFormat::Lv2:
        return "lv2";
    case PluginFormat::Native:
        return "native";
    }
    return "unknown";
}

std::string pluginCatalogKey(const PluginDescriptor& descriptor) {
    return std::string(formatTagForCatalog(descriptor.format)) + ":" + descriptor.id;
}

bool pluginCatalogEntryChanged(
    const PluginDescriptor& before,
    const PluginDescriptor& after
) {
    return before.name != after.name
        || before.numAudioInputs != after.numAudioInputs
        || before.numAudioOutputs != after.numAudioOutputs
        || before.hasMidiInput != after.hasMidiInput
        || before.hasMidiOutput != after.hasMidiOutput;
}

std::optional<std::string> resolveDescriptorBinaryPath(
    PluginHost* pluginHost,
    const PluginDescriptor& descriptor
) {
    if (pluginHost == nullptr) {
        return std::nullopt;
    }

    switch (descriptor.format) {
    case PluginFormat::Clap: {
        auto library = pluginHost->getClapRegistry().getLibrary(descriptor.id);
        if (library) {
            return library->getPath().string();
        }
        return std::nullopt;
    }
    case PluginFormat::Vst3: {
        auto path = pluginHost->getVst3Backend().findPathById(descriptor.id);
        if (path.has_value()) {
            return path->string();
        }
        return std::nullopt;
    }
    default:
        return std::nullopt;
    }
}

nlohmann::json pluginPayloadForDescriptor(
    const PluginDescriptor& descriptor,
    const std::optional<std::string>& binaryPath
) {
    return nlohmann::json{
        {"pluginId", descriptor.id},
        {"binaryPath", binaryPath.has_value() ? nlohmann::json(binaryPath.value()) : nlohmann::json(nullptr)},
        {"plugin", nlohmann::json{{"pluginId", descriptor.id},
                                  {"format", formatTagForCatalog(descriptor.format)},
                                  {"displayName", descriptor.name},
                                  {"manufacturer", nullptr},
                                  {"binaryPath", binaryPath.has_value() ? nlohmann::json(binaryPath.value()) : nlohmann::json(nullptr)}}},
    };
}

std::string canonicalScanLevel(const nlohmann::json& payload) {
    auto level = payload.value("scanLevel", std::string{});
    if (level.empty() && payload.contains("options") && payload["options"].is_object()) {
        level = payload["options"].value("scanLevel", std::string{});
    }

    if (level == "none" || level == "light" || level == "catalog" || level == "full") {
        return level;
    }

    return "catalog";
}
} // namespace

PluginDomain::PluginDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void PluginDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    if (env.domain != "plugin") {
        return;
    }

    if (env.kind != IpcKind::Command) {
        return;
    }

    if (!_engineHost) {
        emitError(
            env,
            session,
            "engine_unavailable",
            "EngineHost is unavailable"
        );
        return;
    }

    if (env.name == "list") {
        handleList(env, session);
        return;
    }

    if (env.name == "rescan") {
        handleRescan(env, session);
        return;
    }

    if (env.name == "cancelScan") {
        handleCancelScan(env, session);
        return;
    }

    if (env.name == "scanStatus") {
        handleScanStatus(env, session);
        return;
    }

    emitError(
        env,
        session,
        "unknown_command",
        "Unknown plugin command"
    );
}

std::optional<loophole::signal::ipc::IpcTarget> PluginDomain::envelopeTargetForOrigin(
    loophole::signal::ipc::IpcOrigin origin
) {
    using namespace loophole::signal::ipc;

    switch (origin) {
    case IpcOrigin::Aura:
        return IpcTarget::Aura;
    case IpcOrigin::Pulse:
        return IpcTarget::Pulse;
    case IpcOrigin::Signal:
        return IpcTarget::Signal;
    case IpcOrigin::Composer:
        return IpcTarget::Composer;
    }

    return std::nullopt;
}

const char* PluginDomain::formatTag(PluginFormat format) noexcept {
    switch (format) {
    case PluginFormat::Clap:
        return "clap";
    case PluginFormat::Vst3:
        return "vst3";
    case PluginFormat::Au:
        return "au";
    case PluginFormat::Lv2:
        return "lv2";
    case PluginFormat::Native:
        return "native";
    }

    return "unknown";
}

void PluginDomain::handleList(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    auto* pluginHost = _engineHost->pluginHost();
    if (!pluginHost) {
        emitError(
            commandEnv,
            session,
            "plugin_host_unavailable",
            "Plugin host is unavailable"
        );
        return;
    }

    const auto descriptors = pluginHost->listPlugins();

    IpcEnvelope response;
    response.version = 1;
    response.id = "plugin-list-" + commandEnv.id;
    response.correlationId = commandEnv.id;
    response.timestamp = currentTimestamp();
    response.origin = IpcOrigin::Signal;
    response.target = target.value();
    response.domain = "plugin";
    response.kind = IpcKind::Event;
    response.name = "list";
    response.priority = commandEnv.priority;

    nlohmann::json plugins = nlohmann::json::array();
    for (const auto& descriptor : descriptors) {
        auto binaryPath = resolveDescriptorBinaryPath(pluginHost, descriptor);
        plugins.push_back({
            {"pluginId", descriptor.id},
            {"format", formatTag(descriptor.format)},
            {"displayName", descriptor.name},
            {"manufacturer", nullptr},
            {"binaryPath", binaryPath.has_value() ? nlohmann::json(binaryPath.value()) : nlohmann::json(nullptr)},
        });
    }

    response.payload = {
        {"plugins", plugins},
    };
    response.error = std::nullopt;

    session->send(response);
}

void PluginDomain::handleRescan(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    auto* pluginHost = _engineHost->pluginHost();
    if (!pluginHost) {
        emitError(
            commandEnv,
            session,
            "plugin_host_unavailable",
            "Plugin host is unavailable"
        );
        return;
    }

    std::string scanId = commandEnv.payload.value(
        "scanId",
        "pluginScan:" + currentTimestamp()
    );
    const auto scanLevel = canonicalScanLevel(commandEnv.payload);
    const bool fullScan = scanLevel == "full";

    {
        std::lock_guard<std::mutex> lock(scanMutex_);
        if (!activeScan_.has_value() && scanThread_.joinable()) {
            scanThread_.join();
        }
        if (scanThread_.joinable()) {
            emitError(
                commandEnv,
                session,
                "scan_in_progress",
                "A plugin scan is already in progress"
            );
            return;
        }

        activeScan_ = ScanState{
            .scanId = scanId,
            .scanLevel = scanLevel,
            .target = target.value(),
            .priority = commandEnv.priority,
        };
    }

    emitEvent(
        session,
        target,
        commandEnv.priority,
        commandEnv.id,
        "rescan",
        nlohmann::json{{"scanId", scanId}, {"scanLevel", scanLevel}}
    );
    emitEvent(
        session,
        target,
        commandEnv.priority,
        commandEnv.id,
        "scanStarted",
        nlohmann::json{
            {"scanId", scanId},
            {"fullScan", fullScan},
            {"scanLevel", scanLevel},
        }
    );

    scanThread_ = std::jthread(
        [this,
         scanId,
         scanLevel,
         target,
         priority = commandEnv.priority,
         weakSession =
             std::weak_ptr<loophole::signal::ipc::TcpClientSession>(session)](
            std::stop_token token
        ) {
            runScan(scanId, scanLevel, target, priority, weakSession, token);
        }
    );
}

void PluginDomain::handleCancelScan(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    std::optional<std::string> activeScanId = std::nullopt;
    bool cancelled = false;
    {
        std::lock_guard<std::mutex> lock(scanMutex_);
        if (scanThread_.joinable()) {
            scanThread_.request_stop();
            cancelled = true;
            if (activeScan_.has_value()) {
                activeScanId = activeScan_->scanId;
            }
        }
    }

    emitEvent(
        session,
        target,
        commandEnv.priority,
        commandEnv.id,
        "cancelScan",
        [&activeScanId, cancelled]() {
            nlohmann::json payload = nlohmann::json::object();
            if (activeScanId.has_value()) {
                payload["scanId"] = activeScanId.value();
            }
            payload["cancelled"] = cancelled;
            return payload;
        }()
    );
}

void PluginDomain::handleScanStatus(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    auto* pluginHost = _engineHost->pluginHost();
    if (!pluginHost) {
        emitError(
            commandEnv,
            session,
            "plugin_host_unavailable",
            "Plugin host is unavailable"
        );
        return;
    }

    auto status = pluginHost->scanStatus();
    std::optional<std::string> scanId = std::nullopt;
    std::optional<std::string> scanLevel = std::nullopt;
    {
        std::lock_guard<std::mutex> lock(scanMutex_);
        if (activeScan_.has_value()) {
            scanId = activeScan_->scanId;
            scanLevel = activeScan_->scanLevel;
        }
    }

    nlohmann::json payload = nlohmann::json::object();
    if (scanId.has_value()) {
        payload["scanId"] = scanId.value();
    }
    if (scanLevel.has_value()) {
        payload["scanLevel"] = scanLevel.value();
    }
    payload["state"] = status.state_tag();
    payload["pluginCount"] = status.plugin_count;
    payload["clapCount"] = status.clap_plugin_count;
    payload["vst3Count"] = status.vst3_plugin_count;
    payload["message"] = status.last_error.value_or("");
    if (status.duration.has_value()) {
        payload["durationMs"] = status.duration->count();
    }

    emitEvent(
        session,
        target,
        commandEnv.priority,
        commandEnv.id,
        "scanStatus",
        payload
    );
}

void PluginDomain::runScan(
    std::string scanId,
    std::string scanLevel,
    std::optional<loophole::signal::ipc::IpcTarget> target,
    loophole::signal::ipc::IpcPriority priority,
    std::weak_ptr<loophole::signal::ipc::TcpClientSession> weakSession,
    std::stop_token stopToken
) {
    auto session = weakSession.lock();
    if (!session) {
        std::lock_guard<std::mutex> lock(scanMutex_);
        activeScan_ = std::nullopt;
        return;
    }

    auto* pluginHost = _engineHost != nullptr ? _engineHost->pluginHost() : nullptr;
    if (!pluginHost) {
        emitEvent(
            session,
            target,
            priority,
            std::nullopt,
            "scanFailed",
            nlohmann::json{
                {"scanId", scanId},
                {"scanLevel", scanLevel},
                {"code", "plugin_host_unavailable"},
                {"message", "Plugin host is unavailable"},
            }
        );
        std::lock_guard<std::mutex> lock(scanMutex_);
        activeScan_ = std::nullopt;
        return;
    }

    if (scanLevel == "none") {
        emitEvent(
            session,
            target,
            priority,
            std::nullopt,
            "scanCompleted",
            nlohmann::json{
                {"scanId", scanId},
                {"scanLevel", scanLevel},
                {"summary", nlohmann::json{{"added", 0}, {"removed", 0}, {"updated", 0}}},
            }
        );
        std::lock_guard<std::mutex> lock(scanMutex_);
        activeScan_ = std::nullopt;
        return;
    }

    pluginHost->scanPlugins(stopToken);
    const auto status = pluginHost->scanStatus();

    if (status.state == PluginHost::PluginScanState::Failed) {
        emitEvent(
            session,
            target,
            priority,
            std::nullopt,
            "scanFailed",
            nlohmann::json{
                {"scanId", scanId},
                {"scanLevel", scanLevel},
                {"code", "plugin_scan_failed"},
                {"message", status.last_error.value_or("Plugin scan failed")},
            }
        );
    } else if (status.state == PluginHost::PluginScanState::Cancelled) {
        emitEvent(
            session,
            target,
            priority,
            std::nullopt,
            "scanFailed",
            nlohmann::json{
                {"scanId", scanId},
                {"scanLevel", scanLevel},
                {"code", "plugin_scan_cancelled"},
                {"message", "Plugin scan cancelled"},
            }
        );
    } else {
        const auto descriptors = pluginHost->listPlugins();
        std::vector<PluginDescriptor> previousCatalog;
        {
            std::lock_guard<std::mutex> lock(scanMutex_);
            previousCatalog = lastEmittedCatalog_;
        }

        std::unordered_map<std::string, PluginDescriptor> previousByKey;
        std::unordered_map<std::string, PluginDescriptor> currentByKey;
        previousByKey.reserve(previousCatalog.size());
        currentByKey.reserve(descriptors.size());

        for (const auto& descriptor : previousCatalog) {
            previousByKey.emplace(pluginCatalogKey(descriptor), descriptor);
        }
        for (const auto& descriptor : descriptors) {
            currentByKey.emplace(pluginCatalogKey(descriptor), descriptor);
        }

        std::uint64_t addedCount = 0;
        std::uint64_t removedCount = 0;
        std::uint64_t updatedCount = 0;

        for (const auto& descriptor : previousCatalog) {
            const auto key = pluginCatalogKey(descriptor);
            if (currentByKey.find(key) != currentByKey.end()) {
                continue;
            }

            emitEvent(
                session,
                target,
                priority,
                std::nullopt,
                "removed",
                nlohmann::json{
                    {"scanId", scanId},
                    {"pluginId", descriptor.id},
                }
            );
            removedCount += 1;
        }

        for (const auto& descriptor : descriptors) {
            const auto key = pluginCatalogKey(descriptor);
            const auto previous = previousByKey.find(key);
            if (previous == previousByKey.end()) {
                nlohmann::json payload = pluginPayloadForDescriptor(
                    descriptor,
                    resolveDescriptorBinaryPath(pluginHost, descriptor)
                );
                payload["scanId"] = scanId;
                emitEvent(
                    session,
                    target,
                    priority,
                    std::nullopt,
                    "added",
                    payload
                );
                addedCount += 1;
                continue;
            }

            if (!pluginCatalogEntryChanged(previous->second, descriptor)) {
                continue;
            }

            nlohmann::json payload = pluginPayloadForDescriptor(
                descriptor,
                resolveDescriptorBinaryPath(pluginHost, descriptor)
            );
            payload["scanId"] = scanId;
            emitEvent(
                session,
                target,
                priority,
                std::nullopt,
                "updated",
                payload
            );
            updatedCount += 1;
        }

        {
            std::lock_guard<std::mutex> lock(scanMutex_);
            lastEmittedCatalog_ = descriptors;
        }

        emitEvent(
            session,
            target,
            priority,
            std::nullopt,
            "scanCompleted",
            nlohmann::json{{"scanId", scanId},
                           {"scanLevel", scanLevel},
                           {"summary", nlohmann::json{{"added", addedCount},
                                                      {"removed", removedCount},
                                                      {"updated", updatedCount}}}}
        );
    }

    std::lock_guard<std::mutex> lock(scanMutex_);
    if (activeScan_.has_value() && activeScan_->scanId == scanId) {
        activeScan_ = std::nullopt;
    }
}

void PluginDomain::emitEvent(
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    std::optional<loophole::signal::ipc::IpcTarget> target,
    loophole::signal::ipc::IpcPriority priority,
    std::optional<std::string> correlationId,
    const std::string& name,
    nlohmann::json payload
) const {
    using namespace loophole::signal::ipc;

    if (!target.has_value()) {
        return;
    }

    IpcEnvelope eventEnv;
    eventEnv.version = 1;
    eventEnv.id = "plugin-" + name + "-" + currentTimestamp();
    eventEnv.correlationId = correlationId;
    eventEnv.timestamp = currentTimestamp();
    eventEnv.origin = IpcOrigin::Signal;
    eventEnv.target = target.value();
    eventEnv.domain = "plugin";
    eventEnv.kind = IpcKind::Event;
    eventEnv.name = name;
    eventEnv.priority = priority;
    eventEnv.payload = std::move(payload);
    eventEnv.error = std::nullopt;

    session->send(eventEnv);
}

void PluginDomain::emitError(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    const std::string& code,
    const std::string& message
) const {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    IpcEnvelope errorEnv;
    errorEnv.version = 1;
    errorEnv.id = "plugin-error-" + commandEnv.id;
    errorEnv.correlationId = commandEnv.id;
    errorEnv.timestamp = currentTimestamp();
    errorEnv.origin = IpcOrigin::Signal;
    errorEnv.target = target.value();
    errorEnv.domain = "plugin";
    errorEnv.kind = IpcKind::Error;
    errorEnv.name = commandEnv.name;
    errorEnv.priority = commandEnv.priority;
    errorEnv.payload = nlohmann::json::object();
    IpcErrorInfo errorInfo;
    errorInfo.code = code;
    errorInfo.message = message;
    errorInfo.details = nlohmann::json::object();
    errorEnv.error = errorInfo;

    session->send(errorEnv);
}
