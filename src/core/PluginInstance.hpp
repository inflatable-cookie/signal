#pragma once

/// PluginInstance - Engine-agnostic plugin instance interface
///
/// Thread: Control thread (prepare, reset, parameter changes)
///         Audio thread (processAudioMidi - read-only parameter access)
/// Ownership: Owned by plugin nodes (MidiFxNode, InstrumentNode, AudioFxNode)
///
/// This interface abstracts plugin hosting across different formats (CLAP, VST3, AU, LV2).
/// Implementations handle format-specific details while providing a uniform API.

#include "core/NodeBuffers.hpp"
#include "core/NodeProcessContext.hpp"
#include "core/GraphSnapshot.hpp" // For PluginFormat enum
#include <string>
#include <vector>
#include <cstdint>

/// Plugin descriptor (metadata about a plugin)
struct PluginDescriptor {
    PluginFormat format;
    std::string id;          // CLAP id, VST3 uid, etc.
    std::string name;
    int numAudioInputs;
    int numAudioOutputs;
    bool hasMidiInput;
    bool hasMidiOutput;
    // Future: latency, bus layouts, etc.
};

/// Plugin instance interface
class PluginInstance {
public:
    virtual ~PluginInstance() = default;

    /// Prepare plugin for processing (called on control thread)
    /// @param sampleRate Sample rate (e.g., 44100.0)
    /// @param maxBlockSize Maximum block size (e.g., 512)
    virtual void prepare(double sampleRate, int maxBlockSize) = 0;

    /// Reset plugin state (called on control thread)
    virtual void reset() = 0;

    /// Process audio and MIDI (called on audio thread)
    /// Must be real-time safe (no locks, allocations, or I/O)
    /// @param audioIn Input audio buffer
    /// @param audioOut Output audio buffer
    /// @param midiIn Input MIDI buffer
    /// @param midiOut Output MIDI buffer
    /// @param ctx Processing context
    virtual void processAudioMidi(
        AudioBuffer& audioIn,
        AudioBuffer& audioOut,
        MidiBuffer& midiIn,
        MidiBuffer& midiOut,
        const NodeProcessContext& ctx
    ) = 0;

    /// Get number of parameters
    virtual int getNumParameters() const = 0;

    /// Get parameter ID at index
    /// @param index Parameter index (0..getNumParameters()-1)
    virtual std::string getParameterId(int index) const = 0;

    /// Get parameter value (normalised 0..1)
    /// @param paramId Parameter ID
    virtual float getParameterValue(const std::string& paramId) const = 0;

    /// Set parameter value (normalised 0..1)
    /// Called on control thread or via lock-free mechanism from audio thread
    /// @param paramId Parameter ID
    /// @param normalisedValue Normalised value (0.0..1.0)
    virtual void setParameterValue(const std::string& paramId, float normalisedValue) = 0;

    /// Get plugin state as byte chunk (for saving presets)
    /// @return State data (empty if not supported)
    virtual std::vector<uint8_t> getStateChunk() const = 0;

    /// Set plugin state from byte chunk (for loading presets)
    /// @param data State data
    virtual void setStateChunk(const std::vector<uint8_t>& data) = 0;

    /// Get plugin descriptor
    virtual const PluginDescriptor& getDescriptor() const = 0;
};

