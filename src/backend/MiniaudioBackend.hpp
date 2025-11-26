#pragma once

/// MiniaudioBackend - Audio backend using miniaudio library
///
/// Thread: Main/control thread for lifecycle, audio thread for callbacks
/// Ownership: Owned by application or EngineHost wrapper
///
/// Real-time safe: The audio callback runs on a high-priority thread and must not
/// allocate memory, acquire locks, or perform I/O operations.

#include "backend/AudioBackend.hpp"
#include <atomic>
#include <string>
#include <memory>
#include <cstdint>

// Forward declaration - miniaudio types (opaque pointers)
struct ma_context;
struct ma_device;

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

    /// Get the name of the output device (or "System Default" if not available)
    std::string getOutputDeviceName() const override;

private:
    // Static callback function for miniaudio (C-compatible)
    // Note: Implementation in .cpp file where miniaudio.h is included
    static void audioCallback(
        void* pDevice,
        void* pOutput,
        const void* pInput,
        unsigned int frameCount
    );

    // Instance callback wrapper (called from static callback)
    void processAudio(
        float* output,
        const float* input,
        unsigned int frameCount
    );

    AudioBackendConfig _config;
    RenderCallback _renderCallback;
    std::atomic<bool> _initialised;
    std::atomic<bool> _running;

    // Miniaudio objects (opaque pointers)
    void* _context;  // ma_context*
    void* _device;   // ma_device*

    // Actual runtime values (may differ from config preferences)
    std::atomic<double> _actualSampleRate;
    std::atomic<uint32_t> _actualBufferSize;
    std::atomic<uint32_t> _actualOutputChannels;
    std::string _outputDeviceName;

    // Host time tracking (monotonic, in seconds)
    std::atomic<double> _hostTimeSeconds;
};

