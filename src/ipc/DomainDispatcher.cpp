/// DomainDispatcher - Routes IPC envelopes to domain handlers
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Local to SignalApp::run() (created in run method)
/// Communication:
///   - Receives envelopes from TcpClientSession handlers
///   - Dispatches to domain handlers synchronously via registry lookup
///   - Domain handlers update EngineHost/TransportState directly
///   - Sends events back to Pulse via TcpClientSession

#include "ipc/DomainDispatcher.hpp"
#include "domains/EngineDomain.hpp"
#include "domains/TransportDomain.hpp"
#include "domains/HardwareDomain.hpp"
#include "domains/MixerDomain.hpp"
#include "domains/NodeDomain.hpp"
#include "domains/AutomationDomain.hpp"
#include "domains/AssetsDomain.hpp"
#include "domains/MeteringDomain.hpp"
#include "logging/Logging.hpp"

namespace loophole::signal::ipc {

DomainDispatcher::DomainDispatcher(EngineHost* engineHost, MeteringService* meteringService)
    : engineHost_(engineHost)
{
    // Register all domain handlers
    domains_.emplace("engine", std::make_unique<EngineDomain>(engineHost_));
    domains_.emplace("transport", std::make_unique<TransportDomain>(engineHost_));
    domains_.emplace("hardware", std::make_unique<HardwareDomain>(engineHost_));
    domains_.emplace("channelMix", std::make_unique<ChannelMixDomain>(engineHost_));
    domains_.emplace("node", std::make_unique<NodeDomain>(engineHost_));
    domains_.emplace("automation", std::make_unique<AutomationDomain>(engineHost_));
    domains_.emplace("assets", std::make_unique<AssetsDomain>(engineHost_));
    domains_.emplace("metering", std::make_unique<MeteringDomain>(meteringService, engineHost_));
}

void DomainDispatcher::handleEnvelope(
    const IpcEnvelope& env,
    const std::shared_ptr<TcpClientSession>& session
) {
    auto it = domains_.find(env.domain);
    if (it != domains_.end()) {
        it->second->handle(env, session);
        return;
    }

    // Unknown domain - log but don't send error (let Pulse handle it)
    std::ostringstream msg;
    msg << "Unknown domain: " << env.domain;
    LOG_WARN({"DomainDispatcher"}, msg.str());
}

} // namespace loophole::signal::ipc
