#include "clap/ClapPluginInstance.hpp"
#include "core/NodeBuffers.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <cmath>
#include <cstring>
#include <sstream>

// Phase 5: Real CLAP implementation
// This replaces the Phase 4 stub with actual CLAP plugin loading and processing

ClapPluginInstance::ClapPluginInstance(
    std::shared_ptr<ClapPluginLibrary> library,
    const clap_plugin_descriptor_t* clapDesc
)
    : _library(library)
    , _clapDesc(clapDesc)
    , _plugin(nullptr)
    , _sampleRate(44100.0)
    , _maxBlockSize(512)
    , _prepared(false)
    , _active(false)
    , _paramsExt(nullptr)
    , _stateExt(nullptr)
{
    // Build PluginDescriptor from CLAP descriptor
    _descriptor.format = PluginFormat::Clap;
    _descriptor.id = clapDesc->id ? clapDesc->id : "";
    _descriptor.name = clapDesc->name ? clapDesc->name : _descriptor.id;

    // Audio I/O will be queried from CLAP plugin ports after plugin creation
    // Use defaults for now (will be updated in queryAudioPorts)
    _descriptor.numAudioInputs = 2;
    _descriptor.numAudioOutputs = 2;
    _descriptor.hasMidiInput = true;
    _descriptor.hasMidiOutput = false;

    // Initialize CLAP host structure
    _host.host_data = this;
    _host.clap_version = CLAP_VERSION; // clap_version_t struct
    _host.name = "Loophole Signal";
    _host.vendor = "Infinite Loop Audio";
    _host.url = "https://github.com/infinite-loop-audio/loophole-signal";
    _host.version = "0.1.0";
    _host.request_restart = hostRequestRestart;
    _host.request_process = hostRequestProcess;
    _host.request_callback = hostRequestCallback;
    _host.get_extension = hostGetExtension;

    // Create plugin instance
    if (!createPlugin()) {
        LOG_ERROR({"ClapPluginInstance"}, std::string("Failed to create plugin: ") + _descriptor.id);
    }
}

ClapPluginInstance::~ClapPluginInstance() {
    destroyPlugin();
}

bool ClapPluginInstance::createPlugin() {
    if (!_library || !_library->isValid() || !_clapDesc) {
        return false;
    }

    const clap_plugin_factory_t* factory = _library->getFactory();
    if (!factory || !factory->create_plugin) {
        LOG_ERROR({"ClapPluginInstance"}, "Factory does not provide create_plugin");
        return false;
    }

    _plugin = factory->create_plugin(factory, &_host, _clapDesc->id);
    if (!_plugin) {
        LOG_ERROR({"ClapPluginInstance"}, "Factory failed to create plugin");
        return false;
    }

    // Initialize plugin
    if (!_plugin->init || !_plugin->init(_plugin)) {
        LOG_ERROR({"ClapPluginInstance"}, "Plugin init failed");
        _plugin = nullptr;
        return false;
    }

    // Query extensions
    queryExtensions();

    // Query parameters
    queryParameters();

    // Audio ports will be queried during negotiateAudioIO() call
    // Don't query here - wait for explicit negotiation request

    LOG_DEBUG({"ClapPluginInstance"}, std::string("Created plugin: ") + _descriptor.name);
    return true;
}

void ClapPluginInstance::destroyPlugin() {
    if (_plugin) {
        deactivatePlugin();

        if (_plugin->destroy) {
            _plugin->destroy(_plugin);
        }
        _plugin = nullptr;
    }
}

void ClapPluginInstance::queryExtensions() {
    if (!_plugin || !_plugin->get_extension) {
        return;
    }

    // Query parameter extension
    _paramsExt = static_cast<const clap_plugin_params*>(
        _plugin->get_extension(_plugin, CLAP_EXT_PARAMS)
    );

    // Query state extension
    _stateExt = static_cast<const clap_plugin_state*>(
        _plugin->get_extension(_plugin, CLAP_EXT_STATE)
    );

    // Query audio ports extension
    _audioPortsExt = static_cast<const clap_plugin_audio_ports*>(
        _plugin->get_extension(_plugin, CLAP_EXT_AUDIO_PORTS)
    );
}

