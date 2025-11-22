#include <catch2/catch_test_macros.hpp>
#include "ipc/Envelope.hpp"

TEST_CASE("Envelope creation", "[envelope]") {
    auto env = makeBasicEnvelope("engine", "command", "start");

    REQUIRE(env.v == 1);
    REQUIRE(env.domain == "engine");
    REQUIRE(env.kind == "command");
    REQUIRE(env.name == "start");
    REQUIRE(env.priority == "normal");
}

TEST_CASE("Envelope fields can be modified", "[envelope]") {
    auto env = makeBasicEnvelope("transport", "event", "state");

    env.id = "test-id-123";
    env.cid = "correlation-id-456";
    env.origin = "pulse";
    env.target = "signal";

    REQUIRE(env.id == "test-id-123");
    REQUIRE(env.cid == "correlation-id-456");
    REQUIRE(env.origin == "pulse");
    REQUIRE(env.target == "signal");
}

