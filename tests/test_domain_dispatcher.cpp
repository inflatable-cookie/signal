#include <catch2/catch_test_macros.hpp>
#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "core/EngineHost.hpp"
#include "core/MeteringService.hpp"
#include <asio/ip/tcp.hpp>
#include <memory>

using namespace loophole::signal::ipc;

TEST_CASE("DomainDispatcher routes envelopes to correct domain", "[domain-dispatcher]") {
    // Create EngineHost and MeteringService
    auto engineHost = std::make_unique<EngineHost>();
    auto meteringService = &engineHost->metering();

    // Create dispatcher
    DomainDispatcher dispatcher(engineHost.get(), meteringService);

    // Create test envelope for engine domain
    IpcEnvelope env;
    env.version = 1;
    env.id = "test-1";
    env.timestamp = "2025-01-01T00:00:00Z";
    env.origin = IpcOrigin::Pulse;
    env.target = IpcTarget::Signal;
    env.domain = "engine";
    env.kind = IpcKind::Command;
    env.name = "start";
    env.priority = IpcPriority::Normal;
    env.payload = nlohmann::json::object();

    // Create a minimal session - we can't easily mock TcpClientSession due to socket requirement
    // So we'll just verify the dispatcher doesn't crash and routes correctly
    // Full integration tests would use real TcpClientSession with actual socket
    asio::io_context io;
    asio::ip::tcp::socket socket(io);
    auto session = std::make_shared<TcpClientSession>(
        std::move(socket),
        [](const IpcEnvelope&, std::shared_ptr<TcpClientSession>) {}
    );

    // Dispatch envelope - should not throw
    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, session));

    // Verify engine was started (check state)
    auto state = engineHost->state();
    REQUIRE((state == EngineHost::State::Running || state == EngineHost::State::Starting));
}

TEST_CASE("DomainDispatcher handles unknown domains gracefully", "[domain-dispatcher]") {
    auto engineHost = std::make_unique<EngineHost>();
    auto meteringService = &engineHost->metering();

    DomainDispatcher dispatcher(engineHost.get(), meteringService);

    // Create envelope for unknown domain
    IpcEnvelope env;
    env.version = 1;
    env.id = "test-2";
    env.timestamp = "2025-01-01T00:00:00Z";
    env.origin = IpcOrigin::Pulse;
    env.target = IpcTarget::Signal;
    env.domain = "unknown";
    env.kind = IpcKind::Command;
    env.name = "test";
    env.priority = IpcPriority::Normal;
    env.payload = nlohmann::json::object();

    asio::io_context io;
    asio::ip::tcp::socket socket(io);
    auto session = std::make_shared<TcpClientSession>(
        std::move(socket),
        [](const IpcEnvelope&, std::shared_ptr<TcpClientSession>) {}
    );

    // Should not throw, just log warning
    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, session));
}