void ClapPluginInstance::queryParameters() {
    _parameters.clear();
    _paramIdToIndex.clear();

    if (!_paramsExt || !_paramsExt->count) {
        return;
    }

    const uint32_t count = _paramsExt->count(_plugin);
    _parameters.reserve(count);

    for (uint32_t i = 0; i < count; ++i) {
        clap_param_info info = {};
        if (!_paramsExt->get_info || !_paramsExt->get_info(_plugin, i, &info)) {
            continue;
        }

        ParameterInfo paramInfo;
        paramInfo.clapId = info.id;
        paramInfo.name = info.name[0] != '\0' ? std::string(info.name) : std::string("param_") + std::to_string(info.id);
        paramInfo.unit = info.module[0] != '\0' ? std::string(info.module) : std::string();
        paramInfo.minValue = info.min_value;
        paramInfo.maxValue = info.max_value;
        paramInfo.defaultValue = info.default_value;
        paramInfo.step = 0.0;
        paramInfo.isAutomatable = (info.flags & CLAP_PARAM_IS_AUTOMATABLE) != 0;
        paramInfo.isBypass = (info.flags & CLAP_PARAM_IS_BYPASS) != 0;
        paramInfo.currentValue = info.default_value; // Will be normalised later

        // Normalise default value to 0..1
        if (paramInfo.maxValue > paramInfo.minValue) {
            paramInfo.currentValue = (paramInfo.defaultValue - paramInfo.minValue) /
                                     (paramInfo.maxValue - paramInfo.minValue);
        } else {
            paramInfo.currentValue = 0.0;
        }

        // Get parameter ID (use name if no ID string available)
        // Note: info.name is a fixed-size array, so check if first char is non-null
        paramInfo.id = paramInfo.name;

        _parameters.push_back(paramInfo);
        _paramIdToIndex[paramInfo.id] = _parameters.size() - 1;
    }

    LOG_VERBOSE({"ClapPluginInstance"}, std::string("Found ") + std::to_string(_parameters.size()) + " parameters");
}

void ClapPluginInstance::queryAudioPorts(int requestedInputs, int requestedOutputs) {
    // Query CLAP audio ports extension to determine actual I/O configuration
    // This is called during plugin creation and can be called again during negotiation

    if (!_audioPortsExt) {
        // No audio ports extension - use defaults
        _descriptor.numAudioInputs = requestedInputs;
        _descriptor.numAudioOutputs = requestedOutputs;
        return;
    }

    // Query input port count
    uint32_t numInputPorts = 0;
    if (_audioPortsExt->count && _audioPortsExt->count(_plugin, true)) {
        numInputPorts = _audioPortsExt->count(_plugin, true);
    }

    // Query output port count
    uint32_t numOutputPorts = 0;
    if (_audioPortsExt->count && _audioPortsExt->count(_plugin, false)) {
        numOutputPorts = _audioPortsExt->count(_plugin, false);
    }

    // If plugin has ports, query the first input and output port for channel counts
    int chosenInputs = requestedInputs;
    int chosenOutputs = requestedOutputs;

    if (numInputPorts > 0 && _audioPortsExt->get) {
        clap_audio_port_info info = {};
        if (_audioPortsExt->get(_plugin, 0, true, &info)) {
            // Use port's channel count if available
            if (info.channel_count > 0) {
                chosenInputs = static_cast<int>(info.channel_count);
            }
        }
    }

    if (numOutputPorts > 0 && _audioPortsExt->get) {
        clap_audio_port_info info = {};
        if (_audioPortsExt->get(_plugin, 0, false, &info)) {
            // Use port's channel count if available
            if (info.channel_count > 0) {
                chosenOutputs = static_cast<int>(info.channel_count);
            }
        }
    }

    // Negotiate: find exact match with requested counts
    // Phase 3: Snapshot is source of truth - we verify plugin supports it, don't change it
    if (numInputPorts > 0) {
        // Check if requested matches any input port
        bool foundMatch = false;
        for (uint32_t i = 0; i < numInputPorts; ++i) {
            clap_audio_port_info info = {};
            if (_audioPortsExt->get(_plugin, i, true, &info)) {
                if (info.channel_count == static_cast<uint32_t>(requestedInputs)) {
                    chosenInputs = requestedInputs;
                    foundMatch = true;
                    break;
                }
            }
        }
        // If no exact match, use first port's channel count (will be checked by caller)
        if (!foundMatch && numInputPorts > 0) {
            clap_audio_port_info info = {};
            if (_audioPortsExt->get(_plugin, 0, true, &info)) {
                chosenInputs = static_cast<int>(info.channel_count);
            }
        }
    }

    if (numOutputPorts > 0) {
        // Check if requested matches any output port
        bool foundMatch = false;
        for (uint32_t i = 0; i < numOutputPorts; ++i) {
            clap_audio_port_info info = {};
            if (_audioPortsExt->get(_plugin, i, false, &info)) {
                if (info.channel_count == static_cast<uint32_t>(requestedOutputs)) {
                    chosenOutputs = requestedOutputs;
                    foundMatch = true;
                    break;
                }
            }
        }
        // If no exact match, use first port's channel count (will be checked by caller)
        if (!foundMatch && numOutputPorts > 0) {
            clap_audio_port_info info = {};
            if (_audioPortsExt->get(_plugin, 0, false, &info)) {
                chosenOutputs = static_cast<int>(info.channel_count);
            }
        }
    }

    // Update descriptor with chosen configuration
    _descriptor.numAudioInputs = chosenInputs;
    _descriptor.numAudioOutputs = chosenOutputs;

    // Log negotiation result
    if (chosenInputs != requestedInputs || chosenOutputs != requestedOutputs) {
        std::ostringstream msg;
        msg << "Plugin I/O negotiation: requested " << requestedInputs << "/" << requestedOutputs
            << ", chosen " << chosenInputs << "/" << chosenOutputs;
        LOG_DEBUG({"ClapPluginInstance"}, msg.str());
    }
}

