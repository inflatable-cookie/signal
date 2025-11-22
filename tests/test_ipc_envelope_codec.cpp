#include <catch2/catch_test_macros.hpp>
#include "ipc/IpcEnvelope.hpp"
#include "ipc/IpcEnvelopeCodec.hpp"

TEST_CASE("IpcEnvelope serialisation and deserialisation", "[ipc][codec]") {
    using namespace loophole::signal::ipc;

    // Create a test envelope
    IpcEnvelope env;
    env.version = 1;
    env.id = "test-id-123";
    env.correlationId = std::nullopt;
    env.timestamp = "2025-11-22T12:00:00.000Z";
    env.origin = IpcOrigin::Pulse;
    env.target = IpcTarget::Signal;
    env.domain = "engine";
    env.kind = IpcKind::Command;
    env.name = "start";
    env.priority = IpcPriority::High;
    env.payload = nlohmann::json::object();
    env.error = std::nullopt;

    // Serialise
    std::string json = serialiseEnvelope(env);
    REQUIRE(!json.empty());

    // Deserialise
    auto env_opt = deserialiseEnvelope(json);
    REQUIRE(env_opt.has_value());

    IpcEnvelope& deserialised = env_opt.value();
    REQUIRE(deserialised.version == env.version);
    REQUIRE(deserialised.id == env.id);
    REQUIRE(deserialised.origin == env.origin);
    REQUIRE(deserialised.target == env.target);
    REQUIRE(deserialised.domain == env.domain);
    REQUIRE(deserialised.kind == env.kind);
    REQUIRE(deserialised.name == env.name);
    REQUIRE(deserialised.priority == env.priority);
}

TEST_CASE("IpcEnvelope with correlation ID", "[ipc][codec]") {
    using namespace loophole::signal::ipc;

    IpcEnvelope env;
    env.version = 1;
    env.id = "reply-456";
    env.correlationId = "command-123";
    env.timestamp = "2025-11-22T12:00:01.000Z";
    env.origin = IpcOrigin::Signal;
    env.target = IpcTarget::Pulse;
    env.domain = "transport";
    env.kind = IpcKind::Event;
    env.name = "state";
    env.priority = IpcPriority::Normal;
    env.payload = nlohmann::json::object();

    std::string json = serialiseEnvelope(env);
    auto env_opt = deserialiseEnvelope(json);
    REQUIRE(env_opt.has_value());

    REQUIRE(env_opt->correlationId.has_value());
    REQUIRE(env_opt->correlationId.value() == "command-123");
}

TEST_CASE("IpcEnvelope with error", "[ipc][codec]") {
    using namespace loophole::signal::ipc;

    IpcEnvelope env;
    env.version = 1;
    env.id = "error-789";
    env.correlationId = "command-456";
    env.timestamp = "2025-11-22T12:00:02.000Z";
    env.origin = IpcOrigin::Signal;
    env.target = IpcTarget::Pulse;
    env.domain = "unknown";
    env.kind = IpcKind::Error;
    env.name = "failed";
    env.priority = IpcPriority::High;
    env.payload = nlohmann::json::object();

    IpcErrorInfo error_info;
    error_info.code = "invalid_domain";
    error_info.message = "Unknown domain";
    error_info.details = nlohmann::json::object();
    env.error = std::make_optional(error_info);

    std::string json = serialiseEnvelope(env);
    auto env_opt = deserialiseEnvelope(json);
    REQUIRE(env_opt.has_value());

    REQUIRE(env_opt->error.has_value());
    REQUIRE(env_opt->error->code == "invalid_domain");
    REQUIRE(env_opt->error->message == "Unknown domain");
}

TEST_CASE("IpcEnvelope deserialise invalid JSON", "[ipc][codec]") {
    using namespace loophole::signal::ipc;

    // Invalid JSON
    auto env_opt = deserialiseEnvelope("{ invalid json }");
    REQUIRE(!env_opt.has_value());

    // Missing required fields
    env_opt = deserialiseEnvelope("{}");
    REQUIRE(!env_opt.has_value());

    // Invalid enum values
    std::string invalid_origin = R"({"v":1,"id":"test","ts":"2025-11-22T12:00:00.000Z","origin":"invalid","target":"signal","domain":"test","kind":"command","name":"test","priority":"normal","payload":{}})";
    env_opt = deserialiseEnvelope(invalid_origin);
    REQUIRE(!env_opt.has_value());
}

