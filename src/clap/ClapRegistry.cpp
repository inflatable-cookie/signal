#include "clap/ClapRegistry.hpp"
#include <iostream>
#include <algorithm>
#include <cstdlib>

ClapRegistry::ClapRegistry() {
    std::cout << "[ClapRegistry] Created" << std::endl;
}

ClapRegistry::~ClapRegistry() {
    clear();
    std::cout << "[ClapRegistry] Destroyed" << std::endl;
}

void ClapRegistry::scanDefaultPaths() {
    std::vector<std::filesystem::path> paths = getDefaultSearchPaths();
    std::cout << "[ClapRegistry] scanDefaultPaths() - found " << paths.size() << " paths to scan" << std::endl;
    std::cout.flush();

    for (const auto& path : paths) {
        if (std::filesystem::exists(path) && std::filesystem::is_directory(path)) {
            std::cout << "[ClapRegistry] Scanning directory: " << path << std::endl;
            std::cout.flush();
            scanPath(path);
            std::cout << "[ClapRegistry] Finished scanning: " << path << std::endl;
            std::cout.flush();
        } else {
            std::cout << "[ClapRegistry] Skipping non-existent path: " << path << std::endl;
            std::cout.flush();
        }
    }

    std::cout << "[ClapRegistry] scanDefaultPaths() complete" << std::endl;
    std::cout.flush();
}

void ClapRegistry::scanPath(const std::filesystem::path& path) {
    if (!std::filesystem::exists(path)) {
        std::cerr << "[ClapRegistry] Path does not exist: " << path << std::endl;
        return;
    }

    if (std::filesystem::is_directory(path)) {
        scanDirectory(path);
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
            std::cerr << "[ClapRegistry] Failed to load plugin: " << path << " - " << e.what() << std::endl;
        }
    }
}

void ClapRegistry::scanDirectory(const std::filesystem::path& dir) {
    std::cout << "[ClapRegistry] scanDirectory() - scanning: " << dir << std::endl;
    std::cout.flush();
    try {
        size_t fileCount = 0;
        for (const auto& entry : std::filesystem::directory_iterator(dir)) {
            const auto& path = entry.path();
            fileCount++;
            std::cout << "[ClapRegistry] Processing entry " << fileCount << ": " << path << std::endl;
            std::cout.flush();

            // Check for .clap bundle (macOS) or .so/.dylib files
            // ClapPluginLibrary now handles .clap bundle resolution internally
            std::string ext = path.extension().string();
            bool isClapFile = (ext == ".clap" || ext == ".so" || ext == ".dylib" || ext == ".dll");

            if (isClapFile) {
                std::cout << "[ClapRegistry] Attempting to load plugin: " << path << std::endl;
                std::cout.flush();
                try {
                    auto library = std::make_shared<ClapPluginLibrary>(path);
                    if (library->isValid()) {
                        std::cout << "[ClapRegistry] Successfully loaded plugin: " << path << std::endl;
                        std::cout.flush();
                        // Register all plugins from this library
                        auto descriptors = library->getAllDescriptors();
                        std::cout << "[ClapRegistry] Found " << descriptors.size() << " plugin(s) in " << path << std::endl;
                        std::cout.flush();
                        for (const auto* clapDesc : descriptors) {
                            if (clapDesc) {
                                registerPlugin(library, clapDesc);
                            }
                        }
                    } else {
                        std::cerr << "[ClapRegistry] Plugin loaded but invalid: " << path << std::endl;
                        std::cerr.flush();
                    }
                } catch (const std::exception& e) {
                    std::cerr << "[ClapRegistry] Exception loading plugin " << path << ": " << e.what() << std::endl;
                    std::cerr.flush();
                } catch (...) {
                    std::cerr << "[ClapRegistry] Unknown exception loading plugin: " << path << std::endl;
                    std::cerr.flush();
                }
            } else if (std::filesystem::is_directory(path)) {
                // Recursively scan subdirectories (with depth limit)
                scanDirectory(path);
            }
        }
    } catch (const std::filesystem::filesystem_error& e) {
        std::cerr << "[ClapRegistry] Error scanning directory: " << dir << " - " << e.what() << std::endl;
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
        std::cout << "[ClapRegistry] Plugin already registered: " << pluginId << std::endl;
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

    std::cout << "[ClapRegistry] Registered plugin: " << desc.name << " (" << pluginId << ")" << std::endl;
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

