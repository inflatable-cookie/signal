#include "core/EngineHost.hpp"
#include <iostream>

EngineHost::EngineHost() : _state(State::Stopped) {
    std::cout << "[EngineHost] Created" << std::endl;
}

EngineHost::~EngineHost() {
    if (_state == State::Running) {
        stop();
    }
    std::cout << "[EngineHost] Destroyed" << std::endl;
}

void EngineHost::start() {
    if (_state == State::Running) {
        std::cout << "[EngineHost] Already running" << std::endl;
        return;
    }

    _state = State::Running;
    std::cout << "[EngineHost] Started" << std::endl;
}

void EngineHost::stop() {
    if (_state == State::Stopped) {
        std::cout << "[EngineHost] Already stopped" << std::endl;
        return;
    }

    _state = State::Stopped;
    std::cout << "[EngineHost] Stopped" << std::endl;
}

void EngineHost::reset() {
    stop();
    std::cout << "[EngineHost] Reset" << std::endl;
}

EngineHost::State EngineHost::state() const noexcept {
    return _state;
}

