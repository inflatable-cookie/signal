#include "vst3/Vst3Backend.hpp"
#include "logging/Logging.hpp"
#include "vst3/Vst3PluginInstance.hpp"
#include <utility>

Vst3Backend::Vst3Backend() = default;

Vst3Backend::~Vst3Backend() = default;

void Vst3Backend::scanDefaultPaths(std::stop_token stopToken) {
#if SIGNAL_ENABLE_VST3
    _registry.scanDefaultPaths(stopToken);
#else
    (void) stopToken;
#endif
}

std::vector<PluginDescriptor> Vst3Backend::listPlugins() const {
#if SIGNAL_ENABLE_VST3
    return _registry.listPlugins();
#else
    return {};
#endif
}

std::unique_ptr<PluginInstance> Vst3Backend::createInstance(
    const PluginDescriptor& desc,
    std::string& error
) {
    if (desc.format != PluginFormat::Vst3) {
        error = "Descriptor format is not VST3";
        return nullptr;
    }

#if SIGNAL_ENABLE_VST3
    const auto* registryDesc = _registry.findPluginById(desc.id);
    if (registryDesc == nullptr) {
        error = "VST3 descriptor not found in registry: " + desc.id;
        return nullptr;
    }

    auto modulePath = _registry.findPathById(desc.id);
    if (!modulePath.has_value()) {
        error = "VST3 module path not found in registry: " + desc.id;
        return nullptr;
    }

    PluginDescriptor runtimeDesc = *registryDesc;
    runtimeDesc.numAudioInputs = desc.numAudioInputs;
    runtimeDesc.numAudioOutputs = desc.numAudioOutputs;
    runtimeDesc.hasMidiInput = desc.hasMidiInput;
    runtimeDesc.hasMidiOutput = desc.hasMidiOutput;

    return std::make_unique<Vst3PluginInstance>(
        std::move(runtimeDesc),
        modulePath->string()
    );
#else
    error = "VST3 support is disabled at build time (SIGNAL_ENABLE_VST3=OFF)";
    return nullptr;
#endif
}

bool Vst3Backend::isEnabled() const noexcept {
#if SIGNAL_ENABLE_VST3
    return true;
#else
    return false;
#endif
}
