#include "vst3/Vst3Registry.hpp"
#include "logging/Logging.hpp"
#include <cstdlib>
#include <sstream>

namespace {
std::string makeStablePluginId(const std::filesystem::path& path) {
    const std::string normalised = std::filesystem::weakly_canonical(path).string();
    const std::size_t hash = std::hash<std::string>{}(normalised);
    std::ostringstream id;
    id << "vst3:" << std::hex << hash;
    return id.str();
}
}

Vst3Registry::Vst3Registry() {
    LOG_DEBUG({"Vst3Registry"}, "Created");
}

Vst3Registry::~Vst3Registry() {
    clear();
    LOG_DEBUG({"Vst3Registry"}, "Destroyed");
}

void Vst3Registry::scanDefaultPaths(std::stop_token stopToken) {
    const auto paths = getDefaultSearchPaths();

    for (const auto& path : paths) {
        if (stopToken.stop_requested()) {
            return;
        }

        if (std::filesystem::exists(path) && std::filesystem::is_directory(path)) {
            scanPath(path, stopToken);
        }
    }
}

void Vst3Registry::scanPath(const std::filesystem::path& path, std::stop_token stopToken) {
    if (stopToken.stop_requested()) {
        return;
    }

    if (!std::filesystem::exists(path)) {
        LOG_WARN({"Vst3Registry"}, std::string("Path does not exist: ") + path.string());
        return;
    }

    if (std::filesystem::is_directory(path)) {
        scanDirectory(path, stopToken);
        return;
    }

    if (std::filesystem::is_regular_file(path)) {
        registerCandidate(path);
    }
}

void Vst3Registry::scanDirectory(const std::filesystem::path& dir, std::stop_token stopToken) {
    try {
        for (const auto& entry : std::filesystem::directory_iterator(dir)) {
            if (stopToken.stop_requested()) {
                return;
            }

            const auto& path = entry.path();
            const std::string extension = path.extension().string();
            const bool looksLikeVst3 = extension == ".vst3"
                || extension == ".vst"
                || extension == ".dll"
                || extension == ".so"
                || extension == ".dylib";

            if (looksLikeVst3) {
                registerCandidate(path);
            } else if (entry.is_directory()) {
                scanDirectory(path, stopToken);
            }
        }
    } catch (const std::filesystem::filesystem_error& e) {
        LOG_ERROR({"Vst3Registry"}, std::string("Error scanning directory: ") + dir.string() + " - " + e.what());
    }
}

void Vst3Registry::registerCandidate(const std::filesystem::path& path) {
    const std::string pluginId = makeStablePluginId(path);

    if (_idToIndex.contains(pluginId)) {
        return;
    }

    PluginDescriptor desc;
    desc.format = PluginFormat::Vst3;
    desc.id = pluginId;
    desc.name = path.stem().string();
    desc.numAudioInputs = 2;
    desc.numAudioOutputs = 2;
    desc.hasMidiInput = true;
    desc.hasMidiOutput = false;

    _entries.push_back(Entry{
        .desc = desc,
        .path = path,
    });
    _idToIndex[pluginId] = _entries.size() - 1;

    LOG_DEBUG({"Vst3Registry"}, std::string("Discovered VST3 candidate: ") + desc.name + " (" + pluginId + ")");
}

std::vector<PluginDescriptor> Vst3Registry::listPlugins() const {
    std::vector<PluginDescriptor> plugins;
    plugins.reserve(_entries.size());

    for (const auto& entry : _entries) {
        plugins.push_back(entry.desc);
    }

    return plugins;
}

const PluginDescriptor* Vst3Registry::findPluginById(const std::string& pluginId) const {
    auto it = _idToIndex.find(pluginId);
    if (it == _idToIndex.end() || it->second >= _entries.size()) {
        return nullptr;
    }

    return &_entries[it->second].desc;
}

std::optional<std::filesystem::path> Vst3Registry::findPathById(const std::string& pluginId) const {
    auto it = _idToIndex.find(pluginId);
    if (it == _idToIndex.end() || it->second >= _entries.size()) {
        return std::nullopt;
    }

    return _entries[it->second].path;
}

void Vst3Registry::clear() {
    _entries.clear();
    _idToIndex.clear();
}

std::vector<std::filesystem::path> Vst3Registry::getDefaultSearchPaths() const {
    std::vector<std::filesystem::path> paths;

    const char* envPath = std::getenv("LOOPHOLE_VST3_PATH");
    if (envPath != nullptr) {
        const std::string pathStr = envPath;
#if defined(_WIN32)
        const char separator = ';';
#else
        const char separator = ':';
#endif

        std::size_t start = 0;
        std::size_t end = pathStr.find(separator);
        while (end != std::string::npos) {
            paths.push_back(pathStr.substr(start, end - start));
            start = end + 1;
            end = pathStr.find(separator, start);
        }
        paths.push_back(pathStr.substr(start));
    }

#if defined(__APPLE__)
    if (const char* home = std::getenv("HOME")) {
        paths.push_back(std::filesystem::path(home) / "Library" / "Audio" / "Plug-Ins" / "VST3");
    }
    paths.push_back("/Library/Audio/Plug-Ins/VST3");
#elif defined(_WIN32)
    if (const char* programFiles = std::getenv("ProgramFiles")) {
        paths.push_back(std::filesystem::path(programFiles) / "Common Files" / "VST3");
    }
    if (const char* commonProgramFiles = std::getenv("CommonProgramFiles")) {
        paths.push_back(std::filesystem::path(commonProgramFiles) / "VST3");
    }
#else
    if (const char* home = std::getenv("HOME")) {
        paths.push_back(std::filesystem::path(home) / ".vst3");
    }
    paths.push_back("/usr/lib/vst3");
    paths.push_back("/usr/local/lib/vst3");
#endif

    return paths;
}
