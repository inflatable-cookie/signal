#include "vst3/Vst3PluginInstance.hpp"
#include <utility>

Vst3PluginInstance::Vst3PluginInstance(
    PluginDescriptor descriptor,
    std::string modulePath
)
    : _descriptor(std::move(descriptor))
    , _modulePath(std::move(modulePath))
{
}

void Vst3PluginInstance::prepare(double sampleRate, int maxBlockSize) {
    _sampleRate = sampleRate;
    _maxBlockSize = maxBlockSize;
}

void Vst3PluginInstance::reset() {
    (void) _sampleRate;
    (void) _maxBlockSize;
}

void Vst3PluginInstance::processAudioMidi(
    AudioBuffer& audioIn,
    AudioBuffer& audioOut,
    MidiBuffer& midiIn,
    MidiBuffer& midiOut,
    const NodeProcessContext& ctx
) {
    (void) ctx;

    audioOut.copyFrom(audioIn);
    midiOut.clear();
    midiOut.append(midiIn);
}

int Vst3PluginInstance::getNumParameters() const {
    return 0;
}

std::string Vst3PluginInstance::getParameterId(int index) const {
    (void) index;
    return {};
}

float Vst3PluginInstance::getParameterValue(const std::string& paramId) const {
    (void) paramId;
    return 0.0f;
}

void Vst3PluginInstance::setParameterValue(const std::string& paramId, float normalisedValue) {
    (void) paramId;
    (void) normalisedValue;
}

std::vector<uint8_t> Vst3PluginInstance::getStateChunk() const {
    return {};
}

void Vst3PluginInstance::setStateChunk(const std::vector<uint8_t>& data) {
    (void) data;
}

const PluginDescriptor& Vst3PluginInstance::getDescriptor() const {
    return _descriptor;
}

bool Vst3PluginInstance::negotiateAudioIO(int requestedInputs, int requestedOutputs) {
    if (requestedInputs < 0 || requestedOutputs < 0) {
        return false;
    }

    _descriptor.numAudioInputs = requestedInputs;
    _descriptor.numAudioOutputs = requestedOutputs;
    return true;
}
