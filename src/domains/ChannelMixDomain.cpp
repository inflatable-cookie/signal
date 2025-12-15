#include "domains/ChannelMixDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/ChannelMixService.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphNode.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

ChannelMixDomain::ChannelMixDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
    LOG_INFO({"ChannelMixDomain"}, "Initialised");
}

void ChannelMixDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "channelMix") {
        LOG_DEBUG({"ChannelMixDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"ChannelMixDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (env.name == "updateChannel") {
        handleUpdateChannel(env.payload);
    } else {
        LOG_WARN({"ChannelMixDomain"}, std::string("Received unhandled channelMix command: ") + env.name);
    }
}

void ChannelMixDomain::handleUpdateChannel(const nlohmann::json& payload) {
    try {
        std::string channelId = payload["channelId"];
        float gain = payload["gain"];
        float pan = payload.value("pan", 0.0f); // Default to 0.0 if not present (backward compatibility)
        bool isMuted = payload["isMuted"];
        bool isSoloed = payload["isSoloed"];
        bool effectiveMuted = payload["effectiveMuted"];

        // Update ChannelMixService state
        _engineHost->channelMix().updateChannel(
            channelId,
            gain,
            pan,
            isMuted,
            isSoloed,
            effectiveMuted
        );

        std::ostringstream msg;
        msg << "Updated channel " << channelId
            << " gain=" << gain
            << " pan=" << pan
            << " muted=" << isMuted
            << " soloed=" << isSoloed
            << " effectiveMuted=" << effectiveMuted;
        LOG_DEBUG({"ChannelMixDomain"}, msg.str());
    } catch (const std::exception& e) {
        LOG_ERROR({"ChannelMixDomain"}, std::string("Failed to parse updateChannel payload: ") + e.what());
    }
}
