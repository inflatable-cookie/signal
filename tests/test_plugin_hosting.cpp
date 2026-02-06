#include <catch2/catch_test_macros.hpp>
#include "core/PluginInstance.hpp"
#include "core/PluginHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphSnapshot.hpp"
#include "core/GraphNodes.hpp"
#include "core/EngineHost.hpp"
#include "core/ParameterChange.hpp"
#include "core/NodeBuffers.hpp"
#include "core/NodeProcessContext.hpp"
#include <memory>
#include <vector>
#include <string>
#include <cstring>
#include <algorithm>

/// Test plugin implementation for testing
class TestPluginInstance : public PluginInstance {
public:
    TestPluginInstance(const PluginDescriptor& desc)
        : _descriptor(desc)
        , _gain(1.0f)
        , _prepared(false)
    {
        // Add a "gain" parameter for testing
        _parameterIds.push_back("gain");
        _parameterValues.push_back(1.0f);
    }

    void prepare(double sampleRate, int maxBlockSize) override {
        _sampleRate = sampleRate;
        _maxBlockSize = maxBlockSize;
        _prepared = true;
    }

    void reset() override {
        _gain = 1.0f;
    }

    void processAudioMidi(
        AudioBuffer& audioIn,
        AudioBuffer& audioOut,
        MidiBuffer& midiIn,
        MidiBuffer& midiOut,
        const NodeProcessContext& ctx
    ) override {
        // Apply gain to audio
        int numChannels = std::min(audioIn.numChannels(), audioOut.numChannels());
        int numFrames = std::min(audioIn.numFrames(), audioOut.numFrames());

        for (int ch = 0; ch < numChannels; ++ch) {
            for (int frame = 0; frame < numFrames; ++frame) {
                float sample = audioIn.getSample(frame, ch) * _gain;
                audioOut.setSample(frame, ch, sample);
            }
        }

        // MIDI pass-through
        midiOut.clear();
        midiOut.append(midiIn);
    }

    int getNumParameters() const override {
        return static_cast<int>(_parameterIds.size());
    }

    std::string getParameterId(int index) const override {
        if (index >= 0 && index < static_cast<int>(_parameterIds.size())) {
            return _parameterIds[index];
        }
        return "";
    }

    float getParameterValue(const std::string& paramId) const override {
        for (size_t i = 0; i < _parameterIds.size(); ++i) {
            if (_parameterIds[i] == paramId) {
                return _parameterValues[i];
            }
        }
        return 0.0f;
    }

    void setParameterValue(const std::string& paramId, float normalisedValue) override {
        normalisedValue = std::max(0.0f, std::min(1.0f, normalisedValue));
        for (size_t i = 0; i < _parameterIds.size(); ++i) {
            if (_parameterIds[i] == paramId) {
                _parameterValues[i] = normalisedValue;
                _gain = normalisedValue; // Update gain for processing
                return;
            }
        }
    }

    std::vector<uint8_t> getStateChunk() const override {
        // Simple state: just the gain value as bytes
        std::vector<uint8_t> state(sizeof(float));
        float gain = _gain;
        std::memcpy(state.data(), &gain, sizeof(float));
        return state;
    }

    void setStateChunk(const std::vector<uint8_t>& data) override {
        if (data.size() >= sizeof(float)) {
            float gain;
            std::memcpy(&gain, data.data(), sizeof(float));
            setParameterValue("gain", gain);
        }
    }

    const PluginDescriptor& getDescriptor() const override {
        return _descriptor;
    }

    bool negotiateAudioIO(int requestedInputs, int requestedOutputs) override {
        // Test plugin: accept requested I/O if it matches supported config
        // For testing: support mono (1/1) and stereo (2/2) only
        if ((requestedInputs == 1 && requestedOutputs == 1) ||
            (requestedInputs == 2 && requestedOutputs == 2)) {
            _descriptor.numAudioInputs = requestedInputs;
            _descriptor.numAudioOutputs = requestedOutputs;
            return true;
        }
        // Mismatch: set to what we actually support (for testing, use stereo as fallback)
        _descriptor.numAudioInputs = 2;
        _descriptor.numAudioOutputs = 2;
        return true; // Query succeeded, but config doesn't match requested
    }

private:
    PluginDescriptor _descriptor;
    double _sampleRate;
    int _maxBlockSize;
    bool _prepared;
    float _gain;
    std::vector<std::string> _parameterIds;
    std::vector<float> _parameterValues;
};

