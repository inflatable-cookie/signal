#include "core/EngineHost.hpp"
#include "backend/AudioBackend.hpp"
#include "backend/MiniaudioBackend.hpp"
#include "backend/AudioBackendConfig.hpp"
#include "backend/OutputDeviceInfo.hpp"
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
#include <iomanip>
#include <string_view>

namespace {

std::string toHexByte(unsigned char value) {
    const char* digits = "0123456789abcdef";
    std::string out;
    out.push_back(digits[(value >> 4) & 0x0F]);
    out.push_back(digits[value & 0x0F]);
    return out;
}

std::string formatUuid(const libremidi::uuid& value) {
    std::string out;
    out.reserve(32);

    for (auto byte : value.bytes) {
        out += toHexByte(byte);
    }

    return out;
}

std::string formatUsbDeviceId(const libremidi::usb_device_identifier& value) {
    std::ostringstream out;
    out << "usb:" << std::hex << std::setw(4) << std::setfill('0')
        << value.vendor_id << ":" << std::setw(4) << value.product_id;
    return out.str();
}

std::string sanitizeSegment(std::string value) {
    for (auto& ch : value) {
        if (ch == ';' || ch == '=' || ch == '|') {
            ch = '_';
        }
    }

    return value;
}

std::uint64_t fnv1a64(std::string_view input) {
    std::uint64_t hash = 14695981039346656037ull;

    for (unsigned char byte : input) {
        hash ^= static_cast<std::uint64_t>(byte);
        hash *= 1099511628211ull;
    }

    return hash;
}

std::string formatHex64(std::uint64_t value) {
    std::ostringstream out;
    out << std::hex << std::setw(16) << std::setfill('0') << value;
    return out.str();
}

template <typename Variant>
std::string formatVariantIdentifier(const Variant& value) {
    struct VariantFormatter {
        std::string operator()(libremidi::monostate) const {
            return "";
        }

        std::string operator()(const std::string& v) const {
            return v;
        }

        std::string operator()(std::uint64_t v) const {
            return std::to_string(v);
        }

        std::string operator()(const libremidi::uuid& v) const {
            return formatUuid(v);
        }

        std::string operator()(const libremidi::usb_device_identifier& v) const {
            return formatUsbDeviceId(v);
        }

        template <typename T>
        std::string operator()(const T&) const {
            return "";
        }
    };

    return libremidi::visit(
        VariantFormatter{},
        value
    );
}

std::string buildStableMidiDeviceId(const libremidi::input_port& port) {
    std::ostringstream out;
    auto api_name = libremidi::get_api_name(port.api);
    out << "libremidi:" << api_name;

    auto container_id = formatVariantIdentifier(port.container);

    if (!container_id.empty()) {
        out << ";c=" << sanitizeSegment(container_id);
    }

    auto device_id = formatVariantIdentifier(port.device);

    if (!device_id.empty()) {
        out << ";d=" << sanitizeSegment(device_id);
    }

    if (!port.manufacturer.empty()) {
        out << ";m=" << sanitizeSegment(port.manufacturer);
    }

    if (!port.product.empty()) {
        out << ";prod=" << sanitizeSegment(port.product);
    }

    if (!port.serial.empty()) {
        out << ";sn=" << sanitizeSegment(port.serial);
    }

    if (port.port != static_cast<libremidi::port_handle>(-1)) {
        out << ";p=" << port.port;
    }

    std::string name = port.display_name;

    if (name.empty()) {
        name = port.port_name;
    }

    if (name.empty()) {
        name = port.device_name;
    }

    if (!name.empty()) {
        out << ";n=" << sanitizeSegment(name);
    }

    std::ostringstream fallback_key;
    fallback_key << api_name << "|" << port.manufacturer << "|" << port.product
                 << "|" << port.serial << "|" << port.display_name << "|" << port.port_name
                 << "|" << port.device_name;
    auto fallback_hash = fnv1a64(fallback_key.str());

    out << ";h=" << formatHex64(fallback_hash);

    return out.str();
}

std::string pickDeviceName(const libremidi::input_port& port, std::size_t fallbackIndex) {
    if (!port.display_name.empty()) {
        return port.display_name;
    }

    if (!port.port_name.empty()) {
        return port.port_name;
    }

    if (!port.device_name.empty()) {
        return port.device_name;
    }

    return "MIDI Input " + std::to_string(fallbackIndex + 1);
}

} // namespace

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
    LOG_INFO({"EngineHost"}, "Created");
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
            info.name = pickDeviceName(port, index);
            info.id = buildStableMidiDeviceId(port);
            info.manufacturer = port.manufacturer;
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
void EngineHost::renderBlock(
    EngineRenderContext& ctx,
    AudioBus& input,
    AudioBus& output
) {
    // Real-time safety: No allocations, locks, or I/O in this function

    // Read transport state snapshot once (lock-free)
    // Pointer remains valid for the entire renderBlock (previous snapshot kept alive)
    const TransportState* transport = getTransportSnapshot();

    // Update context with current playhead (needed for hasWorkToDo check)
    ctx.playheadSamples = _playheadSamples.load(std::memory_order_acquire);

    // Update transport/tempo info in context (Phase 8)
    if (transport) {
        ctx.tempo = transport->tempo;
        ctx.isPlaying = transport->isPlaying;
        ctx.loopEnabled = transport->loopEnabled;

        // Convert loop region from samples/seconds to beats if available
        // For Phase 8, use simple tempo conversion: beats = seconds * tempo / 60
        if (transport->loopEnabled && transport->loopRegion.has_value()) {
            const auto& loop = transport->loopRegion.value();
            ctx.loopStartBeats = (loop.startSeconds * transport->tempo) / 60.0;
            ctx.loopEndBeats = (loop.endSeconds * transport->tempo) / 60.0;
        } else {
            ctx.loopStartBeats = 0.0;
            ctx.loopEndBeats = 0.0;
        }
    } else {
        // Default values if transport not available
        ctx.tempo = 120.0;
        ctx.isPlaying = false;
        ctx.loopEnabled = false;
        ctx.loopStartBeats = 0.0;
        ctx.loopEndBeats = 0.0;
    }

    // Idle fast-path: skip processing if truly idle
    if (!hasWorkToDo(ctx)) {
        // Clear output and return early - no schedule, no tails, no live inputs
        output.clear();
        return;
    }

    // Update context with current playhead (already set above, but keeping for clarity)

    // Clear output buffer
    output.clear();

    // Step 1: Begin automation block evaluation (pre-computes all parameter values)
    _automationService->beginBlock(ctx.playheadSamples, ctx.blockSize, ctx.sampleRate);

    // Step 2: Apply automation values to nodes and services
    // Real-time safe: no allocations, just value lookups and assignments
    const auto& executionOrder = _graphEngine->getExecutionOrder();
    for (GraphNode* node : executionOrder) {
        if (!node) continue;

        std::string nodeId = node->getId();

        // Apply fader automation (gain/spatial.balance)
        if (node->getKind() == NodeKind::Fader) {
            auto* faderNode = dynamic_cast<FaderNode*>(node);
            if (faderNode) {
                // Use node ID as automation target for fader parameters
                const std::string& targetId = node->getId();

                float gain = _automationService->getParameterValue(targetId, "gain");
                float balance = _automationService->getParameterValue(targetId, "spatial.balance");

                faderNode->setGain(gain);
                faderNode->setPan(balance);
            }
        }

        // Apply send level automation
        if (node->getKind() == NodeKind::Send) {
            auto* sendNode = dynamic_cast<SendNode*>(node);
            if (sendNode) {
                const std::string& targetId = node->getId();
                float sendLevel = _automationService->getParameterValue(targetId, "send-level");
                sendNode->setSendLevel(sendLevel);
            }
        }

        // Plugin parameter automation is applied via applyParameterChanges mechanism
        // AutomationService values are pushed into that queue from control thread
        // This keeps plugin parameter updates synchronized with other parameter changes
    }

    // Phase 4: Apply pending parameter changes (lock-free swap)
    if (_parameterChangesPending.load(std::memory_order_acquire)) {
        // Swap pending changes to active (control thread writes, audio thread reads)
        _activeParameterChanges.clear();
        _activeParameterChanges.swap(_pendingParameterChanges);
        _parameterChangesPending.store(false, std::memory_order_release);

        // Apply parameter changes to plugin nodes
        for (const auto& change : _activeParameterChanges) {
            GraphNode* node = _graphEngine->findNode(change.nodeId);
            if (!node) {
                continue;
            }

            // Check if node has a plugin
            PluginInstance* plugin = nullptr;
            if (
                node->getKind() == NodeKind::MidiFx ||
                node->getKind() == NodeKind::Instrument ||
                node->getKind() == NodeKind::AudioFx
            ) {
                auto* pluginNode = dynamic_cast<PluginNode*>(node);
                if (pluginNode) {
                    plugin = pluginNode->getPlugin();
                }
            }

            if (plugin) {
                plugin->setParameterValue(change.paramId, change.normalisedValue);
            }
        }
    }

    // Step 2: Clear all node buffers (prepares for Source/Input Pass and processing)
    // This is done here so buffers are cleared before the Source/Input Pass populates outputs
    for (GraphNode* node : executionOrder) {
        if (node) {
            node->io.audioIn.clear();
            node->io.midiIn.clear();
            node->io.audioOut.clear();
            node->io.midiOut.clear();
        }
    }

    // Step 3: Source/Input Pass - inject schedule data and hardware input
    // This unified pass populates all source and input node outputs before processing
    // Real-time safe: no allocations, no locks, no logging
    std::vector<MidiMessage> hardwareMidiInput; // TODO: Get from MIDI backend when implemented
    _graphEngine->runSourceInputPass(
        ctx,
        _streamScheduler.get(),
        _audioAssetSource.get(),
        input.data(),
        input.numChannels(),
        input.numFrames(),
        hardwareMidiInput
    );

    // Phase 7: Capture from input nodes if recording is active
    if (_recordingSession->isRecording()) {
        uint64_t blockStartSamples = ctx.playheadSamples;

        for (GraphNode* node : executionOrder) {
            if (node && node->getKind() == NodeKind::HardwareAudioInput) {
                auto* inputNode = dynamic_cast<AudioInputNode*>(node);
                if (inputNode) {
                    std::string laneId = _recordingSession->getTargetLaneForInput(inputNode->getId());
                    if (!laneId.empty() && _recordingSession->isLaneArmed(laneId)) {
                        // Capture audio from this input node
                        const auto& audioOut = inputNode->io.audioOut;
                        if (audioOut.numChannels() > 0 && audioOut.numFrames() > 0) {
                            RecordedAudioChunk chunk;
                            chunk.laneId = laneId;
                            chunk.numChannels = audioOut.numChannels();
                            chunk.sampleRate = static_cast<int>(ctx.sampleRate);
                            chunk.startSample = blockStartSamples;
                            chunk.provisionalAssetId = "temp-" + inputNode->getId() + "-" + std::to_string(blockStartSamples);

                            // Convert deinterleaved AudioBuffer to interleaved format
                            int numFrames = audioOut.numFrames();
                            chunk.interleaved.resize(chunk.numChannels * numFrames);
                            audioOut.copyToInterleaved(chunk.interleaved.data(), chunk.numChannels, numFrames);

                            _recordingSession->captureAudioChunk(chunk);
                        }
                    }
                }
            } else if (node && node->getKind() == NodeKind::HardwareMidiInput) {
                auto* midiInputNode = dynamic_cast<MidiInputNode*>(node);
                if (midiInputNode) {
                    std::string laneId = _recordingSession->getTargetLaneForInput(midiInputNode->getId());
                    if (!laneId.empty() && _recordingSession->isLaneArmed(laneId)) {
                        // Capture MIDI from this input node
                        const auto& midiOut = midiInputNode->io.midiOut;
                        if (midiOut.size() > 0) {
                            RecordedMidiChunk chunk;
                            chunk.laneId = laneId;
                            chunk.startSample = blockStartSamples;

                            // Convert MidiBuffer to RecordedMidiEvent
                            const auto& messages = midiOut.getMessages();
                            for (const auto& msg : messages) {
                                RecordedMidiEvent event;
                                event.timeSamples = blockStartSamples + msg.sampleOffset;
                                event.status = msg.status;
                                event.data1 = msg.data1;
                                event.data2 = msg.data2;
                                event.channel = msg.channel;
                                chunk.events.push_back(event);
                            }

                            _recordingSession->captureMidiChunk(chunk);
                        }
                    }
                }
            }
        }
    }

    // Step 4: Process graph (routing, plugin processing)
    // Note: Source/Input Pass was already called in Step 3, so nodes are ready to process
    _graphEngine->processGraph(ctx);

    // Step 3: Find hardware audio output node and mix to host output bus
    GraphNode* outputNode = nullptr;
    const std::string* preferredOutputNodeId = _activeOutputNodeId.load(std::memory_order_acquire);
    const auto& executionOrderAfterGraph = _graphEngine->getExecutionOrder();

    for (GraphNode* node : executionOrderAfterGraph) {
        if (node && node->getKind() == NodeKind::HardwareAudioOutput) {
            if (
                preferredOutputNodeId &&
                !preferredOutputNodeId->empty() &&
                node->getId() == *preferredOutputNodeId
            ) {
                outputNode = node;
                break;
            }

            if (!outputNode) {
                outputNode = node;
            }
        }
    }

    if (outputNode) {
        const std::string* outputMixId = _activeOutputMixId.load(std::memory_order_acquire);
        const std::string& mixId = outputMixId ? *outputMixId : outputNode->getId();

        // Step 4: Mix output node into host output bus.
        //
        // Note: Mute/gain/spatial.balance are owned by nodes in the graph (e.g. FaderNode),
        // so EngineHost can do a straightforward copy/format conversion here.
        const int numChannels = output.numChannels();
        const int numFrames = std::min(output.numFrames(), outputNode->io.audioOut.numFrames());

        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels; ++ch) {
                const float* inChannel = outputNode->io.audioOut.getChannelData(ch);

                if (!inChannel) {
                    output.setSample(frame, ch, 0.0f);
                    continue;
                }

                output.setSample(frame, ch, inChannel[frame]);
            }
        }

        if (numFrames < output.numFrames()) {
            for (int frame = numFrames; frame < output.numFrames(); ++frame) {
                for (int ch = 0; ch < numChannels; ++ch) {
                    output.setSample(frame, ch, 0.0f);
                }
            }
        }

        // Step 5: Capture metering levels from final mixed output
        // Real-time safe: submitSampleBlock is lock-free (uses shared_lock for map lookup only)
        _meteringService->submitSampleBlock(
            mixId,
            output.data(),
            output.numChannels(),
            output.numFrames()
        );

        // Step 6: Capture final output for recording (if recording is active)
        if (_recordingSession->isRecording()) {
            _recordingSession->captureFinalOutput(output, ctx.playheadSamples, mixId);
        }
    } else {
        // No hardware output node - output will be silence
        output.clear();
    }

    // Diagnostic: Check output level
    float maxOutput = 0.0f;
    bool hasOutput = false;
    const int numChannels = output.numChannels();
    const int numFrames = output.numFrames();
    for (int frame = 0; frame < numFrames; ++frame) {
        for (int ch = 0; ch < numChannels; ++ch) {
            float absSample = std::abs(output.sample(frame, ch));
            if (absSample > maxOutput) {
                maxOutput = absSample;
            }
            if (absSample > 0.0001f) {
                hasOutput = true;
            }
        }
    }

    // Diagnostic logging: Periodic status (every ~1 second when playing, less frequent when stopped)
    uint64_t blockCount = _renderBlockCount.fetch_add(1, std::memory_order_acq_rel) + 1;
    uint64_t lastLog = _lastDebugLogBlock.load(std::memory_order_acquire);

    // Only log when playing, or occasionally when stopped (every ~10 seconds) to confirm engine is alive
    bool shouldLog = false;
    if (ctx.isPlaying) {
        // Log every ~1 second when playing
        shouldLog = (blockCount - lastLog >= DEBUG_LOG_INTERVAL_BLOCKS);
    } else {
        // Log every ~10 seconds when stopped (much less frequent)
        static constexpr uint32_t STOPPED_LOG_INTERVAL_BLOCKS = DEBUG_LOG_INTERVAL_BLOCKS * 10;
        shouldLog = (blockCount - lastLog >= STOPPED_LOG_INTERVAL_BLOCKS);
    }

    if (shouldLog) {
        _lastDebugLogBlock.store(blockCount, std::memory_order_release);

        // Log diagnostic info (non-real-time, but throttled)
        bool graphLoaded = _graphEngine->hasGraph();
        bool scheduleLoaded = _streamScheduler->hasSchedule();
        int activeStreamCount = _streamScheduler->getActiveStreamCount();
        // TODO: Future tail handling - check if any nodes have active tail (hasTailCurrently())
        //   Continue rendering even after schedule ends if tail is active

        // Format diagnostic message
        std::ostringstream diagMsg;
        diagMsg << "Block " << blockCount
                << ": playing=" << (ctx.isPlaying ? "yes" : "no")
                << ", playhead=" << ctx.playheadSamples
                << ", graph=" << (graphLoaded ? "yes" : "no")
                << ", schedule=" << (scheduleLoaded ? "yes" : "no")
                << ", activeStreams=" << activeStreamCount
                << ", maxOutput=" << maxOutput;
        LOG_DEBUG({"EngineHost", "Render"}, diagMsg.str());
    }

    // Diagnostic: Track consecutive silence
    if (hasOutput) {
        _consecutiveSilenceBlocks.store(0, std::memory_order_release);
    } else if (ctx.isPlaying) {
        uint64_t silenceCount = _consecutiveSilenceBlocks.fetch_add(1, std::memory_order_acq_rel) + 1;
        // Log warning if we've had silence for a while (e.g., 1 second = ~86 blocks at 44.1kHz/512)
        if (silenceCount == 86) {
            LOG_WARN({"EngineHost", "Render"}, "⚠ WARNING: Output still silence after 1 second of playback");
        }
    }

    // TODO: Phase 2 - Attach Stream Inputs & Minimal Audio/MIDI Flow:
    // - Get active audio segments per stream from StreamScheduler
    // - Load audio data from assets (per streamId)
    // - Feed streams into lane nodes using getStreamBindings()
    // - Process through node graph with real audio/MIDI buffers
    // - Apply automation per node/parameter
    // - Apply per-node mix controls (gain/spatial.balance/mute) via node parameters
    // - Loop handling
    // - Metering
    //
    // Architecture: Signal processes streams via node graph, not clips/channels.
    // Pulse compiles Tracks → Lanes → Streams and sends stream-based schedules.

    // For now, produce silence (nodes are processed but don't generate audio yet)
    // Uncomment the test tone code below to verify audio output:
    /*
    const float testToneFreq = 440.0f; // A4
    const float amplitude = 0.1f;
    float* outData = output.data();
    if (outData && output.numChannels() > 0) {
        for (int frame = 0; frame < output.numFrames(); ++frame) {
            float time = static_cast<float>(ctx.playheadSamples + frame) / static_cast<float>(ctx.sampleRate);
            float sample = amplitude * std::sin(2.0f * M_PI * testToneFreq * time);
            for (int ch = 0; ch < output.numChannels(); ++ch) {
                output.setSample(frame, ch, sample);
            }
        }
    }
    */

    // Update playhead for next block (only when playing)
    // When stopped, playhead is explicitly set by seek/play commands
    if (ctx.isPlaying) {
        uint64_t newPlayhead = ctx.playheadSamples + output.numFrames();
        _playheadSamples.store(newPlayhead, std::memory_order_release);
    }
}

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
