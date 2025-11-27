#pragma once

/// StreamScheduler - Manages scheduled stream playback
///
/// Thread: Control thread (main thread) for updates, audio thread for reads
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (EngineDomain handlers) - builds new ScheduleData
///   - Read by audio thread via lock-free atomic pointer (no locks)
///   - Provides playback state for audio thread
///
/// Thread Safety:
///   - Control thread: builds new ScheduleData in build buffer, atomically swaps pointer
///   - Audio thread: reads atomic pointer once per renderBlock, uses snapshot for entire block
///   - No locks in audio thread path
///
/// Architecture: Signal receives stream-based schedules from Pulse.
/// Pulse compiles Tracks → Lanes → Streams, and sends per-stream audio/MIDI content.
/// Signal processes streams via node graph, not via track/clip semantics.

#include "core/ScheduleData.hpp"
#include <atomic>
#include <memory>
#include <string>
#include <vector>
#include <cstdint>

class StreamScheduler {
public:
    StreamScheduler();
    ~StreamScheduler();

    /// Set schedule from Pulse's PlaybackScheduleCompiled
    /// This replaces the entire schedule atomically
    void setSchedule(
        const std::vector<StreamDescriptor>& streams,
        const std::vector<AudioSegmentCompiled>& audioSegments,
        const std::vector<MidiEventCompiled>& midiEvents,
        const TempoMap& tempoMap,
        double sampleRate
    );

    /// Clear all scheduled streams
    void clearSchedule();

    /// Get active audio segments for a stream at a given sample position
    /// Thread-safe: reads atomic pointer (lock-free)
    /// @param streamId Stream to query
    /// @param samplePosition Current sample position
    /// @return Vector of active audio segments (pointers valid for current render block)
    std::vector<const AudioSegmentCompiled*> getActiveAudioSegments(
        const std::string& streamId,
        uint64_t samplePosition
    ) const;

    /// Get MIDI events for a stream in a sample range
    /// Thread-safe: reads atomic pointer (lock-free)
    /// @param streamId Stream to query
    /// @param startSample Start of range (inclusive)
    /// @param endSample End of range (exclusive)
    /// @return Vector of MIDI events in range (pointers valid for current render block)
    std::vector<const MidiEventCompiled*> getMidiEventsInRange(
        const std::string& streamId,
        uint64_t startSample,
        uint64_t endSample
    ) const;

    /// Get current schedule snapshot (for audio thread)
    /// Thread-safe: reads atomic pointer (lock-free)
    /// @return Pointer to current schedule (valid until next swap)
    const ScheduleData* getSchedule() const;

    /// Check if schedule has been loaded (has streams)
    /// Thread-safe: reads atomic pointer (lock-free)
    bool hasSchedule() const noexcept;

    /// Get count of active streams (for diagnostics)
    /// Thread-safe: reads atomic pointer (lock-free)
    int getActiveStreamCount() const noexcept;

private:
    // Active schedule snapshot (read by audio thread, swapped by control thread)
    // Using raw pointer with shared_ptr for lifetime management
    std::atomic<const ScheduleData*> _activeSchedule;

    // Empty schedule (used when clearing)
    std::shared_ptr<ScheduleData> _emptySchedule;

    // Current schedule (control thread only, used for building new schedules)
    std::shared_ptr<ScheduleData> _currentSchedule;

    // Keep previous schedule alive until next swap (ensures audio thread safety)
    std::shared_ptr<ScheduleData> _previousSchedule;
};

