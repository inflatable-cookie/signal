#include <catch2/catch_test_macros.hpp>
#include "ipc/DomainDispatcher.hpp"
#include "ipc/BinaryEnvelopeCodec.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "core/EngineHost.hpp"
#include "core/MeteringService.hpp"
#include <asio/ip/tcp.hpp>
#include <chrono>
#include <memory>
#include <thread>
#include <vector>

using namespace loophole::signal::ipc;

namespace {

class SessionHarness {
public:
    SessionHarness()
        : _clientSocket(_io)
    {
        asio::ip::tcp::acceptor acceptor(_io, asio::ip::tcp::endpoint(asio::ip::tcp::v4(), 0));

        _clientSocket.connect(acceptor.local_endpoint());
        asio::ip::tcp::socket serverSocket(_io);
        acceptor.accept(serverSocket);

        _session = std::make_shared<TcpClientSession>(
            std::move(serverSocket),
            [](const IpcEnvelope&, std::shared_ptr<TcpClientSession>) {}
        );
    }

    std::shared_ptr<TcpClientSession> session() const {
        return _session;
    }

    IpcEnvelope readEnvelope(std::chrono::milliseconds timeout = std::chrono::milliseconds(1200)) {
        if (!_magicConsumed) {
            auto magic = readExact(4, timeout);
            REQUIRE(magic.size() == 4);
            REQUIRE(magic[0] == 'L');
            REQUIRE(magic[1] == 'P');
            REQUIRE(magic[2] == 'F');
            REQUIRE(magic[3] == '1');
            _magicConsumed = true;
        }

        auto lenBytes = readExact(4, timeout);
        REQUIRE(lenBytes.size() == 4);
        const std::uint32_t len = static_cast<std::uint32_t>(lenBytes[0])
            | (static_cast<std::uint32_t>(lenBytes[1]) << 8)
            | (static_cast<std::uint32_t>(lenBytes[2]) << 16)
            | (static_cast<std::uint32_t>(lenBytes[3]) << 24);
        REQUIRE(len > 1);

        auto frame = readExact(len, timeout);
        REQUIRE(frame.size() == len);
        REQUIRE(frame[0] == 3); // framed-binary kind: binary-envelope-v2

        std::string decodeErr;
        auto decoded = decodeBinaryEnvelopeV2(
            std::span<const std::uint8_t>(frame.data() + 1, frame.size() - 1),
            decodeErr
        );
        REQUIRE(decoded.has_value());
        REQUIRE(decodeErr.empty());
        return decoded.value();
    }

private:
    std::vector<std::uint8_t> readExact(
        std::size_t size,
        std::chrono::milliseconds timeout
    ) {
        std::vector<std::uint8_t> buffer(size);
        std::size_t offset = 0;
        const auto startedAt = std::chrono::steady_clock::now();

        while (offset < size) {
            std::error_code availableErr;
            const auto available = _clientSocket.available(availableErr);
            if (availableErr) {
                FAIL("socket availability check failed: " + availableErr.message());
            }

            if (available == 0) {
                if (std::chrono::steady_clock::now() - startedAt > timeout) {
                    FAIL("timed out waiting for framed envelope bytes");
                }
                std::this_thread::sleep_for(std::chrono::milliseconds(2));
                continue;
            }

            std::error_code readErr;
            const auto bytesRead = _clientSocket.read_some(
                asio::buffer(buffer.data() + offset, size - offset),
                readErr
            );
            if (readErr) {
                FAIL("socket read failed: " + readErr.message());
            }
            offset += bytesRead;
        }

        return buffer;
    }

    asio::io_context _io;
    asio::ip::tcp::socket _clientSocket;
    std::shared_ptr<TcpClientSession> _session;
    bool _magicConsumed = false;
};

IpcEnvelope makeCommand(const std::string& id, const std::string& domain, const std::string& name) {
    IpcEnvelope env;
    env.version = 1;
    env.id = id;
    env.timestamp = "2025-01-01T00:00:00Z";
    env.origin = IpcOrigin::Pulse;
    env.target = IpcTarget::Signal;
    env.domain = domain;
    env.kind = IpcKind::Command;
    env.name = name;
    env.priority = IpcPriority::Normal;
    env.payload = nlohmann::json::object();
    return env;
}

} // namespace

TEST_CASE("DomainDispatcher accepts engine start and emits correlated state event", "[domain-dispatcher][engine]") {
    auto engineHost = std::make_unique<EngineHost>();
    DomainDispatcher dispatcher(engineHost.get(), &engineHost->metering());
    SessionHarness harness;
    auto env = makeCommand("engine-cmd-1", "engine", "start");

    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, harness.session()));

    const auto response = harness.readEnvelope();
    REQUIRE(response.domain == "engine");
    REQUIRE(response.kind == IpcKind::Event);
    REQUIRE(response.name == "state");
    REQUIRE(response.correlationId == env.id);
    REQUIRE(response.origin == IpcOrigin::Signal);
    REQUIRE(response.target == IpcTarget::Pulse);
}

TEST_CASE("DomainDispatcher accepts transport setLoopEnabled and emits state", "[domain-dispatcher][transport]") {
    auto engineHost = std::make_unique<EngineHost>();
    DomainDispatcher dispatcher(engineHost.get(), &engineHost->metering());
    SessionHarness harness;
    auto env = makeCommand("transport-cmd-1", "transport", "setLoopEnabled");
    env.payload = nlohmann::json{{"enabled", true}};

    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, harness.session()));

    const auto response = harness.readEnvelope();
    REQUIRE(response.domain == "transport");
    REQUIRE(response.kind == IpcKind::Event);
    REQUIRE(response.name == "state");
    REQUIRE(response.correlationId == env.id);
}

TEST_CASE("DomainDispatcher accepts hardware refreshOutputDevices and emits state plus control inventory", "[domain-dispatcher][hardware]") {
    auto engineHost = std::make_unique<EngineHost>();
    DomainDispatcher dispatcher(engineHost.get(), &engineHost->metering());
    SessionHarness harness;
    auto env = makeCommand("hardware-cmd-1", "hardware", "refreshOutputDevices");

    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, harness.session()));

    const auto hardwareState = harness.readEnvelope();
    REQUIRE(hardwareState.domain == "hardware");
    REQUIRE(hardwareState.kind == IpcKind::Event);
    REQUIRE(hardwareState.name == "state");
    REQUIRE(hardwareState.correlationId == env.id);

    const auto controlInventory = harness.readEnvelope();
    REQUIRE(controlInventory.domain == "control");
    REQUIRE(controlInventory.kind == IpcKind::Event);
    REQUIRE(controlInventory.name == "deviceInventory");
    REQUIRE(controlInventory.correlationId == env.id);
}

TEST_CASE("DomainDispatcher accepts plugin scanStatus and emits correlated status event", "[domain-dispatcher][plugin]") {
    auto engineHost = std::make_unique<EngineHost>();
    DomainDispatcher dispatcher(engineHost.get(), &engineHost->metering());
    SessionHarness harness;
    auto env = makeCommand("plugin-cmd-1", "plugin", "scanStatus");

    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, harness.session()));

    const auto response = harness.readEnvelope();
    REQUIRE(response.domain == "plugin");
    REQUIRE(response.kind == IpcKind::Event);
    REQUIRE(response.name == "scanStatus");
    REQUIRE(response.correlationId == env.id);
}

TEST_CASE("DomainDispatcher handles unknown domains gracefully", "[domain-dispatcher][unknown]") {
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

    SessionHarness harness;

    // Should not throw, just log warning
    REQUIRE_NOTHROW(dispatcher.handleEnvelope(env, harness.session()));
}