/// Test plugin host that creates TestPluginInstance
class TestPluginHost : public PluginHost {
public:
    std::unique_ptr<PluginInstance> createInstance(const PluginDescriptor& desc) override {
        if (desc.format == PluginFormat::Clap || desc.id == "test-plugin") {
            return std::make_unique<TestPluginInstance>(desc);
        }
        return nullptr;
    }

    bool isFormatSupported(PluginFormat format) const override {
        return format == PluginFormat::Clap;
    }
};

TEST_CASE("Phase 4 - Plugin host abstraction sanity", "[plugin][phase4]") {
    TestPluginHost host;

    PluginDescriptor desc;
    desc.format = PluginFormat::Clap;
    desc.id = "test-plugin";
    desc.name = "Test Plugin";
    desc.numAudioInputs = 2;
    desc.numAudioOutputs = 2;
    desc.hasMidiInput = true;
    desc.hasMidiOutput = true;

    auto plugin = host.createInstance(desc);
    REQUIRE(plugin != nullptr);

    // Test prepare
    plugin->prepare(44100.0, 512);
    REQUIRE(plugin->getNumParameters() > 0);

    // Test parameter access
    std::string paramId = plugin->getParameterId(0);
    REQUIRE(!paramId.empty());
    REQUIRE(plugin->getParameterValue(paramId) == 1.0f);

    // Test parameter change
    plugin->setParameterValue(paramId, 0.5f);
    REQUIRE(plugin->getParameterValue(paramId) == 0.5f);

    // Test processing
    AudioBuffer audioIn, audioOut;
    audioIn.resize(2, 512);
    audioOut.resize(2, 512);
    audioIn.setSample(0, 0, 1.0f); // Unity input

    MidiBuffer midiIn, midiOut;
    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;

    plugin->processAudioMidi(audioIn, audioOut, midiIn, midiOut, npc);

    // Verify gain applied (0.5 * 1.0 = 0.5)
    REQUIRE(std::abs(audioOut.getSample(0, 0) - 0.5f) < 0.01f);
}

