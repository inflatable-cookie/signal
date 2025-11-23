#pragma once

/// AudioThread - Real-time audio processing thread
///
/// Thread: Dedicated high-priority audio thread
/// Ownership: Owned by EngineHost
/// Communication:
///   - Started/stopped by EngineHost (main thread)
///   - Reads state via atomic flags (_shouldStop, _running)
///   - Must NEVER: block, allocate, perform I/O, or hold locks
///   - All communication with other threads must be lock-free

#include <atomic>
#include <functional>
#include <thread>

/// Minimal audio thread for Signal skeleton
class AudioThread {
public:
    using AudioCallback = std::function<void(float* buffer, size_t numFrames, int numChannels)>;

    AudioThread();
    ~AudioThread();

    void start();
    void stop();
    bool isRunning() const noexcept;

    void setCallback(AudioCallback callback);

private:
    void audioLoop();

    std::atomic<bool> _running;
    std::atomic<bool> _shouldStop;
    std::thread _thread;
    AudioCallback _callback;
    static constexpr size_t BUFFER_SIZE = 256;
    static constexpr int NUM_CHANNELS = 2;
    static constexpr double SAMPLE_RATE = 44100.0;
};

