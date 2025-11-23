#include "core/EngineHost.hpp"
#include "core/AudioThread.hpp"
#include <iostream>
#include <memory>
#include <cstdint>

EngineHost::EngineHost()
    : _state(State::Stopped)
    , _lastError(std::nullopt)
    , _shuttingDown(false)
{
    _audioThread = std::make_unique<AudioThread>();
    std::cout << "[EngineHost] Created" << std::endl;
}

EngineHost::~EngineHost() {
    if (_state == State::Running || _state == State::Starting) {
        stop();
    }
    std::cout << "[EngineHost] Destroyed" << std::endl;
}

void EngineHost::start() {
    if (_shuttingDown) {
        std::cout << "[EngineHost] Cannot start: shutting down" << std::endl;
        return;
    }

    if (_state == State::Running) {
        std::cout << "[EngineHost] Already running" << std::endl;
        return;
    }

    if (_state == State::Error) {
        std::cout << "[EngineHost] Cannot start: in error state" << std::endl;
        return;
    }

    _state = State::Starting;
    clearError();

    _audioThread->start();

    // After audio thread starts successfully, transition to running
    _state = State::Running;
    std::cout << "[EngineHost] Started" << std::endl;
}

void EngineHost::stop() {
    if (_state == State::Stopped) {
        std::cout << "[EngineHost] Already stopped" << std::endl;
        return;
    }

    _state = State::Stopped;
    _audioThread->stop();
    std::cout << "[EngineHost] Stopped" << std::endl;
}

void EngineHost::reset() {
    stop();
    clearError();
    _transportState = TransportState();
    std::cout << "[EngineHost] Reset" << std::endl;
}

void EngineHost::shutdown() {
    if (_shuttingDown) {
        return;
    }

    _shuttingDown = true;
    stop();
    std::cout << "[EngineHost] Shutdown complete" << std::endl;
}

EngineHost::State EngineHost::state() const noexcept {
    return _state;
}

std::optional<std::string> EngineHost::lastError() const noexcept {
    return _lastError;
}

void EngineHost::setError(const std::string& error) {
    _state = State::Error;
    _lastError = error;
    std::cout << "[EngineHost] Error: " << error << std::endl;
}

void EngineHost::clearError() {
    if (_state == State::Error) {
        _state = State::Stopped;
    }
    _lastError = std::nullopt;
}

TransportState& EngineHost::transport() {
    return _transportState;
}

const TransportState& EngineHost::transport() const {
    return _transportState;
}

double EngineHost::getCpuLoad() const {
    // Stub implementation - return 0.0 for now
    return 0.0;
}

uint64_t EngineHost::getXruns() const {
    // Stub implementation - return 0 for now
    return 0;
}

double EngineHost::getSampleRate() const {
    return SAMPLE_RATE;
}

size_t EngineHost::getBlockSize() const {
    return BLOCK_SIZE;
}

