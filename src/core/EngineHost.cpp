#include "core/EngineHost.hpp"
#include "backend/AudioBackend.hpp"
#include "backend/MiniaudioBackend.hpp"
#include "backend/AudioBackendConfig.hpp"
#include "backend/OutputDeviceInfo.hpp"
#include "backend/MidiDeviceIdentity.hpp"
#include "core/MeteringService.hpp"
#include "core/AutomationService.hpp"
#include "core/StreamScheduler.hpp"
#include "core/GraphEngine.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "core/AudioAssetSource.hpp"
#include "core/PluginHost.hpp"
#include "core/GraphNodes.hpp"
#include "core/AutomationData.hpp"
#include "core/RecordingCapture.hpp"
#include "logging/Logging.hpp"
#include <libremidi/libremidi.hpp>
#include <iostream>
#include <memory>
#include <cstdint>
#include <chrono>
#include <cmath>
#include <cstring>
#include <algorithm>
#include <unordered_set>
#include <unordered_map>
#include <sstream>

EngineHost::EngineHost()
    : _state(State::Stopped)
    , _lastError(std::nullopt)
    , _shuttingDown(false)
    , _playheadSamples(0)
    , _graphLatencySamples(0)
    , _graphTailSamples(0)
{
    _meteringService = std::make_unique<MeteringService>();
    _automationService = std::make_unique<AutomationService>();
    _streamScheduler = std::make_unique<StreamScheduler>();
    _graphEngine = std::make_unique<GraphEngine>();
    _audioAssetSource = std::make_unique<AudioAssetSourceRouter>(); // Phase 12b.2: Router delegates to stub/file sources
    _pluginHost = std::make_unique<PluginHost>(); // Phase 4: Plugin host
    _recordingSession = std::make_unique<RecordingSession>(); // Phase 7: Recording session
    _parameterChangesPending.store(false, std::memory_order_release);

    // Initialize transport state with default values
    _transportState = std::make_shared<TransportState>();
    _activeTransport.store(_transportState.get(), std::memory_order_release);

    // Initialize automation data with empty snapshot
    _automationData = std::make_shared<AutomationData>(AutomationData::empty());
    _activeAutomation.store(_automationData.get(), std::memory_order_release);

    _outputMixIdA = "";
    _outputMixIdB = "";
    _activeOutputMixId.store(&_outputMixIdA, std::memory_order_release);

    _outputNodeIdA = "";
    _outputNodeIdB = "";
    _activeOutputNodeId.store(&_outputNodeIdA, std::memory_order_release);

#ifdef LOOPHOLE_ENABLE_AUDIO_DEBUG
    _renderBlockCount.store(0, std::memory_order_release);
    _lastDebugLogBlock.store(0, std::memory_order_release);
    _consecutiveSilenceBlocks.store(0, std::memory_order_release);
#endif

    setupAudioBackend();
    LOG_DEBUG({"EngineHost"}, "Created");
}

EngineHost::~EngineHost() {
    if (_state == State::Running || _state == State::Starting) {
        stop();
    }
    LOG_DEBUG({"EngineHost"}, "Destroyed");
}

void EngineHost::start() {
    if (_shuttingDown) {
        LOG_WARN({"EngineHost"}, "Cannot start: shutting down");
        return;
    }

    if (_state == State::Running) {
        LOG_DEBUG({"EngineHost"}, "Already running");
        return;
    }

    if (_state == State::Error) {
        LOG_WARN({"EngineHost"}, "Cannot start: in error state");
        return;
    }

    _state = State::Starting;
    clearError();

    // Start audio backend
    if (!_audioBackend) {
        setError("Audio backend not initialised");
        return;
    }

    if (!_audioBackend->start()) {
        setError("Failed to start audio backend");
        return;
    }

    // After audio starts successfully, transition to running
    _state = State::Running;
    LOG_INFO({"EngineHost"}, "Started");
}

void EngineHost::stop() {
    if (_state == State::Stopped) {
        LOG_DEBUG({"EngineHost"}, "Already stopped");
        return;
    }

    _state = State::Stopped;

    if (_audioBackend) {
        _audioBackend->stop();
    }

    LOG_INFO({"EngineHost"}, "Stopped");
}

