#include "core/AudioThread.hpp"
#include "core/MeteringService.hpp"
#include "logging/Logging.hpp"
#include <cmath>
#include <cstring>
#include <thread>
#include <algorithm>
#include <chrono>

AudioThread::AudioThread()
    : _running(false)
    , _shouldStop(false)
    , _meteringService(nullptr)
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
    LOG_INFO({"AudioThread"}, "Started");
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
    LOG_INFO({"AudioThread"}, "Stopped");
}

bool AudioThread::isRunning() const noexcept {
    return _running.load();
}

void AudioThread::setCallback(AudioCallback callback) {
    _callback = std::move(callback);
}

void AudioThread::setMeteringService(MeteringService* meteringService) {
    _meteringService = meteringService;
}

void AudioThread::setActiveChannels(const std::vector<std::string>& channelIds) {
    _activeChannels = channelIds;
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

        // Update metering for active channels
        if (_meteringService) {
            updateMetering(buffer, BUFFER_SIZE, NUM_CHANNELS);
        }

        // Simulate audio buffer timing
        std::this_thread::sleep_for(frameTime);
    }
}

void AudioThread::updateMetering(float* buffer, size_t numFrames, int numChannels) {
    if (_activeChannels.empty() || !_meteringService) {
        return;
    }

    // Calculate peak and RMS for the buffer
    float peak = 0.0f;
    float sumSquares = 0.0f;
    size_t sampleCount = numFrames * numChannels;

    for (size_t i = 0; i < sampleCount; ++i) {
        float absValue = std::abs(buffer[i]);
        if (absValue > peak) {
            peak = absValue;
        }
        sumSquares += buffer[i] * buffer[i];
    }

    float rms = std::sqrt(sumSquares / static_cast<float>(sampleCount));

    // Get current timestamp
    auto now = std::chrono::steady_clock::now();
    auto duration = now.time_since_epoch();
    auto microseconds = std::chrono::duration_cast<std::chrono::microseconds>(duration).count();

    // Update metering for all active channels (for now, use same levels for all)
    // In a real implementation, each channel would have its own buffer
    for (const auto& channelId : _activeChannels) {
        auto* atomicMetering = _meteringService->getAtomicMetering(channelId);
        if (atomicMetering) {
            atomicMetering->update(peak, rms, static_cast<std::uint64_t>(microseconds));
        }
    }
}

