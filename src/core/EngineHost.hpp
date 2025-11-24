#pragma once

/// EngineHost - Manages engine lifecycle and audio thread
///
/// Thread: Main thread (owned by SignalApp)
/// Ownership: Owned by SignalApp
/// Communication:
///   - Updated by EngineDomain handlers (IPC thread)
///   - Controls AudioThread lifecycle
///   - State readable by audio thread via state() method (lock-free)

#include "core/TransportState.hpp"
#include <atomic>
#include <memory>
#include <optional>
#include <string>
#include <cstdint>

class AudioThread;
class MeteringService;
class MixerService;
class AutomationService;
class ClipScheduler;

class EngineHost {
public:
    enum class State {
        Stopped,
        Starting,
        Running,
        Error
    };

    EngineHost();
    ~EngineHost();

    void start();
    void stop();
    void reset();
    void shutdown();

    State state() const noexcept;
    std::optional<std::string> lastError() const noexcept;
    void setError(const std::string& error);
    void clearError();

    TransportState& transport();
    const TransportState& transport() const;

    // Diagnostic information
    double getCpuLoad() const; // Stub for now
    uint64_t getXruns() const; // Stub for now
    double getSampleRate() const;
    size_t getBlockSize() const;

    // Metering
    MeteringService& metering();
    const MeteringService& metering() const;

    // Mixer
    MixerService& mixer();
    const MixerService& mixer() const;

    AutomationService& automation();
    const AutomationService& automation() const;

    ClipScheduler& clipScheduler();
    const ClipScheduler& clipScheduler() const;

    // Playhead management (for transport control)
    uint64_t getPlayheadSamples() const noexcept;
    void setPlayheadSamples(uint64_t samples) noexcept;

private:
    State _state;
    std::optional<std::string> _lastError;
    std::unique_ptr<AudioThread> _audioThread;
    std::unique_ptr<MeteringService> _meteringService;
    std::unique_ptr<MixerService> _mixerService;
    std::unique_ptr<AutomationService> _automationService;
    std::unique_ptr<ClipScheduler> _clipScheduler;
    TransportState _transportState;
    bool _shuttingDown;

    static constexpr double SAMPLE_RATE = 44100.0;
    static constexpr size_t BLOCK_SIZE = 512;

    // Playhead tracking (for audio thread)
    std::atomic<uint64_t> _playheadSamples;

    void setupAudioCallback();
    void audioCallback(float* buffer, size_t numFrames, int numChannels);
};