bool ClapPluginInstance::negotiateAudioIO(int requestedInputs, int requestedOutputs) {
    // Query CLAP audio ports and negotiate I/O configuration
    queryAudioPorts(requestedInputs, requestedOutputs);

    // Return true if negotiation query succeeded (descriptor updated)
    // The caller (PluginNode) will check if negotiated matches requested
    // This function just ensures we queried the plugin successfully
    return true;
}

void ClapPluginInstance::prepare(double sampleRate, int maxBlockSize) {
    _sampleRate = sampleRate;
    _maxBlockSize = maxBlockSize;

    if (_prepared) {
        return;
    }

    if (!_plugin) {
        LOG_ERROR({"ClapPluginInstance"}, "Cannot prepare: plugin not created");
        return;
    }

    // Prepare audio buffers
    _audioBuffers.inputChannels.clear();
    _audioBuffers.outputChannels.clear();
    _audioBuffers.inputBuffers.clear();
    _audioBuffers.outputBuffers.clear();

    // TODO: Query actual port configuration from plugin
    // For now, assume stereo I/O
    const int numInputChannels = 2;
    const int numOutputChannels = 2;

    _audioBuffers.inputChannels.resize(numInputChannels, nullptr);
    _audioBuffers.outputChannels.resize(numOutputChannels, nullptr);
    _audioBuffers.inputBuffers.resize(1);
    _audioBuffers.outputBuffers.resize(1);

    _audioBuffers.inputBuffers[0].channel_count = numInputChannels;
    _audioBuffers.inputBuffers[0].latency = 0;
    _audioBuffers.inputBuffers[0].constant_mask = 0;
    // CLAP SDK expects float** (non-const), but input buffers are logically const
    // Safe to const_cast since plugin won't modify input data
    _audioBuffers.inputBuffers[0].data32 = const_cast<float**>(reinterpret_cast<const float* const*>(_audioBuffers.inputChannels.data()));
    _audioBuffers.inputBuffers[0].data64 = nullptr;

    _audioBuffers.outputBuffers[0].channel_count = numOutputChannels;
    _audioBuffers.outputBuffers[0].latency = 0;
    _audioBuffers.outputBuffers[0].constant_mask = 0;
    _audioBuffers.outputBuffers[0].data32 = _audioBuffers.outputChannels.data();
    _audioBuffers.outputBuffers[0].data64 = nullptr;

    // Prepare MIDI events
    _midiEvents.events.clear();
    _midiEvents.events.reserve(128); // Pre-allocate for typical MIDI block

    // Set up input events callbacks (using static functions)
    _midiEvents.inputEvents.size = inputEventsSize;
    _midiEvents.inputEvents.get = inputEventsGet;

    // Set up output events callbacks (using static functions)
    // Official CLAP SDK only has try_push, not push
    _midiEvents.outputEvents.try_push = outputEventsTryPush;

    _prepared = true;

    // Activate plugin
    activatePlugin();

    std::ostringstream msg;
    msg << "Prepared plugin: " << _descriptor.name
        << " (sampleRate: " << sampleRate << ", blockSize: " << maxBlockSize << ")";
    LOG_DEBUG({"ClapPluginInstance"}, msg.str());
}

