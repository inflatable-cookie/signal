#include "vst3/Vst3PluginInstance.hpp"
#include <cstdint>
#include <cstring>
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
    static constexpr char kMagic[] = {'V', 'S', 'T', '3', 'S', 'T', 'B', '1'};
    constexpr std::size_t kMagicSize = sizeof(kMagic);
    constexpr std::size_t kStateSize = kMagicSize + sizeof(std::int32_t) * 2 + sizeof(std::uint8_t) * 2;

    std::vector<uint8_t> state(kStateSize);
    std::memcpy(state.data(), kMagic, kMagicSize);

    std::int32_t inputs = static_cast<std::int32_t>(_descriptor.numAudioInputs);
    std::int32_t outputs = static_cast<std::int32_t>(_descriptor.numAudioOutputs);
    std::uint8_t midiIn = _descriptor.hasMidiInput ? 1 : 0;
    std::uint8_t midiOut = _descriptor.hasMidiOutput ? 1 : 0;

    std::size_t offset = kMagicSize;
    std::memcpy(state.data() + offset, &inputs, sizeof(inputs));
    offset += sizeof(inputs);
    std::memcpy(state.data() + offset, &outputs, sizeof(outputs));
    offset += sizeof(outputs);
    std::memcpy(state.data() + offset, &midiIn, sizeof(midiIn));
    offset += sizeof(midiIn);
    std::memcpy(state.data() + offset, &midiOut, sizeof(midiOut));

    return state;
}

void Vst3PluginInstance::setStateChunk(const std::vector<uint8_t>& data) {
    static constexpr char kMagic[] = {'V', 'S', 'T', '3', 'S', 'T', 'B', '1'};
    constexpr std::size_t kMagicSize = sizeof(kMagic);
    constexpr std::size_t kStateSize = kMagicSize + sizeof(std::int32_t) * 2 + sizeof(std::uint8_t) * 2;

    if (data.size() < kStateSize) {
        return;
    }

    if (std::memcmp(data.data(), kMagic, kMagicSize) != 0) {
        return;
    }

    std::size_t offset = kMagicSize;
    std::int32_t inputs = 0;
    std::int32_t outputs = 0;
    std::uint8_t midiIn = 0;
    std::uint8_t midiOut = 0;

    std::memcpy(&inputs, data.data() + offset, sizeof(inputs));
    offset += sizeof(inputs);
    std::memcpy(&outputs, data.data() + offset, sizeof(outputs));
    offset += sizeof(outputs);
    std::memcpy(&midiIn, data.data() + offset, sizeof(midiIn));
    offset += sizeof(midiIn);
    std::memcpy(&midiOut, data.data() + offset, sizeof(midiOut));

    if (inputs >= 0) {
        _descriptor.numAudioInputs = static_cast<int>(inputs);
    }

    if (outputs >= 0) {
        _descriptor.numAudioOutputs = static_cast<int>(outputs);
    }

    _descriptor.hasMidiInput = midiIn != 0;
    _descriptor.hasMidiOutput = midiOut != 0;
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