void EngineHost::reset() {
    stop();
    clearError();

    // Reset transport state (create new snapshot)
    _transportState = std::make_shared<TransportState>();
    _activeTransport.store(_transportState.get(), std::memory_order_release);
    _previousTransport.reset();

    _playheadSamples.store(0, std::memory_order_release);
    _streamScheduler->clearSchedule();
    LOG_INFO({"EngineHost"}, "Reset");
}

void EngineHost::shutdown() {
    if (_shuttingDown) {
        return;
    }

    _shuttingDown = true;
    stop();
    LOG_INFO({"EngineHost"}, "Shutdown complete");
}

EngineHost::State EngineHost::state() const noexcept {
    return _state;
}

std::optional<std::string> EngineHost::lastError() const noexcept {
    return _lastError;
}

void EngineHost::setError(const std::string& error) {
    _state = State::Error;
    _lastError = error;
    LOG_ERROR({"EngineHost"}, std::string("Error: ") + error);
}

void EngineHost::clearError() {
    if (_state == State::Error) {
        _state = State::Stopped;
    }
    _lastError = std::nullopt;
}

TransportState& EngineHost::transport() {
    // Return mutable reference for control thread updates
    // Caller should call commitTransportUpdate() after making changes
    return *_transportState;
}

const TransportState& EngineHost::transport() const {
    return *_transportState;
}

const TransportState* EngineHost::getTransportSnapshot() const {
    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous snapshot kept alive in _previousTransport)
    return _activeTransport.load(std::memory_order_acquire);
}

// Helper method to commit transport updates (called after modifying transport())
void EngineHost::commitTransportUpdate() {
    // Create a new snapshot from current state (copy constructor)
    // At this point, _transportState points to the object that was just modified
    auto newSnapshot = std::make_shared<TransportState>(*_transportState);

    // Keep previous snapshot alive until next swap (ensures audio thread safety)
    _previousTransport = _transportState;

    // Atomically swap pointer (old snapshot kept alive in _previousTransport)
    _activeTransport.store(newSnapshot.get(), std::memory_order_release);

    // Update our mutable state pointer (now points to the new snapshot)
    // This ensures future calls to transport() return the new snapshot
    _transportState = newSnapshot;
}

const AutomationData* EngineHost::getAutomationSnapshot() const {
    // Read atomic pointer once (lock-free)
    // Pointer remains valid until next swap (previous snapshot kept alive in _previousAutomation)
    return _activeAutomation.load(std::memory_order_acquire);
}

RecordingSession& EngineHost::recordingSession() {
    return *_recordingSession;
}

const RecordingSession& EngineHost::recordingSession() const {
    return *_recordingSession;
}

void EngineHost::loadAutomationSnapshot(const AutomationData& snapshot) {
    // Delegate to AutomationService (consolidated automation system)
    _automationService->loadSnapshot(snapshot);

    // Maintain legacy AutomationData snapshot for diagnostic/testing callers
    // This mirrors the transport snapshot pattern used for TransportState.
    auto newSnapshot = std::make_shared<AutomationData>(snapshot);
    _previousAutomation = _automationData;
    _automationData = newSnapshot;
    _activeAutomation.store(_automationData.get(), std::memory_order_release);
}

double EngineHost::getCpuLoad() const {
    // Stub implementation - return 0.0 for now
    return 0.0;
}

uint64_t EngineHost::getXruns() const {
    // Stub implementation - return 0 for now
    return 0;
}

double EngineHost::getSampleRate() const {
    if (_audioBackend) {
        return _audioBackend->getSampleRate();
    }
    return SAMPLE_RATE;
}

size_t EngineHost::getBlockSize() const {
    if (_audioBackend) {
        return static_cast<size_t>(_audioBackend->getBufferSize());
    }
    return BLOCK_SIZE;
}

std::string EngineHost::getOutputDeviceName() const {
    if (_audioBackend) {
        return _audioBackend->getOutputDeviceName();
    }
    return "System Default";
}

int EngineHost::getNumOutputChannels() const {
    if (_audioBackend) {
        return _audioBackend->getNumOutputChannels();
    }
    return 2; // Default to stereo
}