TEST_CASE("Phase 4 - Plugin node integration", "[plugin][phase4][node]") {
    GraphEngine engine;
    TestPluginHost host;

    // Create graph with PluginNode (AudioFx kind) using test plugin
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    fxNode.trackId = "track-1";
    fxNode.pluginFormat = PluginFormat::Clap;
    fxNode.pluginId = "test-plugin";
    fxNode.numAudioInputs = 2;
    fxNode.numAudioOutputs = 2;
    snapshot.nodes.push_back(fxNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::HardwareAudioOutput;
    snapshot.nodes.push_back(deviceNode);

    ConnectionDesc conn;
    conn.fromNodeId = "fx-1";
    conn.toNodeId = "device";
    snapshot.connections.push_back(conn);

    engine.loadGraphSnapshot(snapshot, &host);
    engine.prepareGraph(44100, 512);

    // Verify plugin was created
    auto* fx = dynamic_cast<PluginNode*>(engine.findNode("fx-1"));
    REQUIRE(fx != nullptr);
    REQUIRE(fx->getPluginKind() == PluginNodeKind::AudioFx);
    REQUIRE(fx->getPlugin() != nullptr);

    // Test processing
    fx->io.audioIn.setSample(0, 0, 1.0f); // Unity input

    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;

    fx->process(npc);

    // Verify plugin processed audio (default gain = 1.0, so output = input)
    REQUIRE(std::abs(fx->io.audioOut.getSample(0, 0) - 1.0f) < 0.01f);
}

TEST_CASE("Phase 80 - VST3 factory failure is safe", "[plugin][phase80][vst3]") {
    PluginHost host;

    PluginDescriptor desc;
    desc.format = PluginFormat::Vst3;
    desc.id = "vst3:missing-plugin";
    desc.name = "Missing VST3 Plugin";
    desc.numAudioInputs = 2;
    desc.numAudioOutputs = 2;
    desc.hasMidiInput = true;
    desc.hasMidiOutput = false;

    auto plugin = host.createInstance(desc);
    REQUIRE(plugin == nullptr);
}

TEST_CASE("Phase 4 - Parameter change test", "[plugin][phase4][parameter]") {
    // Use TestPluginHost for reliable plugin creation
    TestPluginHost testHost;
    GraphEngine engine;

    // Create graph with PluginNode (AudioFx kind)
    GraphSnapshot snapshot;
    snapshot.id = "test-graph";

    NodeDesc fxNode;
    fxNode.nodeId = "fx-1";
    fxNode.kind = NodeKind::AudioFx;
    fxNode.pluginFormat = PluginFormat::Clap;
    fxNode.pluginId = "test-plugin";
    fxNode.numAudioInputs = 2;
    fxNode.numAudioOutputs = 2;
    snapshot.nodes.push_back(fxNode);

    NodeDesc deviceNode;
    deviceNode.nodeId = "device";
    deviceNode.kind = NodeKind::HardwareAudioOutput;
    snapshot.nodes.push_back(deviceNode);

    ConnectionDesc conn;
    conn.fromNodeId = "fx-1";
    conn.toNodeId = "device";
    snapshot.connections.push_back(conn);

    engine.loadGraphSnapshot(snapshot, &testHost);
    engine.prepareGraph(44100, 512);

    // Get the plugin node
    auto* fx = dynamic_cast<PluginNode*>(engine.findNode("fx-1"));
    REQUIRE(fx != nullptr);
    REQUIRE(fx->getPluginKind() == PluginNodeKind::AudioFx);
    REQUIRE(fx->getPlugin() != nullptr);

    // Verify initial gain is 1.0
    REQUIRE(std::abs(fx->getPlugin()->getParameterValue("gain") - 1.0f) < 0.01f);

    // Apply parameter change directly to plugin (simulating EngineHost::applyParameterChanges)
    fx->getPlugin()->setParameterValue("gain", 0.5f);

    // Verify parameter was applied
    REQUIRE(std::abs(fx->getPlugin()->getParameterValue("gain") - 0.5f) < 0.01f);

    // Test processing with new parameter value
    fx->io.audioIn.setSample(0, 0, 1.0f); // Unity input

    NodeProcessContext npc;
    npc.sampleRate = 44100;
    npc.blockSize = 512;
    npc.blockStartSample = 0;

    fx->process(npc);

    // Verify output is scaled by gain (0.5 * 1.0 = 0.5)
    REQUIRE(std::abs(fx->io.audioOut.getSample(0, 0) - 0.5f) < 0.01f);
}

TEST_CASE("Phase 4 - Plugin state round-trip test", "[plugin][phase4][state]") {
    TestPluginHost host;

    PluginDescriptor desc;
    desc.format = PluginFormat::Clap;
    desc.id = "test-plugin";
    desc.name = "Test Plugin";
    desc.numAudioInputs = 2;
    desc.numAudioOutputs = 2;
    desc.hasMidiInput = true;
    desc.hasMidiOutput = true;

    auto plugin = host.createInstance(desc);
    REQUIRE(plugin != nullptr);

    plugin->prepare(44100.0, 512);

    // Set initial parameter value
    plugin->setParameterValue("gain", 0.75f);
    REQUIRE(std::abs(plugin->getParameterValue("gain") - 0.75f) < 0.01f);

    // Get state chunk
    auto state = plugin->getStateChunk();
    REQUIRE(!state.empty());

    // Modify parameter
    plugin->setParameterValue("gain", 0.25f);
    REQUIRE(std::abs(plugin->getParameterValue("gain") - 0.25f) < 0.01f);

    // Restore state
    plugin->setStateChunk(state);

    // Verify parameter returned to saved value
    REQUIRE(std::abs(plugin->getParameterValue("gain") - 0.75f) < 0.01f);
}

TEST_CASE("Phase 3 - Plugin I/O negotiation: mono plugin, mono snapshot", "[plugin][phase3][io]") {
    GraphEngine engine;
    TestPluginHost host;

    GraphSnapshot snapshot;
    snapshot.id = "mono-plugin-test";

    // Create mono plugin node
    NodeDesc pluginDesc;
    pluginDesc.nodeId = "plugin-mono";
    pluginDesc.kind = NodeKind::AudioFx;
    pluginDesc.pluginFormat = PluginFormat::Clap;
    pluginDesc.pluginId = "test.plugin.mono";
    pluginDesc.numAudioInputs = 1;  // Mono input
    pluginDesc.numAudioOutputs = 1; // Mono output
    snapshot.nodes.push_back(pluginDesc);

    engine.loadGraphSnapshot(snapshot, &host);
    engine.prepareGraph(44100, 512);

    // Find the plugin node
    auto* node = engine.findNode("plugin-mono");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::AudioFx);

    // Check that config is mono (from snapshot)
    const auto& config = node->getAudioConfig();
    REQUIRE(config.numInputChannels == 1);
    REQUIRE(config.numOutputChannels == 1);
    REQUIRE(config.layout == ChannelLayout::Mono);
}

