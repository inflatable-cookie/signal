#pragma once

/// ClapPluginInstance - Real CLAP plugin host adapter
///
/// Thread: Control thread (prepare, reset, parameter changes)
///         Audio thread (processAudioMidi - real-time safe)
/// Ownership: Owned by plugin nodes
///
/// This implements the PluginInstance interface by wrapping real CLAP plugin functionality.
/// Phase 5: Replaces stub implementation with real CLAP loading and processing.

#include "core/PluginInstance.hpp"
#include "clap/ClapPluginLibrary.hpp"
#include "clap/MusicalTimeInfo.hpp"
#include "clap/clap.h"
#include <string>
#include <vector>
#include <memory>

/// CLAP plugin instance implementation
class ClapPluginInstance : public PluginInstance {
public:
    /// Create CLAP plugin instance from library and descriptor
    /// @param library Shared pointer to CLAP library (keeps it alive)
    /// @param clapDesc CLAP plugin descriptor
    ClapPluginInstance(
        std::shared_ptr<ClapPluginLibrary> library,
        const clap_plugin_descriptor_t* clapDesc
    );
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
    std::shared_ptr<ClapPluginLibrary> _library;
    const clap_plugin_descriptor_t* _clapDesc;
    const clap_plugin_t* _plugin;
    clap_host_t _host;

    double _sampleRate;
    int _maxBlockSize;
    bool _prepared;
    bool _active;

    // Parameter cache (for real-time safe access)
    struct ParameterInfo {
        std::string id;
        clap_id clapId;
        double minValue;
        double maxValue;
        double defaultValue;
        double currentValue; // Normalised 0..1
    };
    std::vector<ParameterInfo> _parameters;
    std::unordered_map<std::string, size_t> _paramIdToIndex;

    // CLAP extensions
    const clap_plugin_params* _paramsExt;
    const clap_plugin_state* _stateExt;

    // Audio/MIDI buffers for CLAP processing
    struct ClapAudioBuffers {
        std::vector<const float*> inputChannels;
        std::vector<float*> outputChannels;
        std::vector<clap_audio_buffer> inputBuffers;
        std::vector<clap_audio_buffer> outputBuffers;
    };
    ClapAudioBuffers _audioBuffers;

    // MIDI event conversion
    struct ClapMidiEvents {
        std::vector<clap_event_midi> events;
        clap_input_events inputEvents;
        clap_output_events outputEvents;
    };
    ClapMidiEvents _midiEvents;

    // Time-info support
    MusicalTimeInfo _currentTimeInfo;
    void updateTimeInfo(const NodeProcessContext& ctx);

    // Helper methods
    bool createPlugin();
    void destroyPlugin();
    void activatePlugin();
    void deactivatePlugin();
    void queryParameters();
    void queryExtensions();

    // CLAP host callbacks
    static void hostRequestRestart(const clap_host* host);
    static void hostRequestProcess(const clap_host* host);
    static void hostRequestCallback(const clap_host* host);
    static const void* hostGetExtension(const clap_host* host, const char* extension_id);

    // CLAP MIDI event callbacks (static)
    static uint32_t inputEventsSize(const clap_input_events* events);
    static const clap_event_header_t* inputEventsGet(const clap_input_events* events, uint32_t index);
    static bool outputEventsTryPush(const clap_output_events* events, const clap_event_header_t* header);

    // MIDI conversion helpers
    void convertMidiToClap(const MidiBuffer& midiIn);
    void convertClapToMidi(MidiBuffer& midiOut);
};

/// Factory function for creating CLAP instances (called by PluginHost)
std::unique_ptr<PluginInstance> createClapInstance(
    std::shared_ptr<ClapPluginLibrary> library,
    const clap_plugin_descriptor_t* clapDesc
);