std::string EngineHost::getActiveOutputDeviceId() const {
    if (_audioBackend) {
        // Try to cast to MiniaudioBackend to access device-specific methods
        auto* miniaudioBackend = dynamic_cast<MiniaudioBackend*>(_audioBackend.get());
        if (miniaudioBackend) {
            return miniaudioBackend->getActiveOutputDeviceId();
        }
    }
    return "";
}

std::vector<OutputDeviceInfo> EngineHost::enumerateOutputDevices() const {
    if (_audioBackend) {
        // Try to cast to MiniaudioBackend to access device-specific methods
        auto* miniaudioBackend = dynamic_cast<MiniaudioBackend*>(_audioBackend.get());
        if (miniaudioBackend) {
            return miniaudioBackend->enumerateOutputDevices();
        }
    }
    return {};
}

bool EngineHost::setOutputDevice(const std::string& deviceId) {
    if (_audioBackend) {
        // Try to cast to MiniaudioBackend to access device-specific methods
        auto* miniaudioBackend = dynamic_cast<MiniaudioBackend*>(_audioBackend.get());
        if (miniaudioBackend) {
            return miniaudioBackend->setOutputDevice(deviceId);
        }
    }
    return false;
}

std::vector<MidiInputDeviceInfo> EngineHost::enumerateMidiInputDevices() const {
    std::vector<MidiInputDeviceInfo> devices;

    try {
        libremidi::observer observer;
        auto ports = observer.get_input_ports();
        devices.reserve(ports.size());

        for (std::size_t index = 0; index < ports.size(); ++index) {
            const auto& port = ports[index];
            MidiInputDeviceInfo info;
            info.name = loophole::signal::midi::pickDeviceName(port, index);
            info.id = loophole::signal::midi::buildStableMidiDeviceId(port);
            info.manufacturer = port.manufacturer;
            info.api = libremidi::get_api_name(port.api);
            info.container_id = loophole::signal::midi::formatPortIdentifier(port.container);
            info.device_id = loophole::signal::midi::formatPortIdentifier(port.device);
            if (port.port != static_cast<libremidi::port_handle>(-1)) {
                info.port_handle = static_cast<std::uint64_t>(port.port);
            }
            info.port_name = port.port_name;
            info.device_name = port.device_name;
            info.display_name = port.display_name;
            info.product = port.product;
            info.serial = port.serial;
            info.is_connected = true;
            devices.push_back(std::move(info));
        }

    } catch (const std::exception& e) {
        LOG_WARN({"EngineHost"}, std::string("Failed to enumerate MIDI inputs: ") + e.what());
    }

    return devices;
}

MeteringService& EngineHost::metering() {
    return *_meteringService;
}

const MeteringService& EngineHost::metering() const {
    return *_meteringService;
}

PluginHost* EngineHost::pluginHost() {
    return _pluginHost.get();
}

const PluginHost* EngineHost::pluginHost() const {
    return _pluginHost.get();
}

AutomationService& EngineHost::automation() {
    return *_automationService;
}

const AutomationService& EngineHost::automation() const {
    return *_automationService;
}

StreamScheduler& EngineHost::streamScheduler() {
    return *_streamScheduler;
}

const StreamScheduler& EngineHost::streamScheduler() const {
    return *_streamScheduler;
}

GraphEngine& EngineHost::graphEngine() {
    return *_graphEngine;
}

const GraphEngine& EngineHost::graphEngine() const {
    return *_graphEngine;
}

