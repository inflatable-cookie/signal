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
#include <memory>
#include <optional>
#include <string>

class AudioThread;
class MeteringService;

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

private:
    State _state;
    std::optional<std::string> _lastError;
    std::unique_ptr<AudioThread> _audioThread;
    std::unique_ptr<MeteringService> _meteringService;
    TransportState _transportState;
    bool _shuttingDown;

    static constexpr double SAMPLE_RATE = 44100.0;
    static constexpr size_t BLOCK_SIZE = 512;
};

