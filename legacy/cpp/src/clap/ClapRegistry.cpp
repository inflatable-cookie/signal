#include "clap/ClapRegistry.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <cstdlib>
#include <sstream>

ClapRegistry::ClapRegistry() {
    LOG_DEBUG({"ClapRegistry"}, "Created");
}

ClapRegistry::~ClapRegistry() {
    clear();
    LOG_DEBUG({"ClapRegistry"}, "Destroyed");
}

void ClapRegistry::scanDefaultPaths(std::stop_token stopToken) {
    std::vector<std::filesystem::path> paths = getDefaultSearchPaths();

    for (const auto& path : paths) {
        if (stopToken.stop_requested()) {
            return;
        }

        if (std::filesystem::exists(path) && std::filesystem::is_directory(path)) {
            scanPath(path, stopToken);
        }
    }
}

void ClapRegistry::scanPath(const std::filesystem::path& path, std::stop_token stopToken) {
    if (stopToken.stop_requested()) {
        return;
    }

    if (!std::filesystem::exists(path)) {
        LOG_WARN({"ClapRegistry"}, std::string("Path does not exist: ") + path.string());
        return;
    }

    if (std::filesystem::is_directory(path)) {
        scanDirectory(path, stopToken);
    } else if (std::filesystem::is_regular_file(path)) {
        // Single file - try to load it
        try {
            auto library = std::make_shared<ClapPluginLibrary>(path);
            if (library->isValid()) {
                // Register all plugins from this library
                auto descriptors = library->getAllDescriptors();
                for (const auto* clapDesc : descriptors) {
                    if (clapDesc) {
                        registerPlugin(library, clapDesc);
                    }
                }
            }
        } catch (const std::exception& e) {
            LOG_WARN({"ClapRegistry"}, std::string("Failed to load plugin: ") + path.string() + " - " + e.what());
        }
    }
}

void ClapRegistry::scanDirectory(const std::filesystem::path& dir, std::stop_token stopToken) {
    try {
        for (const auto& entry : std::filesystem::directory_iterator(dir)) {
            if (stopToken.stop_requested()) {
                return;
            }

            const auto& path = entry.path();

            // Check for .clap bundle (macOS) or .so/.dylib files
            // ClapPluginLibrary now handles .clap bundle resolution internally
            std::string ext = path.extension().string();
            bool isClapFile = (ext == ".clap" || ext == ".so" || ext == ".dylib" || ext == ".dll");

            if (isClapFile) {
                try {
                    auto library = std::make_shared<ClapPluginLibrary>(path);
                    if (library->isValid()) {
                        // Register all plugins from this library
                        auto descriptors = library->getAllDescriptors();
                        for (const auto* clapDesc : descriptors) {
                            if (clapDesc) {
                                registerPlugin(library, clapDesc);
                            }
                        }
                    } else {
                        LOG_DEBUG({"ClapRegistry"}, std::string("Failed to load plugin: ") + path.string());
                    }
                } catch (const std::exception& e) {
                    LOG_WARN({"ClapRegistry"}, std::string("Exception loading plugin ") + path.string() + ": " + e.what());
                } catch (...) {
                    LOG_WARN({"ClapRegistry"}, std::string("Unknown exception loading plugin: ") + path.string());
                }
            } else if (std::filesystem::is_directory(path)) {
                // Recursively scan subdirectories (with depth limit)
                scanDirectory(path, stopToken);
            }
        }
    } catch (const std::filesystem::filesystem_error& e) {
        LOG_ERROR({"ClapRegistry"}, std::string("Error scanning directory: ") + dir.string() + " - " + e.what());
    }
}

void ClapRegistry::registerPlugin(
    std::shared_ptr<ClapPluginLibrary> library,
    const clap_plugin_descriptor_t* clapDesc
) {
    if (!clapDesc || !clapDesc->id) {
        return;
    }

    std::string pluginId = clapDesc->id;

    // Check if already registered
    if (_idToIndex.find(pluginId) != _idToIndex.end()) {
        LOG_DEBUG({"ClapRegistry"}, std::string("Plugin already registered: ") + pluginId);
        return;
    }

    // Create PluginDescriptor from CLAP descriptor
    PluginDescriptor desc;
    desc.format = PluginFormat::Clap;
    desc.id = pluginId;
    desc.name = clapDesc->name ? clapDesc->name : pluginId;

    // TODO: Query actual I/O counts from CLAP plugin (requires plugin instance)
    // For now, use defaults
    desc.numAudioInputs = 2;
    desc.numAudioOutputs = 2;
    desc.hasMidiInput = true;
    desc.hasMidiOutput = false;

    // Register entry
    Entry entry;
    entry.desc = desc;
    entry.library = library;
    entry.clapDesc = clapDesc;

    _entries.push_back(entry);
    _idToIndex[pluginId] = _entries.size() - 1;

    LOG_DEBUG({"ClapRegistry"}, std::string("Found plugin: ") + desc.name + " (" + pluginId + ")");
}

std::vector<PluginDescriptor> ClapRegistry::listPlugins() const {
    std::vector<PluginDescriptor> plugins;
    plugins.reserve(_entries.size());

    for (const auto& entry : _entries) {
        plugins.push_back(entry.desc);
    }

    return plugins;
}

const PluginDescriptor* ClapRegistry::findPluginById(const std::string& pluginId) const {
    auto it = _idToIndex.find(pluginId);
    if (it != _idToIndex.end() && it->second < _entries.size()) {
        return &_entries[it->second].desc;
    }
    return nullptr;
}

std::shared_ptr<ClapPluginLibrary> ClapRegistry::getLibrary(const std::string& pluginId) const {
    auto it = _idToIndex.find(pluginId);
    if (it != _idToIndex.end() && it->second < _entries.size()) {
        return _entries[it->second].library;
    }
    return nullptr;
}

void ClapRegistry::clear() {
    _entries.clear();
    _idToIndex.clear();
}

std::vector<std::filesystem::path> ClapRegistry::getDefaultSearchPaths() const {
    std::vector<std::filesystem::path> paths;

    // Check environment variable
    const char* envPath = std::getenv("LOOPHOLE_CLAP_PATH");
    if (envPath) {
        std::string pathStr = envPath;
        // Split by platform-specific separator
        #if defined(_WIN32)
            const char separator = ';';
        #else
            const char separator = ':';
        #endif

        size_t start = 0;
        size_t end = pathStr.find(separator);
        while (end != std::string::npos) {
            paths.push_back(pathStr.substr(start, end - start));
            start = end + 1;
            end = pathStr.find(separator, start);
        }
        paths.push_back(pathStr.substr(start));
    }

    // Add common system paths
    #if defined(__APPLE__)
        // macOS
        paths.push_back(std::filesystem::path(std::getenv("HOME")) / "Library" / "Audio" / "Plug-Ins" / "CLAP");
        paths.push_back("/Library/Audio/Plug-Ins/CLAP");
    #elif defined(_WIN32)
        // Windows
        const char* programFiles = std::getenv("ProgramFiles");
        if (programFiles) {
            paths.push_back(std::filesystem::path(programFiles) / "Common Files" / "CLAP");
        }
        const char* localAppData = std::getenv("LOCALAPPDATA");
        if (localAppData) {
            paths.push_back(std::filesystem::path(localAppData) / "Programs" / "CLAP");
        }
    #else
        // Linux
        paths.push_back(std::filesystem::path(std::getenv("HOME")) / ".clap");
        paths.push_back("/usr/lib/clap");
        paths.push_back("/usr/local/lib/clap");
    #endif

    return paths;
}