void EngineHost::loadGraphSnapshot(const GraphSnapshot& snapshot) {
    _graphEngine->loadGraphSnapshot(snapshot, _pluginHost.get(), this);

    // Determine the active output node ID (HardwareAudioOutputNode) and the
    // output Channel identifier used for metering/recording identifiers.
    //
    // Prefer the default hardware output node (deviceIsDefault=true) and
    // prefer nodes that are fed by an explicit output Fader node in the same
    // Channel (Phase 10 output Channel/Fader work).
    std::string selectedOutputNodeId;
    std::string selectedOutputChannelId;
    bool selectedHasExplicitOutputFader = false;

    const auto* previousOutputNodeId = _activeOutputNodeId.load(std::memory_order_acquire);
    const auto* previousOutputMixId = _activeOutputMixId.load(std::memory_order_acquire);

    auto nodeById = [&snapshot](const std::string& id) -> const NodeDesc* {
        for (const auto& node : snapshot.nodes) {
            if (node.nodeId == id) {
                return &node;
            }
        }

        return nullptr;
    };

    for (const auto& node : snapshot.nodes) {
        if (node.kind != NodeKind::HardwareAudioOutput) {
            continue;
        }

        const bool isDefault = node.deviceIsDefault.value_or(false);
        const std::string channelId = node.channelId.value_or("");

        bool hasExplicitOutputFader = false;

        if (!channelId.empty()) {
            for (const auto& conn : snapshot.connections) {
                if (!conn.fromNodeId.has_value()) {
                    continue;
                }

                if (conn.toNodeId != node.nodeId) {
                    continue;
                }

                const auto* fromNode = nodeById(conn.fromNodeId.value());
                if (!fromNode) {
                    continue;
                }

                if (
                    fromNode->kind == NodeKind::Fader &&
                    fromNode->channelId.has_value() &&
                    fromNode->channelId.value() == channelId
                ) {
                    hasExplicitOutputFader = true;
                    break;
                }
            }
        }

        const bool preferNode =
            selectedOutputNodeId.empty() ||
            (!selectedHasExplicitOutputFader && hasExplicitOutputFader) ||
            (!selectedHasExplicitOutputFader && !hasExplicitOutputFader && isDefault);

        if (!preferNode) {
            continue;
        }

        selectedOutputNodeId = node.nodeId;
        selectedOutputChannelId = channelId;
        selectedHasExplicitOutputFader = hasExplicitOutputFader;
    }

    if (selectedOutputNodeId.empty() && previousOutputNodeId) {
        selectedOutputNodeId = *previousOutputNodeId;
    }

    if (selectedOutputChannelId.empty()) {
        if (previousOutputMixId && !previousOutputMixId->empty()) {
            selectedOutputChannelId = *previousOutputMixId;
        } else {
            selectedOutputChannelId = selectedOutputNodeId;
        }
    }

    const std::string* currentMix = _activeOutputMixId.load(std::memory_order_acquire);
    std::string* nextMix = (currentMix == &_outputMixIdA) ? &_outputMixIdB : &_outputMixIdA;
    *nextMix = selectedOutputChannelId;
    _activeOutputMixId.store(nextMix, std::memory_order_release);

    const std::string* currentNode = _activeOutputNodeId.load(std::memory_order_acquire);
    std::string* nextNode = (currentNode == &_outputNodeIdA) ? &_outputNodeIdB : &_outputNodeIdA;
    *nextNode = selectedOutputNodeId;
    _activeOutputNodeId.store(nextNode, std::memory_order_release);

    if (!nextMix->empty()) {
        _meteringService->registerChannel(*nextMix);
    }

    // Update graph latency and tail metrics
    // These are computed from the graph structure and cached for audio thread access
    int totalLatency = _graphEngine->getTotalLatencyInSamples();
    int maxTail = _graphEngine->getMaxTailInSamples();
    _graphLatencySamples.store(totalLatency, std::memory_order_release);
    _graphTailSamples.store(maxTail, std::memory_order_release);

    // Log graph latency/tail info (once per graph update, not per block)
    std::ostringstream msg;
    msg << "Graph latency=" << totalLatency << " samples, tail=" << maxTail << " samples";
    LOG_INFO({"EngineHost", "Latency"}, msg.str());

    // TODO: Future latency compensation work:
    //   - Adjust playhead offset by totalLatency when computing musical position
    //   - Adjust schedule lookahead to account for latency
    //   - Extend rendering beyond schedule end by maxTail when stopping playback

    // Prepare graph if engine is already running
    if (_state == State::Running) {
        prepareEngine(
            static_cast<int>(getSampleRate()),
            static_cast<size_t>(getBlockSize())
        );
    }
}

