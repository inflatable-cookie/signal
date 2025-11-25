#pragma once

/// PluginHost - Factory for creating plugin instances
///
/// Thread: Control thread only
/// Ownership: Owned by EngineHost
///
/// This class is responsible for creating PluginInstance objects from PluginDescriptors.
/// Phase 5: Uses ClapRegistry for plugin discovery and creation.

#include "core/PluginInstance.hpp"
#include <memory>
#include <string>

// Forward declarations
class ClapRegistry;
std::unique_ptr<PluginInstance> createClapInstance(
    std::shared_ptr<class ClapPluginLibrary> library,
    const struct clap_plugin_descriptor* clapDesc
);

class PluginHost {
public:
    PluginHost();
    virtual ~PluginHost();

    /// Create a plugin instance from a descriptor
    /// @param desc Plugin descriptor
    /// @return Plugin instance, or nullptr if creation failed or format not supported
    virtual std::unique_ptr<PluginInstance> createInstance(const PluginDescriptor& desc);

    /// Check if a plugin format is supported
    /// @param format Plugin format
    /// @return true if format is supported, false otherwise
    virtual bool isFormatSupported(PluginFormat format) const;

    /// Get the CLAP registry (for discovery)
    ClapRegistry& getClapRegistry() { return *_clapRegistry; }
    const ClapRegistry& getClapRegistry() const { return *_clapRegistry; }

    /// Scan for CLAP plugins (deferred from construction)
    /// This should be called after Signal has started its server
    /// to prevent crashes from blocking startup
    void scanPlugins();

private:
    std::unique_ptr<ClapRegistry> _clapRegistry;
};

