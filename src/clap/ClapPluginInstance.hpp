#pragma once

/// ClapPluginInstance - CLAP plugin host adapter
///
/// Thread: Control thread (prepare, reset, parameter changes)
///         Audio thread (processAudioMidi - real-time safe)
/// Ownership: Owned by plugin nodes
///
/// This is a minimal CLAP host adapter for Phase 4. It implements the PluginInstance
/// interface by wrapping CLAP plugin functionality.

#include "core/PluginInstance.hpp"
#include <string>
#include <vector>
#include <memory>

// Forward declarations for CLAP types
// For Phase 4, we'll use a minimal stub that can be replaced with real CLAP later
struct ClapPlugin;
struct ClapHost;

/// CLAP plugin instance implementation
class ClapPluginInstance : public PluginInstance {
public:
    ClapPluginInstance(const PluginDescriptor& desc);
    ~ClapPluginInstance() override;

    // PluginInstance interface
    void prepare(double sampleRate, int maxBlockSize) override;
    void reset() override;
    void processAudioMidi(
        AudioBuffer& audioIn,
        AudioBuffer& audioOut,
        MidiBuffer& midiIn,
        MidiBuffer& midiOut,
        const NodeProcessContext& ctx
    ) override;

    int getNumParameters() const override;
    std::string getParameterId(int index) const override;
    float getParameterValue(const std::string& paramId) const override;
    void setParameterValue(const std::string& paramId, float normalisedValue) override;

    std::vector<uint8_t> getStateChunk() const override;
    void setStateChunk(const std::vector<uint8_t>& data) override;

    const PluginDescriptor& getDescriptor() const override { return _descriptor; }

private:
    PluginDescriptor _descriptor;
    double _sampleRate;
    int _maxBlockSize;
    bool _prepared;
    bool _active;

    // CLAP plugin handle (will be real CLAP plugin pointer in future)
    // For Phase 4, this is a stub
    void* _clapPlugin; // Placeholder - will be ClapPlugin* in real implementation

    // Parameter cache (for real-time safe access)
    std::vector<std::string> _parameterIds;
    std::vector<float> _parameterValues; // Normalised 0..1

    // Helper methods
    bool loadClapPlugin();
    void unloadClapPlugin();
    void activatePlugin();
    void deactivatePlugin();
};

/// Factory function for creating CLAP instances (called by PluginHost)
std::unique_ptr<PluginInstance> createClapInstance(const PluginDescriptor& desc);