void EngineHost::prepareEngine(int sampleRate, int maxBlockSize) {
    _graphEngine->prepareGraph(sampleRate, maxBlockSize);

    // Update asset source router sample rate for tone generation
    if (_audioAssetSource) {
        auto* router = dynamic_cast<AudioAssetSourceRouter*>(_audioAssetSource.get());
        if (router) {
            router->setSampleRate(static_cast<double>(sampleRate));
        }
    }

    // TODO: Also prepare plugins, allocate buffers, etc. in future phases
}

void EngineHost::prepareAudioAsset(const std::string& assetId, const std::string& path, uint32_t channels, uint32_t sampleRate, uint64_t frames) {
    if (!_audioAssetSource) {
        LOG_ERROR({"EngineHost"}, "Cannot prepare asset: asset source is null");
        return;
    }

    auto* router = dynamic_cast<AudioAssetSourceRouter*>(_audioAssetSource.get());
    if (!router) {
        LOG_ERROR({"EngineHost"}, "Cannot prepare asset: asset source is not a router");
        return;
    }

    AudioAssetMetadata metadata;
    metadata.path = path;
    metadata.channels = channels;
    metadata.sampleRate = sampleRate;
    metadata.frames = frames;

    if (router->prepareAsset(assetId, metadata)) {
        LOG_INFO({"EngineHost"}, std::string("Prepared audio asset: ") + assetId);
    } else {
        LOG_ERROR({"EngineHost"}, std::string("Failed to prepare audio asset: ") + assetId);
    }
}

uint64_t EngineHost::getPlayheadSamples() const noexcept {
    return _playheadSamples.load(std::memory_order_acquire);
}

void EngineHost::setPlayheadSamples(uint64_t samples) noexcept {
    _playheadSamples.store(samples, std::memory_order_release);
}

void EngineHost::applyParameterChanges(const std::vector<ParameterChange>& changes) {
    // Called on control thread - queue changes for audio thread
    _pendingParameterChanges.insert(_pendingParameterChanges.end(), changes.begin(), changes.end());
    _parameterChangesPending.store(true, std::memory_order_release);
}

void EngineHost::setupAudioBackend() {
    // Create MiniaudioBackend (placeholder implementation)
    _audioBackend = std::make_unique<MiniaudioBackend>();

    // Configure backend
    AudioBackendConfig config;
    config.preferredSampleRate = SAMPLE_RATE;
    config.preferredBufferSize = static_cast<int>(BLOCK_SIZE);
    config.numInputChannels = 0;   // No input for now
    config.numOutputChannels = 2;  // Stereo output

    if (!_audioBackend->initialise(config)) {
        LOG_ERROR({"EngineHost"}, "Failed to initialise audio backend");
        _audioBackend.reset();
        return;
    }

    // Set render callback to call renderBlock
    _audioBackend->setRenderCallback([this](
        EngineRenderContext& ctx,
        AudioBus& input,
        AudioBus& output
    ) {
        this->renderBlock(ctx, input, output);
    });

    LOG_INFO({"EngineHost"}, "Audio backend configured");
}

/// High-level render sequence (per block):
/// 1. Sync transport and context
/// 2. Clear node buffers
/// 3. Source/Input Pass (schedule + hardware injection)
/// 4. Automation update
/// 5. Graph processing (nodes)
/// 6. Mixer final mix
/// 7. Metering capture
/// 8. Recording capture
/// 9. Diagnostics + playhead update
bool EngineHost::hasWorkToDo(const EngineRenderContext& ctx) const noexcept {
    // Real-time safe: read-only, no allocations or locks
    // Check all conditions that require full processing

    // 1. Transport is playing
    if (ctx.isPlaying) {
        return true;
    }

    // 2. Schedule has active streams at current position
    if (_streamScheduler->hasActiveStreams(ctx.playheadSamples)) {
        return true;
    }

    // 3. Graph has active tails (plugins still producing output after playback stops)
    if (_graphEngine->hasActiveTails()) {
        return true;
    }

    // 4. Graph has live inputs/monitors (AudioInput, MidiInput, or instrument with live MIDI)
    if (_graphEngine->hasLiveInputsOrMonitors()) {
        return true;
    }

    // All conditions false - truly idle, can use fast-path
    return false;
}
