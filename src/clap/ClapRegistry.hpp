#pragma once

/// ClapRegistry - Plugin discovery and registry for CLAP plugins
///
/// Thread: Control thread only
/// Ownership: Owned by PluginHost
///
/// This class scans configured directories for CLAP plugins and maintains a registry
/// of available plugins. It provides lookup by plugin ID and enumeration of all plugins.

#include "core/PluginInstance.hpp"
#include "clap/ClapPluginLibrary.hpp"
#include <string>
#include <stop_token>
#include <vector>
#include <unordered_map>
#include <filesystem>
#include <memory>

/// CLAP plugin registry
class ClapRegistry {
public:
    ClapRegistry();
    ~ClapRegistry();

    /// Scan default CLAP plugin paths
    /// Uses environment variable LOOPHOLE_CLAP_PATH or common system paths
    void scanDefaultPaths(std::stop_token stopToken = {});

    /// Scan a specific path for CLAP plugins
    /// @param path Directory path to scan
    void scanPath(const std::filesystem::path& path, std::stop_token stopToken = {});

    /// Get all registered plugin descriptors
    /// @return Vector of plugin descriptors
    std::vector<PluginDescriptor> listPlugins() const;

    /// Find a plugin descriptor by ID
    /// @param pluginId CLAP plugin ID
    /// @return Plugin descriptor pointer, or nullptr if not found
    const PluginDescriptor* findPluginById(const std::string& pluginId) const;

    /// Get the library for a plugin ID
    /// @param pluginId CLAP plugin ID
    /// @return Shared pointer to library, or nullptr if not found
    std::shared_ptr<ClapPluginLibrary> getLibrary(const std::string& pluginId) const;

    /// Clear the registry
    void clear();

private:
    struct Entry {
        PluginDescriptor desc;
        std::shared_ptr<ClapPluginLibrary> library;
        const clap_plugin_descriptor_t* clapDesc;
    };

    std::vector<Entry> _entries;
    std::unordered_map<std::string, size_t> _idToIndex; // pluginId -> index in _entries

    // Helper methods
    void scanDirectory(const std::filesystem::path& dir, std::stop_token stopToken);
    void registerPlugin(
        std::shared_ptr<ClapPluginLibrary> library,
        const clap_plugin_descriptor_t* clapDesc
    );
    std::vector<std::filesystem::path> getDefaultSearchPaths() const;
};
