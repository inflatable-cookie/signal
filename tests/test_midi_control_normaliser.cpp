#include <catch2/catch_test_macros.hpp>
#include "domains/control/MidiControlNormaliser.hpp"

TEST_CASE("MidiControlNormaliser normalises realtime and note messages", "[control][midi]") {
    using namespace loophole::signal::control;

    auto realtime = normaliseMidiMessage(0xfa, 0, 0);
    REQUIRE(realtime.has_value());
    REQUIRE(realtime->control_key == "midi:rt:fa");
    REQUIRE(realtime->action == "press");

    auto note_on = normaliseMidiMessage(0x90, 60, 100);
    REQUIRE(note_on.has_value());
    REQUIRE(note_on->control_key == "midi:note-on:60:1");
    REQUIRE(note_on->action == "press");
    REQUIRE(note_on->value == Approx(100.0));

    auto note_off = normaliseMidiMessage(0x90, 60, 0);
    REQUIRE(note_off.has_value());
    REQUIRE(note_off->control_key == "midi:note-off:60:1");
    REQUIRE(note_off->action == "release");
    REQUIRE_FALSE(note_off->value.has_value());
}

TEST_CASE("MidiControlNormaliser normalises CC and pitch messages", "[control][midi]") {
    using namespace loophole::signal::control;

    auto cc = normaliseMidiMessage(0xb2, 74, 127);
    REQUIRE(cc.has_value());
    REQUIRE(cc->control_key == "midi:cc:74:3");
    REQUIRE(cc->action == "change");
    REQUIRE(cc->value == Approx(127.0));

    auto pitch = normaliseMidiMessage(0xe0, 0x00, 0x40);
    REQUIRE(pitch.has_value());
    REQUIRE(pitch->control_key == "midi:pitch:8192:1");
    REQUIRE(pitch->action == "change");
    REQUIRE(pitch->value == Approx(8192.0));
}
