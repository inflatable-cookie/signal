#pragma once

/// Vst3Backend - VST3 hosting scaffold behind PluginHost
///
/// Thread: Control thread only
/// Ownership: Owned by PluginHost

#include "core/PluginInstance.hpp"
#include "vst3/Vst3Registry.hpp"
#include <stop_token>
#include <string>
#include <vector>

class Vst3Backend {
public:
    Vst3Backend();
    ~Vst3Backend();

    void scanDefaultPaths(std::stop_token stopToken = {});
    std::vector<PluginDescriptor> listPlugins() const;

    std::unique_ptr<PluginInstance> createInstance(
        const PluginDescriptor& desc,
        std::string& error
    );

    bool isEnabled() const noexcept;

private:
    Vst3Registry _registry;
};