void ClapPluginInstance::activatePlugin() {
    if (_active || !_prepared || !_plugin) {
        return;
    }

    if (!_plugin->activate) {
        _active = true;
        return;
    }

    // Activate with sample rate and block size
    if (_plugin->activate(_plugin, _sampleRate, 32, _maxBlockSize)) {
        _active = true;

        if (_plugin->start_processing) {
            _plugin->start_processing(_plugin);
        }
    } else {
        LOG_ERROR({"ClapPluginInstance"}, "Plugin activation failed");
    }
}

void ClapPluginInstance::deactivatePlugin() {
    if (!_active || !_plugin) {
        return;
    }

    if (_plugin->stop_processing) {
        _plugin->stop_processing(_plugin);
    }

    if (_plugin->deactivate) {
        _plugin->deactivate(_plugin);
    }

    _active = false;
}

void ClapPluginInstance::reset() {
    if (!_plugin) {
        return;
    }

    if (_plugin->reset) {
        _plugin->reset(_plugin);
    }

    // Reset parameter values to defaults
    for (auto& param : _parameters) {
        param.currentValue = (param.defaultValue - param.minValue) /
                            (param.maxValue - param.minValue);
    }
}

void ClapPluginInstance::processAudioMidi(
    AudioBuffer& audioIn,
    AudioBuffer& audioOut,
    MidiBuffer& midiIn,
    MidiBuffer& midiOut,
    const NodeProcessContext& ctx
) {
    if (!_prepared || !_active || !_plugin || !_plugin->process) {
        // Fallback: pass-through
        audioOut.copyFrom(audioIn);
        midiOut.clear();
        midiOut.append(midiIn);
        return;
    }

    // Update time-info from context (Phase 8)
    updateTimeInfo(ctx);

    // Convert MIDI to CLAP events
    convertMidiToClap(midiIn);

    // Set up audio buffer pointers
    const int numInputChannels = std::min(audioIn.numChannels(), static_cast<int>(_audioBuffers.inputChannels.size()));
    const int numOutputChannels = std::min(audioOut.numChannels(), static_cast<int>(_audioBuffers.outputChannels.size()));
    const int numFrames = std::min(audioIn.numFrames(), audioOut.numFrames());

    for (int ch = 0; ch < numInputChannels; ++ch) {
        _audioBuffers.inputChannels[ch] = audioIn.getChannelData(ch);
    }
    for (int ch = 0; ch < numOutputChannels; ++ch) {
        _audioBuffers.outputChannels[ch] = audioOut.getChannelData(ch);
    }

    // Update buffer metadata
    _audioBuffers.inputBuffers[0].channel_count = numInputChannels;
    _audioBuffers.outputBuffers[0].channel_count = numOutputChannels;

    // Build CLAP process structure
    clap_process process = {};
    process.steady_time = static_cast<uint32_t>(ctx.blockStartSample);
    process.frames_count = static_cast<uint32_t>(numFrames);
    process.audio_inputs = _audioBuffers.inputBuffers.data();
    process.audio_inputs_count = numInputChannels > 0 ? 1 : 0;
    process.audio_outputs = _audioBuffers.outputBuffers.data();
    process.audio_outputs_count = numOutputChannels > 0 ? 1 : 0;
    process.in_events = &_midiEvents.inputEvents;
    process.out_events = &_midiEvents.outputEvents;

    // Store instance pointer in thread-local storage for callbacks
    // This allows static callbacks to access the instance
    static thread_local ClapPluginInstance* currentInstance = nullptr;
    ClapPluginInstance* prevInstance = currentInstance;
    currentInstance = this;

    // Process plugin
    clap_process_status status = _plugin->process(_plugin, &process);

    // Restore previous instance (if any)
    currentInstance = prevInstance;

    // Restore previous instance (if any)
    currentInstance = prevInstance;

    // Handle process status
    if (status == CLAP_PROCESS_ERROR) {
        // Error - fallback to pass-through
        audioOut.copyFrom(audioIn);
        midiOut.clear();
        midiOut.append(midiIn);
    } else {
        // Success - convert CLAP MIDI output back
        convertClapToMidi(midiOut);
    }
}

