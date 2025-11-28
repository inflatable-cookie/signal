#pragma once

/// EngineHost - Manages engine lifecycle and audio backend
///
/// Thread: Main thread (owned by SignalApp)
/// Ownership: Owned by SignalApp
/// Communication:
///   - Updated by EngineDomain handlers (IPC thread)
///   - Controls AudioBackend lifecycle
///   - State readable by audio thread via state() method (lock-free)

#include "core/TransportState.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/ParameterChange.hpp"
#include "core/AutomationData.hpp"
#include "backend/OutputDeviceInfo.hpp"
#include <atomic>
#include <memory>
#include <optional>
#include <string>
#include <cstdint>
#include <vector>
#include <vector>

class AudioBackend;
class MeteringService;
class MixerService;
class AutomationService;
class StreamScheduler;
class GraphEngine;
class AudioAssetSource;
class PluginHost;
class RecordingSession;

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

    // Get current automation snapshot (for audio thread - lock-free)
    // Returns const pointer - caller must ensure it's not used after next swap
    const AutomationData* getAutomationSnapshot() const;

    // Recording session access
    RecordingSession& recordingSession();
    const RecordingSession& recordingSession() const;

    // Commit transport state updates (creates new snapshot and swaps atomically)
    // Must be called after modifying transport() to make changes visible to audio thread
    void commitTransportUpdate();

    // Diagnostic information
    double getCpuLoad() const; // Stub for now
    uint64_t getXruns() const; // Stub for now
    double getSampleRate() const;
    size_t getBlockSize() const;
    std::string getOutputDeviceName() const;
    int getNumOutputChannels() const;
    std::string getActiveOutputDeviceId() const;
    std::vector<OutputDeviceInfo> enumerateOutputDevices() const;
    bool setOutputDevice(const std::string& deviceId);

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

    // Plugin host
    PluginHost* pluginHost();
    const PluginHost* pluginHost() const;

    // Graph engine
    GraphEngine& graphEngine();
    const GraphEngine& graphEngine() const;

    // Load graph snapshot (called from IPC thread)
    void loadGraphSnapshot(const GraphSnapshot& snapshot);

    // Parameter changes (called from IPC thread)
    void applyParameterChanges(const std::vector<ParameterChange>& changes);

    // Load automation snapshot (called from IPC thread)
    void loadAutomationSnapshot(const AutomationData& snapshot);

    // Prepare engine (called on control thread)
    void prepareEngine(int sampleRate, int maxBlockSize);

    // Asset source management (called from IPC thread)
    void prepareAudioAsset(const std::string& assetId, const std::string& path, uint32_t channels, uint32_t sampleRate, uint64_t frames);

    // Playhead management (for transport control)
    uint64_t getPlayheadSamples() const noexcept;
    void setPlayheadSamples(uint64_t samples) noexcept;

    // Graph latency and tail (read-only, safe for audio thread)
    // These values are updated when the graph snapshot is loaded/updated
    int getGraphLatencySamples() const noexcept { return _graphLatencySamples.load(std::memory_order_acquire); }
    int getGraphTailSamples() const noexcept { return _graphTailSamples.load(std::memory_order_acquire); }

    // Audio thread entry point (called from AudioBackend)
    void renderBlock(
        EngineRenderContext& ctx,
        AudioBus& input,
        AudioBus& output
    );

private:
    State _state;
    std::optional<std::string> _lastError;
    std::unique_ptr<AudioBackend> _audioBackend;
    std::unique_ptr<MeteringService> _meteringService;
    std::unique_ptr<MixerService> _mixerService;
    std::unique_ptr<AutomationService> _automationService;
    std::unique_ptr<StreamScheduler> _streamScheduler;
    std::unique_ptr<GraphEngine> _graphEngine;
    std::unique_ptr<AudioAssetSource> _audioAssetSource; // Phase 3: Asset source for audio streaming
    std::unique_ptr<PluginHost> _pluginHost; // Phase 4: Plugin host for creating plugin instances
    std::unique_ptr<RecordingSession> _recordingSession; // Phase 7: Recording capture session

    // Parameter change queue (lock-free, double-buffered for real-time safety)
    // Control thread: writes to _pendingParameterChanges
    // Audio thread: reads from _activeParameterChanges at block start
    std::vector<ParameterChange> _pendingParameterChanges;
    std::vector<ParameterChange> _activeParameterChanges;
    std::atomic<bool> _parameterChangesPending;

    // Transport state (thread-safe snapshot swap)
    // Control thread: updates via transport() which creates new snapshot and swaps atomically
    // Audio thread: reads via getTransportSnapshot() which returns const pointer (lock-free)
    // Using raw pointer with shared_ptr for lifetime management
    std::atomic<const TransportState*> _activeTransport;
    std::shared_ptr<TransportState> _transportState;  // Current mutable state (control thread only)
    std::shared_ptr<TransportState> _previousTransport;  // Keep previous snapshot alive until next swap

    // Legacy automation data (deprecated - kept for backward compatibility during transition)
    // Automation is now handled by AutomationService exclusively
    std::atomic<const AutomationData*> _activeAutomation;
    std::shared_ptr<AutomationData> _automationData;
    std::shared_ptr<AutomationData> _previousAutomation;

    bool _shuttingDown;

    static constexpr double SAMPLE_RATE = 44100.0;
    static constexpr size_t BLOCK_SIZE = 512;

    // Playhead tracking (for audio thread)
    std::atomic<uint64_t> _playheadSamples;

    // Graph latency and tail (updated on control thread, read on audio thread)
    // These are computed from the graph when it's loaded/updated
    std::atomic<int> _graphLatencySamples;
    std::atomic<int> _graphTailSamples;

#ifdef LOOPHOLE_ENABLE_AUDIO_DEBUG
    // Diagnostic counters for audio thread debugging
    std::atomic<uint64_t> _renderBlockCount;
    std::atomic<uint64_t> _lastDebugLogBlock;
    std::atomic<uint32_t> _consecutiveSilenceBlocks;
    static constexpr uint32_t DEBUG_LOG_INTERVAL_BLOCKS = 86; // Approx 1 second at 44.1kHz/512 block size
#endif

    void setupAudioBackend();
};

