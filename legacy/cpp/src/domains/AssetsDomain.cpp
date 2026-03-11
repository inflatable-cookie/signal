#include "domains/AssetsDomain.hpp"
#include "core/EngineHost.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

AssetsDomain::AssetsDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void AssetsDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "assets") {
        LOG_DEBUG({"AssetsDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"AssetsDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"AssetsDomain"}, "EngineHost is null");
        return;
    }

    if (env.name == "registerAudioAsset") {
        handleRegisterAudioAsset(env.payload);
    } else {
        LOG_WARN({"AssetsDomain"}, std::string("Unknown command: ") + env.name);
    }
}

void AssetsDomain::handleRegisterAudioAsset(const nlohmann::json& payload) {
    try {
        std::string assetId = payload.value("assetId", "");
        if (assetId.empty()) {
            LOG_ERROR({"AssetsDomain"}, "Missing assetId in registerAudioAsset");
            return;
        }

        std::string path = payload.value("path", "");
        if (path.empty()) {
            LOG_ERROR({"AssetsDomain"}, "Missing path in registerAudioAsset");
            return;
        }

        uint32_t channels = payload.value("channels", 2u);
        uint32_t sampleRate = payload.value("sampleRate", 44100u);
        uint64_t frames = payload.value("frames", 0u);

        // Diagnostic logging: asset registration
        std::ostringstream msg;
        msg << "Registering audio asset: id='" << assetId
            << "', path='" << path
            << "', channels=" << channels
            << ", sampleRate=" << sampleRate
            << ", frames=" << frames;
        LOG_INFO({"AssetsDomain"}, msg.str());

        if (frames == 0) {
            LOG_WARN({"AssetsDomain"}, std::string("Invalid frames count (0) for asset: ") + assetId);
        }

        _engineHost->prepareAudioAsset(assetId, path, channels, sampleRate, frames);
        LOG_DEBUG({"AssetsDomain"}, std::string("Called prepareAudioAsset for: ") + assetId);
    } catch (const std::exception& e) {
        LOG_ERROR({"AssetsDomain"}, std::string("Failed to parse registerAudioAsset payload: ") + e.what());
    }
}