void ClapPluginInstance::convertMidiToClap(const MidiBuffer& midiIn) {
    _midiEvents.events.clear();

    for (const auto& msg : midiIn.getMessages()) {
        clap_event_midi event = {};
        event.header.size = sizeof(clap_event_midi);
        event.header.space_id = 0; // CLAP_CORE_EVENT_SPACE
        event.header.type = CLAP_EVENT_MIDI;
        event.header.flags = 0;
        event.header.time = static_cast<uint32_t>(msg.sampleOffset);
        event.port_index = 0;

        // Convert MIDI message to CLAP format
        if (msg.status < 0xF0) { // Channel message
            event.data[0] = msg.status;
            event.data[1] = msg.data1;
            event.data[2] = msg.data2;
            _midiEvents.events.push_back(event);
        }
    }
}

void ClapPluginInstance::convertClapToMidi(MidiBuffer& midiOut) {
    // Phase 5: CLAP output events are collected but not yet converted back to MIDI
    // This will be implemented when we need plugin MIDI output
    midiOut.clear();
}

int ClapPluginInstance::getNumParameters() const {
    return static_cast<int>(_parameters.size());
}

std::string ClapPluginInstance::getParameterId(int index) const {
    if (index >= 0 && index < static_cast<int>(_parameters.size())) {
        return _parameters[index].id;
    }
    return "";
}

float ClapPluginInstance::getParameterValue(const std::string& paramId) const {
    auto it = _paramIdToIndex.find(paramId);
    if (it != _paramIdToIndex.end() && it->second < _parameters.size()) {
        return static_cast<float>(_parameters[it->second].currentValue);
    }
    return 0.0f;
}

std::vector<PluginParameterDescriptor> ClapPluginInstance::listParameterDescriptors() const {
    std::vector<PluginParameterDescriptor> descriptors;
    descriptors.reserve(_parameters.size());

    for (const auto& param : _parameters) {
        PluginParameterDescriptor descriptor;
        descriptor.paramId = param.id;
        descriptor.name = param.name;
        descriptor.unit = param.unit;
        descriptor.minValue = static_cast<float>(param.minValue);
        descriptor.maxValue = static_cast<float>(param.maxValue);
        descriptor.defaultValue = static_cast<float>(param.defaultValue);
        descriptor.step = static_cast<float>(param.step);
        descriptor.isAutomatable = param.isAutomatable;
        descriptor.isBypass = param.isBypass;
        descriptors.push_back(std::move(descriptor));
    }

    return descriptors;
}

void ClapPluginInstance::setParameterValue(const std::string& paramId, float normalisedValue) {
    // Clamp to [0.0, 1.0]
    normalisedValue = std::max(0.0f, std::min(1.0f, normalisedValue));

    auto it = _paramIdToIndex.find(paramId);
    if (it == _paramIdToIndex.end() || it->second >= _parameters.size()) {
        return;
    }

    auto& param = _parameters[it->second];
    param.currentValue = static_cast<double>(normalisedValue);

    // Convert normalised value to CLAP value
    double clapValue = param.minValue + normalisedValue * (param.maxValue - param.minValue);

    // Set parameter via CLAP extension
    if (_paramsExt && _paramsExt->get_value) {
        // Note: CLAP parameter setting is typically done via events, not direct set
        // For Phase 5, we'll cache the value and flush during process
        // TODO: Implement proper CLAP parameter event handling
    }
}

