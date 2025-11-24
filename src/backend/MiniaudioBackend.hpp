#pragma once

/// MiniaudioBackend - Audio backend using miniaudio library
///
/// Thread: Main/control thread for lifecycle, audio thread for callbacks
/// Ownership: Owned by application or EngineHost wrapper
///
/// This is a placeholder implementation that simulates audio callbacks.
/// In the future, this will be replaced with actual miniaudio integration.

#include "backend/AudioBackend.hpp"
#include <atomic>
#include <thread>
#include <memory>
#include <vector>

class MiniaudioBackend : public AudioBackend {
public:
    MiniaudioBackend();
    ~MiniaudioBackend() override;

    bool initialise(const AudioBackendConfig& config) override;
    void shutdown() override;
    bool start() override;
    void stop() override;
    void setRenderCallback(RenderCallback callback) override;

    double getSampleRate() const override;
    int getBufferSize() const override;
    int getNumInputChannels() const override;
    int getNumOutputChannels() const override;

private:
    void audioLoop();

    AudioBackendConfig _config;
    RenderCallback _renderCallback;
    std::atomic<bool> _running;
    std::atomic<bool> _shouldStop;
    std::thread _audioThread;

    // Audio buffers (interleaved format)
    std::vector<float> _inputBuffer;
    std::vector<float> _outputBuffer;

    // Host time tracking (monotonic, in seconds)
    std::atomic<double> _hostTimeSeconds;
};

