#pragma once

/// Vst3Registry - Plugin discovery and registry for VST3 plugins
///
/// Thread: Control thread only
/// Ownership: Owned by Vst3Backend
///
/// This class scans configured directories for VST3 plugins and maintains a
/// lightweight descriptor registry keyed by stable IDs.

#include "core/PluginInstance.hpp"
#include <filesystem>
#include <optional>
#include <stop_token>
#include <string>
#include <unordered_map>
#include <vector>

class Vst3Registry {
public:
    struct Entry {
        PluginDescriptor desc;
        std::filesystem::path path;
    };

    Vst3Registry();
    ~Vst3Registry();

    void scanDefaultPaths(std::stop_token stopToken = {});
    void scanPath(const std::filesystem::path& path, std::stop_token stopToken = {});

    std::vector<PluginDescriptor> listPlugins() const;
    const PluginDescriptor* findPluginById(const std::string& pluginId) const;
    std::optional<std::filesystem::path> findPathById(const std::string& pluginId) const;

    void clear();

private:
    std::vector<Entry> _entries;
    std::unordered_map<std::string, size_t> _idToIndex;

    void scanDirectory(const std::filesystem::path& dir, std::stop_token stopToken);
    void registerCandidate(const std::filesystem::path& path);
    std::vector<std::filesystem::path> getDefaultSearchPaths() const;
};
