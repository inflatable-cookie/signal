#pragma once

/// AudioBackend - Backend-agnostic interface for audio device I/O
///
/// Thread: Main/control thread for lifecycle, audio thread for callbacks
/// Ownership: Owned by application or EngineHost wrapper
///
/// Implementations: MiniaudioBackend, JackBackend, etc.

#include "backend/AudioBackendConfig.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include <functional>
#include <memory>

// Forward declarations
class AudioBackend;

/// Render callback type: (context, input, output) -> void
/// Called from audio thread - must be real-time safe
using RenderCallback = std::function<void(
    EngineRenderContext& ctx,
    AudioBus& input,
    AudioBus& output
)>;

class AudioBackend {
public:
    virtual ~AudioBackend() = default;

    /// Initialize the backend with configuration
    /// @param config Backend configuration
    /// @return true if initialization succeeded, false otherwise
    virtual bool initialise(const AudioBackendConfig& config) = 0;

    /// Shutdown and clean up resources
    virtual void shutdown() = 0;

    /// Start audio streaming (callbacks will begin)
    /// @return true if start succeeded, false otherwise
    virtual bool start() = 0;

    /// Stop audio streaming (callbacks will stop)
    virtual void stop() = 0;

    /// Set the render callback (must be called before start())
    /// @param callback Function to call on each audio block
    virtual void setRenderCallback(RenderCallback callback) = 0;

    /// Get current sample rate (after initialization)
    virtual double getSampleRate() const = 0;

    /// Get current buffer size (after initialization)
    virtual int getBufferSize() const = 0;

    /// Get number of input channels
    virtual int getNumInputChannels() const = 0;

    /// Get number of output channels
    virtual int getNumOutputChannels() const = 0;

    /// Get output device name (if available)
    /// @return Device name or empty string if not available
    virtual std::string getOutputDeviceName() const {
        return "System Default";
    }
};

