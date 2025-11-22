#include <catch2/catch_test_macros.hpp>
#include "ipc/Router.hpp"
#include "ipc/Envelope.hpp"
#include <memory>

class TestHandler : public DomainHandler {
public:
    int callCount = 0;
    std::string lastDomain;
    std::string lastName;

    void handle(const Envelope& env) override {
        callCount++;
        lastDomain = env.domain;
        lastName = env.name;
    }
};

TEST_CASE("Router registers and dispatches to handlers", "[router]") {
    IpcRouter router;
    auto handler = std::make_shared<TestHandler>();

    router.registerHandler("engine", handler);

    Envelope env;
    env.domain = "engine";
    env.name = "start";
    env.kind = "command";

    router.dispatch(env);

    REQUIRE(handler->callCount == 1);
    REQUIRE(handler->lastDomain == "engine");
    REQUIRE(handler->lastName == "start");
}

TEST_CASE("Router ignores unknown domains", "[router]") {
    IpcRouter router;
    auto handler = std::make_shared<TestHandler>();

    router.registerHandler("engine", handler);

    Envelope env;
    env.domain = "unknown";
    env.name = "test";
    env.kind = "command";

    router.dispatch(env);

    REQUIRE(handler->callCount == 0);
}

TEST_CASE("Router supports multiple handlers per domain", "[router]") {
    IpcRouter router;
    auto handler1 = std::make_shared<TestHandler>();
    auto handler2 = std::make_shared<TestHandler>();

    router.registerHandler("engine", handler1);
    router.registerHandler("engine", handler2);

    Envelope env;
    env.domain = "engine";
    env.name = "stop";
    env.kind = "command";

    router.dispatch(env);

    REQUIRE(handler1->callCount == 1);
    REQUIRE(handler2->callCount == 1);
    REQUIRE(handler1->lastName == "stop");
    REQUIRE(handler2->lastName == "stop");
}

