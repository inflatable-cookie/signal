#include "vst3/Vst3PluginInstance.hpp"
#include <cstdint>
#include <algorithm>
#include <cmath>
#include <cstring>
#include <utility>

Vst3PluginInstance::Vst3PluginInstance(
    PluginDescriptor descriptor,
    std::string modulePath
)
    : _descriptor(std::move(descriptor))
    , _modulePath(std::move(modulePath))
{
    _parameterDescriptors.push_back(PluginParameterDescriptor{
        .paramId = "bypass",
        .name = "Bypass",
        .unit = "",
        .minValue = 0.0f,
        .maxValue = 1.0f,
        .defaultValue = 0.0f,
        .step = 1.0f,
        .isAutomatable = true,
        .isBypass = true
    });
    _parameterValues.emplace("bypass", 0.0f);
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
    return static_cast<int>(_parameterDescriptors.size());
}

std::string Vst3PluginInstance::getParameterId(int index) const {
    if (index < 0 || index >= static_cast<int>(_parameterDescriptors.size())) {
        return {};
    }

    return _parameterDescriptors[static_cast<std::size_t>(index)].paramId;
}

float Vst3PluginInstance::getParameterValue(const std::string& paramId) const {
    const auto it = _parameterValues.find(paramId);
    if (it == _parameterValues.end()) {
        return 0.0f;
    }

    return it->second;
}

std::vector<PluginParameterDescriptor> Vst3PluginInstance::listParameterDescriptors() const {
    return _parameterDescriptors;
}

void Vst3PluginInstance::setParameterValue(const std::string& paramId, float normalisedValue) {
    auto it = std::find_if(
        _parameterDescriptors.begin(),
        _parameterDescriptors.end(),
        [&paramId](const PluginParameterDescriptor& descriptor) {
            return descriptor.paramId == paramId;
        }
    );

    if (it == _parameterDescriptors.end()) {
        return;
    }

    float clamped = std::max(it->minValue, std::min(it->maxValue, normalisedValue));

    if (it->isBypass) {
        clamped = clamped >= 0.5f ? 1.0f : 0.0f;
    } else if (it->step > 0.0f) {
        clamped = std::round(clamped / it->step) * it->step;
        clamped = std::max(it->minValue, std::min(it->maxValue, clamped));
    }

    _parameterValues[paramId] = clamped;
}

std::vector<uint8_t> Vst3PluginInstance::getStateChunk() const {
    static constexpr char kMagic[] = {'V', 'S', 'T', '3', 'S', 'T', 'B', '1'};
    constexpr std::size_t kMagicSize = sizeof(kMagic);
    std::vector<uint8_t> state;
    state.reserve(
        kMagicSize
        + sizeof(std::int32_t) * 2
        + sizeof(std::uint8_t) * 2
        + sizeof(std::uint32_t)
        + _parameterValues.size() * (sizeof(std::uint16_t) + 16 + sizeof(float))
    );
    state.insert(state.end(), kMagic, kMagic + kMagicSize);

    std::int32_t inputs = static_cast<std::int32_t>(_descriptor.numAudioInputs);
    std::int32_t outputs = static_cast<std::int32_t>(_descriptor.numAudioOutputs);
    std::uint8_t midiIn = _descriptor.hasMidiInput ? 1 : 0;
    std::uint8_t midiOut = _descriptor.hasMidiOutput ? 1 : 0;

    auto appendBytes = [&state](const auto& value) {
        const auto* ptr = reinterpret_cast<const std::uint8_t*>(&value);
        state.insert(state.end(), ptr, ptr + sizeof(value));
    };

    appendBytes(inputs);
    appendBytes(outputs);
    appendBytes(midiIn);
    appendBytes(midiOut);

    const auto parameterCount = static_cast<std::uint32_t>(_parameterValues.size());
    appendBytes(parameterCount);

    for (const auto& [paramId, value] : _parameterValues) {
        const auto clampedSize = std::min<std::size_t>(paramId.size(), 0xffff);
        const auto idSize = static_cast<std::uint16_t>(clampedSize);
        appendBytes(idSize);
        state.insert(state.end(), paramId.begin(), paramId.begin() + static_cast<std::ptrdiff_t>(idSize));
        appendBytes(value);
    }

    return state;
}

void Vst3PluginInstance::setStateChunk(const std::vector<uint8_t>& data) {
    static constexpr char kMagic[] = {'V', 'S', 'T', '3', 'S', 'T', 'B', '1'};
    constexpr std::size_t kMagicSize = sizeof(kMagic);
    constexpr std::size_t kMinimumStateSize =
        kMagicSize + sizeof(std::int32_t) * 2 + sizeof(std::uint8_t) * 2 + sizeof(std::uint32_t);

    if (data.size() < kMinimumStateSize) {
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
    std::uint32_t parameterCount = 0;

    std::memcpy(&inputs, data.data() + offset, sizeof(inputs));
    offset += sizeof(inputs);
    std::memcpy(&outputs, data.data() + offset, sizeof(outputs));
    offset += sizeof(outputs);
    std::memcpy(&midiIn, data.data() + offset, sizeof(midiIn));
    offset += sizeof(midiIn);
    std::memcpy(&midiOut, data.data() + offset, sizeof(midiOut));
    offset += sizeof(midiOut);
    std::memcpy(&parameterCount, data.data() + offset, sizeof(parameterCount));
    offset += sizeof(parameterCount);

    if (inputs >= 0) {
        _descriptor.numAudioInputs = static_cast<int>(inputs);
    }

    if (outputs >= 0) {
        _descriptor.numAudioOutputs = static_cast<int>(outputs);
    }

    _descriptor.hasMidiInput = midiIn != 0;
    _descriptor.hasMidiOutput = midiOut != 0;

    for (std::uint32_t i = 0; i < parameterCount; ++i) {
        if (offset + sizeof(std::uint16_t) > data.size()) {
            break;
        }

        std::uint16_t idSize = 0;
        std::memcpy(&idSize, data.data() + offset, sizeof(idSize));
        offset += sizeof(idSize);

        if (offset + idSize + sizeof(float) > data.size()) {
            break;
        }

        std::string paramId(
            reinterpret_cast<const char*>(data.data() + offset),
            reinterpret_cast<const char*>(data.data() + offset + idSize)
        );
        offset += idSize;

        float value = 0.0f;
        std::memcpy(&value, data.data() + offset, sizeof(value));
        offset += sizeof(value);

        setParameterValue(paramId, value);
    }
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
