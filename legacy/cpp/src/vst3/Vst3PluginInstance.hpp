#pragma once

/// Vst3PluginInstance - Phase 80 runtime scaffold instance
///
/// Thread: Control thread (prepare/reset/state), audio thread (process)
/// Ownership: Created by Vst3Backend, owned by PluginNode

#include "core/PluginInstance.hpp"
#include <string>
#include <unordered_map>

class Vst3PluginInstance : public PluginInstance {
public:
    Vst3PluginInstance(
        PluginDescriptor descriptor,
        std::string modulePath
    );

    ~Vst3PluginInstance() override = default;

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
    std::vector<PluginParameterDescriptor> listParameterDescriptors() const override;
    void setParameterValue(const std::string& paramId, float normalisedValue) override;

    std::vector<uint8_t> getStateChunk() const override;
    void setStateChunk(const std::vector<uint8_t>& data) override;

    const PluginDescriptor& getDescriptor() const override;
    bool negotiateAudioIO(int requestedInputs, int requestedOutputs) override;

private:
    PluginDescriptor _descriptor;
    std::string _modulePath;
    std::vector<PluginParameterDescriptor> _parameterDescriptors;
    std::unordered_map<std::string, float> _parameterValues;
    double _sampleRate{0.0};
    int _maxBlockSize{0};
};
