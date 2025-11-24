#pragma once

/// PluginHost - Factory for creating plugin instances
///
/// Thread: Control thread only
/// Ownership: Owned by EngineHost
///
/// This class is responsible for creating PluginInstance objects from PluginDescriptors.
/// In Phase 4, only CLAP is implemented; other formats return nullptr.

#include "core/PluginInstance.hpp"
#include <memory>
#include <string>

// Forward declaration
std::unique_ptr<PluginInstance> createClapInstance(const PluginDescriptor& desc);

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

private:
    // Future: plugin discovery, caching, etc.
};

