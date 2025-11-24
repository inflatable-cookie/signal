#include "backend/MiniaudioBackend.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include <iostream>
#include <chrono>
#include <cmath>
#include <cstring>

MiniaudioBackend::MiniaudioBackend()
    : _running(false)
    , _shouldStop(false)
    , _hostTimeSeconds(0.0)
{
}

MiniaudioBackend::~MiniaudioBackend() {
    shutdown();
}

bool MiniaudioBackend::initialise(const AudioBackendConfig& config) {
    if (_running.load()) {
        std::cout << "[MiniaudioBackend] Cannot initialise: already running" << std::endl;
        return false;
    }

    _config = config;

    // Allocate buffers (interleaved format)
    int totalInputSamples = config.numInputChannels * config.preferredBufferSize;
    int totalOutputSamples = config.numOutputChannels * config.preferredBufferSize;

    _inputBuffer.resize(totalInputSamples, 0.0f);
    _outputBuffer.resize(totalOutputSamples, 0.0f);

    std::cout << "[MiniaudioBackend] Initialised: "
              << "sampleRate=" << config.preferredSampleRate
              << ", bufferSize=" << config.preferredBufferSize
              << ", inChannels=" << config.numInputChannels
              << ", outChannels=" << config.numOutputChannels
              << std::endl;

    return true;
}

void MiniaudioBackend::shutdown() {
    stop();

    _inputBuffer.clear();
    _outputBuffer.clear();
    _renderCallback = nullptr;

    std::cout << "[MiniaudioBackend] Shutdown complete" << std::endl;
}

bool MiniaudioBackend::start() {
    if (_running.load()) {
        std::cout << "[MiniaudioBackend] Already running" << std::endl;
        return false;
    }

    if (!_renderCallback) {
        std::cout << "[MiniaudioBackend] Cannot start: no render callback set" << std::endl;
        return false;
    }

    _shouldStop = false;
    _running = true;
    _hostTimeSeconds.store(0.0);
    _audioThread = std::thread(&MiniaudioBackend::audioLoop, this);

    std::cout << "[MiniaudioBackend] Started" << std::endl;
    return true;
}

void MiniaudioBackend::stop() {
    if (!_running.load()) {
        return;
    }

    _shouldStop = true;
    if (_audioThread.joinable()) {
        _audioThread.join();
    }
    _running = false;

    std::cout << "[MiniaudioBackend] Stopped" << std::endl;
}

void MiniaudioBackend::setRenderCallback(RenderCallback callback) {
    _renderCallback = std::move(callback);
}

double MiniaudioBackend::getSampleRate() const {
    return _config.preferredSampleRate;
}

int MiniaudioBackend::getBufferSize() const {
    return _config.preferredBufferSize;
}

int MiniaudioBackend::getNumInputChannels() const {
    return _config.numInputChannels;
}

int MiniaudioBackend::getNumOutputChannels() const {
    return _config.numOutputChannels;
}

void MiniaudioBackend::audioLoop() {
    // Simulate audio callback timing
    const double sampleRate = _config.preferredSampleRate;
    const int blockSize = _config.preferredBufferSize;
    const auto frameTime = std::chrono::nanoseconds(
        static_cast<long long>(blockSize / sampleRate * 1e9)
    );

    auto startTime = std::chrono::steady_clock::now();

    while (!_shouldStop.load()) {
        // Calculate host time (monotonic, in seconds)
        auto now = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(now - startTime);
        double hostTime = elapsed.count() / 1e9;
        _hostTimeSeconds.store(hostTime, std::memory_order_release);

        // Clear output buffer
        std::memset(_outputBuffer.data(), 0, _outputBuffer.size() * sizeof(float));

        // Create render context
        EngineRenderContext ctx;
        ctx.hostTimeSeconds = hostTime;
        ctx.sampleRate = sampleRate;
        ctx.blockSize = blockSize;
        ctx.playheadSamples = 0; // Will be updated by EngineHost

        // Wrap buffers in AudioBus objects
        AudioBus inputBus(
            _inputBuffer.data(),
            _config.numInputChannels,
            blockSize,
            true  // read-only
        );

        AudioBus outputBus(
            _outputBuffer.data(),
            _config.numOutputChannels,
            blockSize,
            false  // writable
        );

        // Call render callback
        if (_renderCallback) {
            _renderCallback(ctx, inputBus, outputBus);
        }

        // Simulate audio buffer timing
        std::this_thread::sleep_for(frameTime);
    }
}

