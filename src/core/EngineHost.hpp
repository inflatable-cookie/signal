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
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include <atomic>
#include <memory>
#include <optional>
#include <string>
#include <cstdint>

class AudioThread;
class AudioBackend;
class MeteringService;
class MixerService;
class AutomationService;
class StreamScheduler;

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

    // Transport state access (control thread)
    // Returns mutable reference for updates - creates new snapshot internally
    TransportState& transport();
    const TransportState& transport() const;

    // Get current transport snapshot (for audio thread - lock-free)
    // Returns const pointer - caller must ensure it's not used after next swap
    // In practice, this is safe because renderBlock completes before next swap
    const TransportState* getTransportSnapshot() const;

    // Commit transport state updates (creates new snapshot and swaps atomically)
    // Must be called after modifying transport() to make changes visible to audio thread
    void commitTransportUpdate();

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

    StreamScheduler& streamScheduler();
    const StreamScheduler& streamScheduler() const;

    // Playhead management (for transport control)
    uint64_t getPlayheadSamples() const noexcept;
    void setPlayheadSamples(uint64_t samples) noexcept;

    // Audio thread entry point (called from AudioBackend)
    void renderBlock(
        EngineRenderContext& ctx,
        AudioBus& input,
        AudioBus& output
    );

private:
    State _state;
    std::optional<std::string> _lastError;
    std::unique_ptr<AudioThread> _audioThread;  // Legacy - will be removed in future
    std::unique_ptr<AudioBackend> _audioBackend;
    std::unique_ptr<MeteringService> _meteringService;
    std::unique_ptr<MixerService> _mixerService;
    std::unique_ptr<AutomationService> _automationService;
    std::unique_ptr<StreamScheduler> _streamScheduler;

    // Transport state (thread-safe snapshot swap)
    // Control thread: updates via transport() which creates new snapshot and swaps atomically
    // Audio thread: reads via getTransportSnapshot() which returns const pointer (lock-free)
    // Using raw pointer with shared_ptr for lifetime management
    std::atomic<const TransportState*> _activeTransport;
    std::shared_ptr<TransportState> _transportState;  // Current mutable state (control thread only)
    std::shared_ptr<TransportState> _previousTransport;  // Keep previous snapshot alive until next swap

    bool _shuttingDown;

    static constexpr double SAMPLE_RATE = 44100.0;
    static constexpr size_t BLOCK_SIZE = 512;

    // Playhead tracking (for audio thread)
    std::atomic<uint64_t> _playheadSamples;

    void setupAudioCallback();
    void audioCallback(float* buffer, size_t numFrames, int numChannels);  // Legacy
    void setupAudioBackend();
};

