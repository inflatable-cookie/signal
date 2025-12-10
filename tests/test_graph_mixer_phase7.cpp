#include <catch2/catch_test_macros.hpp>
#include "core/EngineHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphNodes.hpp"
#include <nlohmann/json.hpp>

TEST_CASE("Phase 7 - MixerChannelNode initial state from graph snapshot", "[graph][phase7][mixer]") {
    using nlohmann::json;

    // Build a minimal graph snapshot JSON with a mixer-channel node that has mixer metadata.
    json j = {
        {"id", "test-graph"},
        {"nodes", json::array({
             json{
                 {"nodeId", "mixer-track-1"},
                 {"kind", "mixer-channel"},
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

    auto* node = host.graphEngine().findNode("mixer-track-1");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::MixerChannel);

    auto* mixer = dynamic_cast<MixerChannelNode*>(node);
    REQUIRE(mixer != nullptr);

    // MixerChannelNode should reflect gain and pan from the snapshot metadata.
    REQUIRE(mixer->getGain() == Approx(0.5f));
    REQUIRE(mixer->getPan() == Approx(-0.25f));
}

TEST_CASE("Phase 7 - MixerChannelNode respects muted flag from graph snapshot", "[graph][phase7][mixer]") {
    using nlohmann::json;

    // Build a minimal graph snapshot JSON with a muted mixer-channel node.
    json j = {
        {"id", "test-graph"},
        {"nodes", json::array({
             json{
                 {"nodeId", "mixer-track-2"},
                 {"kind", "mixer-channel"},
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

    auto* node = host.graphEngine().findNode("mixer-track-2");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::MixerChannel);

    auto* mixer = dynamic_cast<MixerChannelNode*>(node);
    REQUIRE(mixer != nullptr);

    // Muted flag should force gain to 0.0 at load time, regardless of the gain value.
    REQUIRE(mixer->getGain() == Approx(0.0f));
}

