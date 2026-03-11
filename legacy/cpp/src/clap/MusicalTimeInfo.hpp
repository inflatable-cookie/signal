#pragma once

/// MusicalTimeInfo - Musical timing information for plugins
///
/// Provides tempo, time signature, song position, and loop information
/// for use by CLAP plugins and other time-aware components.

struct MusicalTimeInfo {
    double tempo;                 // BPM
    int timeSigNumerator;         // Time signature numerator (beats per bar)
    int timeSigDenominator;       // Time signature denominator (beat unit)
    double songPosBeats;          // Current position in beats
    double songPosSeconds;        // Current position in seconds (optional)
    bool playing;                 // Is transport playing?
    bool loopEnabled;             // Is loop enabled?
    double loopStartBeats;        // Loop start in beats
    double loopEndBeats;          // Loop end in beats
};

