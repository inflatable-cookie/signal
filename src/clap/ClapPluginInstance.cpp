#include "clap/ClapPluginInstance.hpp"
#include <iostream>
#include <algorithm>
#include <cmath>

// Phase 4: Minimal CLAP stub implementation
// This will be replaced with real CLAP hosting in a future phase
// For now, it provides a safe no-op that can be tested

ClapPluginInstance::ClapPluginInstance(const PluginDescriptor& desc)
    : _descriptor(desc)
    , _sampleRate(44100.0)
    , _maxBlockSize(512)
    , _prepared(false)
    , _active(false)
    , _clapPlugin(nullptr)
{
    // Phase 4: Stub - attempt to load plugin (will fail gracefully)
    // Always initialize stub mode parameters (will be replaced by real plugin params if loading succeeds)
    _parameterIds.push_back("gain");
    _parameterValues.push_back(1.0f);

    if (!loadClapPlugin()) {
        std::cerr << "[ClapPluginInstance] Failed to load CLAP plugin: " << desc.id << " (using stub mode)" << std::endl;
        // Continue with stub mode (safe no-op)
    }
}

ClapPluginInstance::~ClapPluginInstance() {
    deactivatePlugin();
    unloadClapPlugin();
}

bool ClapPluginInstance::loadClapPlugin() {
    // Phase 4: Stub implementation
    // TODO: Real CLAP loading will:
    // 1. Search for .clap bundle/so/dylib
    // 2. Load CLAP entry point
    // 3. Create plugin instance
    // 4. Initialize plugin
    // For now, return false (stub mode)
    _clapPlugin = nullptr;
    return false;
}

void ClapPluginInstance::unloadClapPlugin() {
    // Phase 4: Stub - no cleanup needed
    _clapPlugin = nullptr;
}

void ClapPluginInstance::activatePlugin() {
    if (_active || !_prepared) {
        return;
    }
    // Phase 4: Stub - would call clap_plugin->activate() here
    _active = true;
}

void ClapPluginInstance::deactivatePlugin() {
    if (!_active) {
        return;
    }
    // Phase 4: Stub - would call clap_plugin->deactivate() here
    _active = false;
}

void ClapPluginInstance::prepare(double sampleRate, int maxBlockSize) {
    _sampleRate = sampleRate;
    _maxBlockSize = maxBlockSize;

    if (_prepared) {
        return;
    }

    // Phase 4: Stub - would call clap_plugin->init() and clap_plugin->activate() here
    // For now, just mark as prepared
    _prepared = true;

    // Initialize parameter cache if not already done (stub - would query plugin for real parameters)
    if (_parameterIds.empty()) {
        // Add a dummy parameter for testing
        _parameterIds.push_back("gain");
        _parameterValues.push_back(1.0f); // Default: unity gain
    }

    // Activate plugin
    activatePlugin();

    std::cout << "[ClapPluginInstance] Prepared plugin: " << _descriptor.name
              << " (stub mode)" << std::endl;
}

void ClapPluginInstance::reset() {
    // Phase 4: Stub - would call clap_plugin->reset() here
    // Reset parameter values to defaults
    for (size_t i = 0; i < _parameterValues.size(); ++i) {
        _parameterValues[i] = 1.0f; // Default values
    }
}

void ClapPluginInstance::processAudioMidi(
    AudioBuffer& audioIn,
    AudioBuffer& audioOut,
    MidiBuffer& midiIn,
    MidiBuffer& midiOut,
    const NodeProcessContext& ctx
) {
    if (!_prepared || !_active) {
        // Stub mode: pass-through
        audioOut.copyFrom(audioIn);
        midiOut.clear();
        midiOut.append(midiIn);
        return;
    }

    // Phase 4: Stub implementation
    // Real CLAP processing would:
    // 1. Convert AudioBuffer to CLAP audio buffers
    // 2. Convert MidiBuffer to CLAP MIDI events
    // 3. Call clap_plugin->process()
    // 4. Convert CLAP output back to AudioBuffer/MidiBuffer

    // For now, apply a simple gain (from "gain" parameter) as a test
    // If no "gain" parameter exists, use default 1.0
    float gain = 1.0f;
    if (!_parameterIds.empty() && _parameterIds[0] == "gain") {
        gain = _parameterValues[0];
    }

    int numChannels = std::min(audioIn.numChannels(), audioOut.numChannels());
    int numFrames = std::min(audioIn.numFrames(), audioOut.numFrames());

    for (int ch = 0; ch < numChannels; ++ch) {
        for (int frame = 0; frame < numFrames; ++frame) {
            float sample = audioIn.getSample(frame, ch) * gain;
            audioOut.setSample(frame, ch, sample);
        }
    }

    // MIDI pass-through
    midiOut.clear();
    midiOut.append(midiIn);
}

int ClapPluginInstance::getNumParameters() const {
    return static_cast<int>(_parameterIds.size());
}

std::string ClapPluginInstance::getParameterId(int index) const {
    if (index >= 0 && index < static_cast<int>(_parameterIds.size())) {
        return _parameterIds[index];
    }
    return "";
}

float ClapPluginInstance::getParameterValue(const std::string& paramId) const {
    for (size_t i = 0; i < _parameterIds.size(); ++i) {
        if (_parameterIds[i] == paramId) {
            return _parameterValues[i];
        }
    }
    return 0.0f;
}

void ClapPluginInstance::setParameterValue(const std::string& paramId, float normalisedValue) {
    // Clamp to [0.0, 1.0]
    normalisedValue = std::max(0.0f, std::min(1.0f, normalisedValue));

    for (size_t i = 0; i < _parameterIds.size(); ++i) {
        if (_parameterIds[i] == paramId) {
            _parameterValues[i] = normalisedValue;
            // Phase 4: Stub - would call clap_plugin->set_parameter_value() here
            return;
        }
    }
}

std::vector<uint8_t> ClapPluginInstance::getStateChunk() const {
    // Phase 4: Stub - would call clap_plugin->get_state() here
    // For now, return empty (not supported in stub mode)
    return std::vector<uint8_t>();
}

void ClapPluginInstance::setStateChunk(const std::vector<uint8_t>& data) {
    // Phase 4: Stub - would call clap_plugin->set_state() here
    // For now, no-op
    (void)data;
}

// Factory function
std::unique_ptr<PluginInstance> createClapInstance(const PluginDescriptor& desc) {
    try {
        auto instance = std::make_unique<ClapPluginInstance>(desc);
        std::cout << "[createClapInstance] Created CLAP instance: " << desc.id << std::endl;
        return instance;
    } catch (const std::exception& e) {
        std::cerr << "[createClapInstance] Failed to create CLAP instance: " << e.what() << std::endl;
        return nullptr;
    }
}

