#include <catch2/catch_test_macros.hpp>
#include <catch2/catch_approx.hpp>
#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "core/EngineHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include <asio/ip/tcp.hpp>
#include <memory>

using namespace loophole::signal::ipc;

TEST_CASE("node.setParameter muted applies to PluginNode and silences output", "[node][muted]") {
    // Prepare EngineHost with a simple graph containing a PluginNode (AudioFx).
    auto engineHost = std::make_unique<EngineHost>();
    engineHost->prepareEngine(44100, 512);

    using nlohmann::json;
    json j = {
        {"id", "test-graph"},
        {"nodes", json::array({
             json{
                 {"nodeId", "fx-1"},
                 {"kind", "audio-fx"},
                 {"trackId", "track-1"},
                 {"numAudioInputs", 2},
                 {"numAudioOutputs", 2}
             },
             json{
                 {"nodeId", "device"},
                 {"kind", "hardware-audio-output"},
                 {"numAudioInputs", 2},
                 {"numAudioOutputs", 2}
             }
         })},
        {"connections", json::array()}
    };

    auto snapshotOpt = GraphSnapshot::fromJson(j);
    REQUIRE(snapshotOpt.has_value());
    engineHost->loadGraphSnapshot(snapshotOpt.value());

    auto* node = engineHost->graphEngine().findNode("fx-1");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::AudioFx);

    auto* fx = dynamic_cast<PluginNode*>(node);
    REQUIRE(fx != nullptr);

    // Seed input with non-zero audio so we can verify mute behaviour.
    fx->io.audioIn.setSample(0, 0, 1.0f);
    fx->io.audioIn.setSample(0, 1, 1.0f);

    // Dispatch node.setParameter muted=true through the DomainDispatcher.
    auto meteringService = &engineHost->metering();
    DomainDispatcher dispatcher(engineHost.get(), meteringService);

    IpcEnvelope env;
    env.version = 1;
    env.id = "test-1";
    env.timestamp = "2025-01-01T00:00:00Z";
    env.origin = IpcOrigin::Pulse;
    env.target = IpcTarget::Signal;
    env.domain = "node";
    env.kind = IpcKind::Command;
    env.name = "setParameter";
    env.priority = IpcPriority::Normal;
    env.payload = json{
        {"nodeId", "fx-1"},
        {"parameterId", "muted"},
        {"value", true}
    };

    asio::io_context io;
    asio::ip::tcp::socket socket(io);
    auto session = std::make_shared<TcpClientSession>(
        std::move(socket),
        [](const IpcEnvelope&, std::shared_ptr<TcpClientSession>) {}
    );

    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, session));
    REQUIRE(fx->isMuted());

    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;

    fx->process(npc);

    // Muted plugin nodes must emit silence (and should not invoke plugin processing).
    REQUIRE(fx->io.audioOut.getSample(0, 0) == Catch::Approx(0.0f));
    REQUIRE(fx->io.audioOut.getSample(0, 1) == Catch::Approx(0.0f));
}