TEST_CASE("Phase 3 - Plugin I/O negotiation: stereo plugin, stereo snapshot", "[plugin][phase3][io]") {
    GraphEngine engine;
    TestPluginHost host;

    GraphSnapshot snapshot;
    snapshot.id = "stereo-plugin-test";

    // Create stereo plugin node
    NodeDesc pluginDesc;
    pluginDesc.nodeId = "plugin-stereo";
    pluginDesc.kind = NodeKind::AudioFx;
    pluginDesc.pluginFormat = PluginFormat::Clap;
    pluginDesc.pluginId = "test.plugin.stereo";
    pluginDesc.numAudioInputs = 2;  // Stereo input
    pluginDesc.numAudioOutputs = 2; // Stereo output
    snapshot.nodes.push_back(pluginDesc);

    engine.loadGraphSnapshot(snapshot, &host);
    engine.prepareGraph(44100, 512);

    // Find the plugin node
    auto* node = engine.findNode("plugin-stereo");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::AudioFx);

    // Check that config is stereo (from snapshot)
    const auto& config = node->getAudioConfig();
    REQUIRE(config.numInputChannels == 2);
    REQUIRE(config.numOutputChannels == 2);
    REQUIRE(config.layout == ChannelLayout::Stereo);
}

TEST_CASE("Phase 3 - Plugin I/O negotiation: mono-only plugin, stereo snapshot", "[plugin][phase3][io]") {
    GraphEngine engine;
    TestPluginHost host;

    GraphSnapshot snapshot;
    snapshot.id = "mismatch-plugin-test";

    // Create plugin node with stereo request, but plugin only supports mono
    NodeDesc pluginDesc;
    pluginDesc.nodeId = "plugin-mismatch";
    pluginDesc.kind = NodeKind::AudioFx;
    pluginDesc.pluginFormat = PluginFormat::Clap;
    pluginDesc.pluginId = "test.plugin.mono"; // This plugin only supports mono
    pluginDesc.numAudioInputs = 2;  // Request stereo
    pluginDesc.numAudioOutputs = 2; // Request stereo
    snapshot.nodes.push_back(pluginDesc);

    engine.loadGraphSnapshot(snapshot, &host);
    engine.prepareGraph(44100, 512);

    // Find the plugin node
    auto* node = engine.findNode("plugin-mismatch");
    REQUIRE(node != nullptr);
    REQUIRE(node->getKind() == NodeKind::AudioFx);

    // Check that config is still stereo (snapshot is source of truth)
    const auto& config = node->getAudioConfig();
    REQUIRE(config.numInputChannels == 2);
    REQUIRE(config.numOutputChannels == 2);
    REQUIRE(config.layout == ChannelLayout::Stereo);

    // Note: In a real scenario, the plugin would be bypassed (_ioNegotiationOk = false)
    // but the NodeAudioConfig remains as requested from snapshot
}
