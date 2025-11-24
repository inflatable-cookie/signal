#pragma once

/// ScheduleData - Immutable schedule snapshot for audio thread
///
/// Thread: Audio thread (read-only)
/// Ownership: Owned by StreamScheduler, swapped atomically
///
/// This structure contains a complete, self-contained snapshot of the schedule
/// that can be safely read from the audio thread without locking.
///
/// Architecture: Signal receives stream-based schedules from Pulse.
/// Pulse compiles Tracks → Lanes → Streams, and sends per-stream audio/MIDI content.
/// Signal processes streams via node graph, not via track/clip semantics.

#include <string>
#include <unordered_map>
#include <vector>
#include <cstdint>
#include <memory>

/// Stream descriptor (from Pulse)
struct StreamDescriptor {
    std::string streamId;
    std::string trackId;  // For debugging/metadata only
    std::string laneId;   // For debugging/metadata only
    std::string streamType; // "audio" | "midi"
};

/// Compiled audio segment (sample-based, for audio thread)
struct AudioSegmentCompiled {
    std::string streamId;
    std::string assetId;
    uint64_t startSamples;
    uint64_t endSamples;
    uint64_t assetStartSamples; // Offset into asset
};

/// Compiled MIDI event (sample-based, for audio thread)
struct MidiEventCompiled {
    std::string streamId;
    uint64_t timeSamples;
    uint8_t status;
    uint8_t data1;
    uint8_t data2;
    uint8_t channel;
};

/// Tempo map entry (simplified - full implementation in future)
struct TempoMapEntry {
    double timeBeats;
    double tempo; // BPM
};

/// Tempo map (simplified - full implementation in future)
struct TempoMap {
    std::vector<TempoMapEntry> entries;
    double defaultTempo; // Default tempo if map is empty
};

/// Immutable schedule snapshot (stream-based)
///
/// This structure is fully self-contained and can be safely read from the
/// audio thread. Once created, it is never modified - new schedules create
/// new ScheduleData instances.
///
/// Architecture alignment:
/// - Streams are the exclusive identifiers of input sources
/// - AudioSegments and MidiEvents are organized by streamId
/// - No track/clip semantics - only streams
struct ScheduleData {
    // Stream descriptors (all streams in this schedule)
    std::vector<StreamDescriptor> streams;

    // Audio segments (organized by streamId for efficient lookup)
    std::vector<AudioSegmentCompiled> audioSegments;
    std::unordered_map<std::string, std::vector<const AudioSegmentCompiled*>> audioSegmentsByStream;

    // MIDI events (organized by streamId for efficient lookup)
    std::vector<MidiEventCompiled> midiEvents;
    std::unordered_map<std::string, std::vector<const MidiEventCompiled*>> midiEventsByStream;

    // Tempo map
    TempoMap tempoMap;

    // Sample rate (immutable for this snapshot)
    double sampleRate;

    ScheduleData(double sampleRate, double defaultTempo = 120.0)
        : sampleRate(sampleRate)
    {
        tempoMap.defaultTempo = defaultTempo;
    }

    // Non-copyable (we use unique_ptr for ownership)
    ScheduleData(const ScheduleData&) = delete;
    ScheduleData& operator=(const ScheduleData&) = delete;

    // Movable
    ScheduleData(ScheduleData&&) = default;
    ScheduleData& operator=(ScheduleData&&) = default;

    // Build lookup maps (called after segments/events are added)
    void buildLookupMaps();
};