std::vector<uint8_t> ClapPluginInstance::getStateChunk() const {
    if (!_stateExt || !_stateExt->save) {
        return std::vector<uint8_t>();
    }

    // Phase 5: State saving requires implementing clap_ostream
    // For now, return empty (will be implemented in future phase)
    return std::vector<uint8_t>();
}

void ClapPluginInstance::setStateChunk(const std::vector<uint8_t>& data) {
    if (!_stateExt || !_stateExt->load || data.empty()) {
        return;
    }

    // Phase 5: State loading requires implementing clap_istream
    // For now, no-op (will be implemented in future phase)
    (void)data;
}

// CLAP host callbacks
void ClapPluginInstance::hostRequestRestart(const clap_host* host) {
    // Phase 5: Request restart - log for now
    LOG_DEBUG({"ClapPluginInstance"}, "Plugin requested restart");
}

void ClapPluginInstance::hostRequestProcess(const clap_host* host) {
    // Phase 5: Request process - not needed for our architecture
}

void ClapPluginInstance::hostRequestCallback(const clap_host* host) {
    // Phase 5: Request callback - schedule for main thread
    LOG_DEBUG({"ClapPluginInstance"}, "Plugin requested callback");
}

const void* ClapPluginInstance::hostGetExtension(const clap_host* host, const char* extension_id) {
    // Phase 8: Support CLAP_EXT_TIMELINE for time-info
    // For Phase 8, we provide minimal time-info support
    // Full timeline extension implementation will come later
    (void)host;
    (void)extension_id;
    return nullptr;
}

void ClapPluginInstance::updateTimeInfo(const NodeProcessContext& ctx) {
    // Update musical time info from processing context (Phase 8)
    // For Phase 8, use simple tempo and position
    // Future: integrate with full tempo map from Pulse
    _currentTimeInfo.tempo = ctx.tempo;
    _currentTimeInfo.timeSigNumerator = 4; // Default 4/4 for Phase 8
    _currentTimeInfo.timeSigDenominator = 4;

    // Convert sample position to beats: beats = (samples / sampleRate) * tempo / 60
    double positionSeconds = static_cast<double>(ctx.blockStartSample) / static_cast<double>(ctx.sampleRate);
    _currentTimeInfo.songPosSeconds = positionSeconds;
    _currentTimeInfo.songPosBeats = (positionSeconds * ctx.tempo) / 60.0;

    _currentTimeInfo.playing = ctx.isPlaying;
    _currentTimeInfo.loopEnabled = ctx.loopEnabled;
    _currentTimeInfo.loopStartBeats = ctx.loopStartBeats;
    _currentTimeInfo.loopEndBeats = ctx.loopEndBeats;
}

uint32_t ClapPluginInstance::inputEventsSize(const clap_input_events* events) {
    auto* midiEvents = static_cast<ClapMidiEvents*>(events->ctx);
    if (!midiEvents) {
        return 0;
    }
    return static_cast<uint32_t>(midiEvents->events.size());
}

const clap_event_header_t* ClapPluginInstance::inputEventsGet(const clap_input_events* events, uint32_t index) {
    auto* midiEvents = static_cast<ClapMidiEvents*>(events->ctx);
    if (!midiEvents || index >= midiEvents->events.size()) {
        return nullptr;
    }
    return reinterpret_cast<const clap_event_header_t*>(&midiEvents->events[index]);
}

bool ClapPluginInstance::outputEventsTryPush(const clap_output_events* events, const clap_event_header_t* header) {
    // For Phase 5, we'll collect output events but not process them yet
    (void)events;
    (void)header;
    return true;
}

// Factory function
std::unique_ptr<PluginInstance> createClapInstance(
    std::shared_ptr<ClapPluginLibrary> library,
    const clap_plugin_descriptor_t* clapDesc
) {
    try {
        return std::make_unique<ClapPluginInstance>(library, clapDesc);
    } catch (const std::exception& e) {
        LOG_ERROR({"ClapPluginInstance"}, std::string("Failed to create CLAP instance: ") + e.what());
        return nullptr;
    }
}
