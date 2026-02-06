#include <catch2/catch_test_macros.hpp>
#include "ipc/BinaryEnvelopeCodec.hpp"
#include "ipc/IpcEnvelope.hpp"

TEST_CASE("BinaryEnvelopeCodec encodes and decodes engine.state", "[ipc][binary-envelope-v2]") {
    using namespace loophole::signal::ipc;

    IpcEnvelope env;
    env.version = 1;
    env.id = "test-id-123";
    env.correlationId = std::nullopt;
    env.timestamp = "2025-11-22T12:00:00.000Z";
    env.origin = IpcOrigin::Pulse;
    env.target = IpcTarget::Signal;
    env.domain = "engine";
    env.kind = IpcKind::Event;
    env.name = "state";
    env.priority = IpcPriority::High;
    env.payload = nlohmann::json::object();
    env.error = std::nullopt;

    std::string err;
    auto bytes = tryEncodeBinaryEnvelopeV2(env, err);
    REQUIRE(bytes.has_value());
    REQUIRE(err.empty());

    std::string decodeErr;
    auto decoded = decodeBinaryEnvelopeV2(std::span<const std::uint8_t>(bytes->data(), bytes->size()), decodeErr);
    REQUIRE(decoded.has_value());
    REQUIRE(decodeErr.empty());

    REQUIRE(decoded->id == env.id);
    REQUIRE(decoded->correlationId == env.correlationId);
    REQUIRE(decoded->timestamp == env.timestamp);
    REQUIRE(decoded->origin == env.origin);
    REQUIRE(decoded->target == env.target);
    REQUIRE(decoded->domain == env.domain);
    REQUIRE(decoded->kind == env.kind);
    REQUIRE(decoded->name == env.name);
    REQUIRE(decoded->priority == env.priority);
}

TEST_CASE("BinaryEnvelopeCodec encodes parameter.valuesSnapshot", "[ipc][binary-envelope-v2]") {
    using namespace loophole::signal::ipc;

    IpcEnvelope env;
    env.version = 1;
    env.id = "test-parameter-values-1";
    env.correlationId = "test-command-1";
    env.timestamp = "2025-11-22T12:00:00.000Z";
    env.origin = IpcOrigin::Signal;
    env.target = IpcTarget::Pulse;
    env.domain = "parameter";
    env.kind = IpcKind::Event;
    env.name = "valuesSnapshot";
    env.priority = IpcPriority::Normal;
    env.payload = {
        {"scope", {{"nodeId", "node-1"}}},
        {"values", {{"gain", 0.25}, {"bypass", 1.0}}}
    };
    env.error = std::nullopt;

    std::string err;
    auto bytes = tryEncodeBinaryEnvelopeV2(env, err);
    REQUIRE(bytes.has_value());
    REQUIRE(err.empty());

    std::string decodeErr;
    auto decoded = decodeBinaryEnvelopeV2(std::span<const std::uint8_t>(bytes->data(), bytes->size()), decodeErr);
    REQUIRE(decoded.has_value());
    REQUIRE(decodeErr.empty());
    REQUIRE(decoded->domain == env.domain);
    REQUIRE(decoded->kind == env.kind);
    REQUIRE(decoded->name == env.name);
}

TEST_CASE("BinaryEnvelopeCodec encodes parameter.valueChanged", "[ipc][binary-envelope-v2]") {
    using namespace loophole::signal::ipc;

    IpcEnvelope env;
    env.version = 1;
    env.id = "test-parameter-changed-1";
    env.correlationId = "test-command-2";
    env.timestamp = "2025-11-22T12:00:00.000Z";
    env.origin = IpcOrigin::Signal;
    env.target = IpcTarget::Pulse;
    env.domain = "parameter";
    env.kind = IpcKind::Event;
    env.name = "valueChanged";
    env.priority = IpcPriority::Normal;
    env.payload = {
        {"scope", {{"nodeId", "node-1"}, {"pluginInstanceId", "plugin-1"}}},
        {"paramId", "gain"},
        {"value", 0.5}
    };
    env.error = std::nullopt;

    std::string err;
    auto bytes = tryEncodeBinaryEnvelopeV2(env, err);
    REQUIRE(bytes.has_value());
    REQUIRE(err.empty());

    std::string decodeErr;
    auto decoded = decodeBinaryEnvelopeV2(std::span<const std::uint8_t>(bytes->data(), bytes->size()), decodeErr);
    REQUIRE(decoded.has_value());
    REQUIRE(decodeErr.empty());
    REQUIRE(decoded->domain == env.domain);
    REQUIRE(decoded->kind == env.kind);
    REQUIRE(decoded->name == env.name);
}
