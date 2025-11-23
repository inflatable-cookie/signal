#include "core/AudioThread.hpp"
#include <cmath>
#include <cstring>
#include <iostream>
#include <thread>

AudioThread::AudioThread()
    : _running(false)
    , _shouldStop(false)
{
}

AudioThread::~AudioThread() {
    stop();
}

void AudioThread::start() {
    if (_running.load()) {
        return;
    }

    _shouldStop = false;
    _running = true;
    _thread = std::thread(&AudioThread::audioLoop, this);
    std::cout << "[AudioThread] Started" << std::endl;
}

void AudioThread::stop() {
    if (!_running.load()) {
        return;
    }

    _shouldStop = true;
    if (_thread.joinable()) {
        _thread.join();
    }
    _running = false;
    std::cout << "[AudioThread] Stopped" << std::endl;
}

bool AudioThread::isRunning() const noexcept {
    return _running.load();
}

void AudioThread::setCallback(AudioCallback callback) {
    _callback = std::move(callback);
}

void AudioThread::audioLoop() {
    // For now, just simulate audio processing by sleeping
    // In a real implementation, this would be driven by audio hardware callbacks
    float buffer[BUFFER_SIZE * NUM_CHANNELS];
    const auto frameTime = std::chrono::nanoseconds(
        static_cast<long long>(BUFFER_SIZE / SAMPLE_RATE * 1e9)
    );

    while (!_shouldStop.load()) {
        if (_callback) {
            _callback(buffer, BUFFER_SIZE, NUM_CHANNELS);
        } else {
            // Generate silence
            std::memset(buffer, 0, sizeof(buffer));
        }

        // Simulate audio buffer timing
        std::this_thread::sleep_for(frameTime);
    }
}

