#pragma once

/// RecordingCapture - Capture buffer and recording session management
///
/// Thread: Audio thread (capture), Control thread (session management)
/// Ownership: Owned by EngineHost
///
/// Phase 7: Basic recording capture from input nodes into buffers
/// Real-time safe: Uses lock-free ring buffer for audio thread writes

#include "core/GraphNode.hpp"
#include "core/ScheduleData.hpp"
#include <string>
#include <vector>
#include <cstdint>
#include <atomic>
#include <memory>
#include <unordered_map>
#include <unordered_set>
#include <shared_mutex>
#include <utility>

// Forward declaration
class AudioBus;

/// Recorded audio chunk (interleaved format for I/O simplicity)
struct RecordedAudioChunk {
    std::string provisionalAssetId; // Temporary GUID until Pulse assigns AssetId
    std::vector<float> interleaved; // Interleaved audio data
    int numChannels;
    int sampleRate;
    uint64_t startSample; // Absolute sample position in session
    std::string laneId;   // Target lane for this recording
};

/// Recorded MIDI event
struct RecordedMidiEvent {
    uint64_t timeSamples; // Absolute sample position
    uint8_t status;
    uint8_t data1;
    uint8_t data2;
    uint8_t channel;
};

/// Recorded MIDI chunk
struct RecordedMidiChunk {
    std::vector<RecordedMidiEvent> events;
    uint64_t startSample;
    std::string laneId; // Target lane for this recording
};

/// Simple lock-free single-producer, single-consumer queue for audio chunks
/// For Phase 7, we use a simple vector with atomic index (SPSC)
template<typename T>
class LockFreeQueue {
public:
    LockFreeQueue(size_t capacity) : _buffer(capacity), _writeIndex(0), _readIndex(0), _capacity(capacity) {}

    bool push(const T& item) {
        size_t currentWrite = _writeIndex.load(std::memory_order_relaxed);
        size_t nextWrite = (currentWrite + 1) % _capacity;

        // Check if queue is full (simple check - may have false positives)
        if (nextWrite == _readIndex.load(std::memory_order_acquire)) {
            return false; // Queue full
        }

        _buffer[currentWrite] = item;
        _writeIndex.store(nextWrite, std::memory_order_release);
        return true;
    }

    bool pop(T& item) {
        size_t currentRead = _readIndex.load(std::memory_order_relaxed);

        if (currentRead == _writeIndex.load(std::memory_order_acquire)) {
            return false; // Queue empty
        }

        item = _buffer[currentRead];
        _readIndex.store((currentRead + 1) % _capacity, std::memory_order_release);
        return true;
    }

    bool empty() const {
        return _readIndex.load(std::memory_order_acquire) == _writeIndex.load(std::memory_order_acquire);
    }

private:
    std::vector<T> _buffer;
    std::atomic<size_t> _writeIndex;
    std::atomic<size_t> _readIndex;
    size_t _capacity;
};

/// Recording session state
class RecordingSession {
public:
    RecordingSession();
    ~RecordingSession();

    /// Start recording at a specific playhead sample (control thread)
    void startRecording(uint64_t startSample);

    /// Stop recording (control thread)
    void stopRecording();

    /// Check if recording is active (audio thread safe)
    bool isRecording() const noexcept {
        return _isRecording.load(std::memory_order_acquire);
    }

    /// Set arm state for a lane (control thread)
    void setArmState(const std::string& laneId, bool armed);

    /// Replace the full armed-lane set atomically on the control thread.
    void replaceArmedLanes(const std::vector<std::string>& laneIds);

    /// Check if a lane is armed (audio thread safe)
    bool isLaneArmed(const std::string& laneId) const;

    /// Snapshot the current armed lane ids (control thread).
    std::vector<std::string> getArmedLaneIds() const;

    /// Bind input node to target lane (control thread)
    void bindInputToLane(const std::string& inputNodeId, const std::string& laneId);

    /// Clear all input bindings (control thread).
    void clearInputBindings();

    /// Replace all input bindings atomically on the control thread.
    void replaceInputBindings(const std::vector<std::pair<std::string, std::string>>& bindings);

    /// Get target lane for an input node (audio thread safe)
    std::string getTargetLaneForInput(const std::string& inputNodeId) const;

    /// Capture audio chunk (called from audio thread)
    /// Returns true if capture succeeded, false if queue full
    bool captureAudioChunk(const RecordedAudioChunk& chunk);

    /// Capture MIDI chunk (called from audio thread)
    /// Returns true if capture succeeded, false if queue full
    bool captureMidiChunk(const RecordedMidiChunk& chunk);

    /// Capture final mixed output (called from audio thread)
    /// Converts AudioBus to RecordedAudioChunk and queues for async flush
    /// @param output Final mixed output bus
    /// @param blockStartSamples Absolute sample position of block start
    /// @param laneId Target lane for recording (defaults to "master")
    /// @return true if capture succeeded, false if queue full or not recording
    bool captureFinalOutput(
        const class AudioBus& output,
        uint64_t blockStartSamples,
        const std::string& laneId = "master"
    );

    /// Consume captured audio chunks (control thread)
    /// Returns number of chunks consumed
    size_t consumeAudioChunks(std::vector<RecordedAudioChunk>& out);

    /// Consume captured MIDI chunks (control thread)
    /// Returns number of chunks consumed
    size_t consumeMidiChunks(std::vector<RecordedMidiChunk>& out);

    /// Get recording start sample (audio thread safe)
    uint64_t getRecordingStartSample() const noexcept {
        return _recordingStartSample.load(std::memory_order_acquire);
    }

private:
    std::atomic<bool> _isRecording;
    std::atomic<uint64_t> _recordingStartSample;

    // Arm state (protected by mutex for control thread updates, atomic reads for audio thread)
    std::unordered_map<std::string, std::atomic<bool>> _armedLanes;
    mutable std::shared_mutex _armStateMutex;

    // Input node to lane mapping
    std::unordered_map<std::string, std::string> _inputToLaneMap;
    mutable std::shared_mutex _inputMapMutex;

    // Capture queues (lock-free SPSC)
    static constexpr size_t CAPTURE_QUEUE_SIZE = 1024;
    LockFreeQueue<RecordedAudioChunk> _audioChunkQueue;
    LockFreeQueue<RecordedMidiChunk> _midiChunkQueue;
};
