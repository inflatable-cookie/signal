#include <catch2/catch_test_macros.hpp>
#include "core/EngineHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphNodes.hpp"
#include <nlohmann/json.hpp>

TEST_CASE("Phase 7 - FaderNode initial state from graph snapshot", "[graph][phase7][mixer]") {
    using nlohmann::json;

    // Build a minimal graph snapshot JSON with a fader node that has mixer metadata.
    json j = {
        {"id", "test-graph"},
        {"nodes", json::array({
             json{
                 {"nodeId", "fader-track-1"},
                 {"kind", "fader"},
                 {"trackId", "track-1"},
                 {"mixer", {
                     {"gain", 0.5f},
                     {"pan", -0.25f},
                     {"muted", false},
                     {"soloed", false}
                 }}
             }
         })},
        {"connections", json::array()}
    };

    auto snapshotOpt = GraphSnapshot::fromJson(j);
    REQUIRE(snapshotOpt.has_value());

    EngineHost host;
    host.prepareEngine(44100, 512);

    // Load the parsed snapshot into the graph engine.
    host.loadGraphSnapshot(snapshotOpt.value());

    auto* node = host.graphEngine().findNode("fader-track-1");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::Fader);

    auto* faderNode = dynamic_cast<FaderNode*>(node);
    REQUIRE(faderNode != nullptr);

    // FaderNode should reflect gain and pan from the snapshot metadata.
    REQUIRE(faderNode->getGain() == Approx(0.5f));
    REQUIRE(faderNode->getPan() == Approx(-0.25f));
}

TEST_CASE("Phase 7 - FaderNode ignores muted flag from graph snapshot (mute via node parameter)", "[graph][phase7][mixer]") {
    using nlohmann::json;

    // Build a minimal graph snapshot JSON with a muted fader node.
    json j = {
        {"id", "test-graph"},
        {"nodes", json::array({
             json{
                 {"nodeId", "fader-track-2"},
                 {"kind", "fader"},
                 {"trackId", "track-2"},
                 {"mixer", {
                     {"gain", 0.8f},
                     {"pan", 0.0f},
                     {"muted", true},
                     {"soloed", false}
                 }}
             }
         })},
        {"connections", json::array()}
    };

    auto snapshotOpt = GraphSnapshot::fromJson(j);
    REQUIRE(snapshotOpt.has_value());

    EngineHost host;
    host.prepareEngine(44100, 512);
    host.loadGraphSnapshot(snapshotOpt.value());

    auto* node = host.graphEngine().findNode("fader-track-2");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::Fader);

    auto* faderNode = dynamic_cast<FaderNode*>(node);
    REQUIRE(faderNode != nullptr);

    // Muted is now projected to nodes via `node.setParameter` (`muted`), so the
    // graph snapshot mute flag must not affect gain at load time.
    REQUIRE(faderNode->getGain() == Approx(0.8f));
}
