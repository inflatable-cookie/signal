#include "clap/ClapPluginLibrary.hpp"
#include "clap/clap.h"
#include "core/PluginCrashTracking.hpp"
#include <iostream>
#include <cstring>
#include <filesystem>
#include <setjmp.h>
#include <csignal>

// Platform-specific dynamic loading
#if defined(_WIN32)
    #include <windows.h>
    #define CLAP_LIB_HANDLE HMODULE
    #define CLAP_LIB_LOAD(path) LoadLibraryA(path.string().c_str())
    #define CLAP_LIB_UNLOAD(handle) FreeLibrary(handle)
    #define CLAP_LIB_SYMBOL(handle, name) GetProcAddress(handle, name)
#else
    #include <dlfcn.h>
    #define CLAP_LIB_HANDLE void*
    #define CLAP_LIB_LOAD(path) dlopen(path.string().c_str(), RTLD_LAZY)
    #define CLAP_LIB_UNLOAD(handle) dlclose(handle)
    #define CLAP_LIB_SYMBOL(handle, name) dlsym(handle, name)
#endif

// CLAP entry point is a global data symbol (struct), not a function
// Using official CLAP SDK types: clap_plugin_entry_t from clap/entry.h

/// Resolve the actual library path from a .clap bundle on macOS
/// On macOS, .clap files are bundles (directories) containing the actual library
/// Structure: PluginName.clap/Contents/MacOS/PluginName
static std::filesystem::path resolveClapLibraryPath(const std::filesystem::path& path) {
    // If it's not a .clap extension, assume it's already the library path
    if (path.extension() != ".clap") {
        return path;
    }

    #if defined(__APPLE__)
        // Get bundle name from path stem (e.g., "ThingsCrusher" from "ThingsCrusher.clap")
        std::string bundleName = path.stem().string();
        std::filesystem::path macosDir = path / "Contents" / "MacOS";

        // First, try the canonical bundle binary path
        std::filesystem::path canonical = macosDir / bundleName;

        if (std::filesystem::exists(canonical) && std::filesystem::is_regular_file(canonical)) {
            return canonical;
        }

        // Fallback: scan MacOS directory if it exists
        if (std::filesystem::exists(macosDir) && std::filesystem::is_directory(macosDir)) {
            for (const auto& entry : std::filesystem::directory_iterator(macosDir)) {
                const auto& entryPath = entry.path();
                if (std::filesystem::is_regular_file(entryPath)) {
                    std::string ext = entryPath.extension().string();
                    // Accept files with .dylib/.so extensions, or files without extensions (Mach-O bundles)
                    if (ext == ".dylib" || ext == ".so" || ext.empty()) {
                        return entryPath;
                    }
                }
            }
        }

        // If no library found, log error and return original path (will fail with clear error)
        std::cerr << "[ClapPluginLibrary] No library file found in bundle: " << path << std::endl;
        std::cerr << "[ClapPluginLibrary] Expected canonical path: " << canonical << " (not found)" << std::endl;
        if (std::filesystem::exists(macosDir) && std::filesystem::is_directory(macosDir)) {
            std::cerr << "[ClapPluginLibrary] MacOS directory exists but contains no suitable library file" << std::endl;
        } else {
            std::cerr << "[ClapPluginLibrary] MacOS directory does not exist: " << macosDir << std::endl;
        }
        std::cerr.flush();
        return path;
    #else
        // On Linux/Windows, .clap might be a direct library file
        return path;
    #endif
}

ClapPluginLibrary::ClapPluginLibrary(const std::filesystem::path& path)
    : _path(path)
    , _handle(nullptr)
    , _entry(nullptr)
    , _factory(nullptr)
    , _valid(false)
{
    // Set global flag for crash reporting
    g_inPluginLoading = true;
    std::string pathStr = path.string();
    strncpy(g_currentPluginPath, pathStr.c_str(), sizeof(g_currentPluginPath) - 1);
    g_currentPluginPath[sizeof(g_currentPluginPath) - 1] = '\0';

    try {
        if (loadLibrary()) {
            _valid = true;
        } else {
            std::cerr << "[ClapPluginLibrary] Failed to load plugin: " << path << std::endl;
            std::cerr.flush();
            // Jump buffer flag should already be cleared by loadLibrary() on failure
        }
    } catch (const std::exception& e) {
        std::cerr << "[ClapPluginLibrary] Exception in constructor for " << path << ": " << e.what() << std::endl;
        std::cerr.flush();
        g_inPluginLoading = false;
        g_currentPluginPath[0] = '\0';
        g_pluginLoadJumpSet = false; // Clear jump buffer flag
        throw;
    } catch (...) {
        std::cerr << "[ClapPluginLibrary] Unknown exception in constructor for: " << path << std::endl;
        std::cerr.flush();
        g_inPluginLoading = false;
        g_currentPluginPath[0] = '\0';
        g_pluginLoadJumpSet = false; // Clear jump buffer flag
        throw;
    }

    // Clear flag on success
    g_inPluginLoading = false;
    g_currentPluginPath[0] = '\0';
    g_pluginLoadJumpSet = false; // Ensure jump buffer flag is cleared
}

ClapPluginLibrary::~ClapPluginLibrary() {
    unloadLibrary();
}

ClapPluginLibrary::ClapPluginLibrary(ClapPluginLibrary&& other) noexcept
    : _path(std::move(other._path))
    , _handle(other._handle)
    , _entry(other._entry)
    , _factory(other._factory)
    , _valid(other._valid)
{
    other._handle = nullptr;
    other._entry = nullptr;
    other._factory = nullptr;
    other._valid = false;
}

ClapPluginLibrary& ClapPluginLibrary::operator=(ClapPluginLibrary&& other) noexcept {
    if (this != &other) {
        unloadLibrary();
        _path = std::move(other._path);
        _handle = other._handle;
        _entry = other._entry;
        _factory = other._factory;
        _valid = other._valid;
        other._handle = nullptr;
        other._entry = nullptr;
        other._factory = nullptr;
        other._valid = false;
    }
    return *this;
}

bool ClapPluginLibrary::loadLibrary() {
    // Resolve the actual library path (handles .clap bundles on macOS)
    std::filesystem::path libraryPath = resolveClapLibraryPath(_path);

    _handle = CLAP_LIB_LOAD(libraryPath);
    if (!_handle) {
        #if defined(_WIN32)
            DWORD error = GetLastError();
            std::cerr << "[ClapPluginLibrary] LoadLibrary failed: " << libraryPath << " - " << error << std::endl;
        #else
            const char* error = dlerror();
            std::cerr << "[ClapPluginLibrary] dlopen failed: " << libraryPath << " - " << (error ? error : "unknown") << std::endl;
        #endif
        std::cerr.flush();
        return false;
    }

    // Find CLAP entry point
    if (!initClapEntry()) {
        std::cerr << "[ClapPluginLibrary] initClapEntry() failed for: " << libraryPath << std::endl;
        std::cerr.flush();
        unloadLibrary();
        return false;
    }

    // Initialize factory
    if (!initFactory()) {
        std::cerr << "[ClapPluginLibrary] initFactory() failed for: " << libraryPath << std::endl;
        std::cerr.flush();
        unloadLibrary();
        return false;
    }

    return true;
}

void ClapPluginLibrary::unloadLibrary() {
    if (_entry && _entry->deinit) {
        _entry->deinit();
    }

    if (_handle) {
        CLAP_LIB_UNLOAD(_handle);
        _handle = nullptr;
    }
    _entry = nullptr;
    _factory = nullptr;
    _valid = false;
}

bool ClapPluginLibrary::initClapEntry() {
    // CLAP plugins export clap_entry as a global data symbol (const clap_plugin_entry_t), not a function
    // Use dlsym/GetProcAddress to get the address of the clap_plugin_entry_t struct
    void* symbolAddr = CLAP_LIB_SYMBOL(_handle, "clap_entry");

    if (!symbolAddr) {
        std::cerr << "[ClapPluginLibrary] clap_entry symbol not found in: " << _path << std::endl;
        std::cerr.flush();
        return false;
    }

    // Validate symbol address is not in an obviously invalid memory range
    if (reinterpret_cast<uintptr_t>(symbolAddr) < 0x1000) {
        std::cerr << "[ClapPluginLibrary] Invalid symbol address (too low): " << symbolAddr << " for: " << _path << std::endl;
        std::cerr << "[ClapPluginLibrary] This may indicate an ABI mismatch - plugin may be incompatible" << std::endl;
        std::cerr.flush();
        return false;
    }

    // Cast to pointer to clap_plugin_entry_t struct
    // Use sigsetjmp/siglongjmp to recover from bus errors when accessing the struct
    g_pluginLoadJumpSet = true;
    int jumpResult = sigsetjmp(g_pluginLoadJumpBuf, 1);

    if (jumpResult != 0) {
        // We jumped back from a signal handler - accessing the struct caused a crash
        std::cerr << "[ClapPluginLibrary] Bus error/segfault when accessing clap_entry struct for: " << _path << std::endl;
        std::cerr << "[ClapPluginLibrary] This indicates a likely ABI mismatch - plugin binary layout does not match host expectations" << std::endl;
        std::cerr << "[ClapPluginLibrary] Skipping this plugin" << std::endl;
        std::cerr.flush();
        g_pluginLoadJumpSet = false;
        // Clean up the library handle since we can't use it
        if (_handle) {
            CLAP_LIB_UNLOAD(_handle);
            _handle = nullptr;
        }
        return false;
    }

    // Safe to access - if it crashes, signal handler will jump back here
    // Keep jump buffer protection active while accessing struct fields
    auto* entryPtr = reinterpret_cast<const clap_plugin_entry_t*>(symbolAddr);

    if (!entryPtr) {
        std::cerr << "[ClapPluginLibrary] clap_entry struct pointer is null for: " << _path << std::endl;
        std::cerr.flush();
        g_pluginLoadJumpSet = false;
        return false;
    }

    // Check version compatibility using official clap_version_t struct
    const clap_version_t& pluginVersion = entryPtr->clap_version;
    const clap_version_t& hostVersion = CLAP_VERSION;

    if (pluginVersion.major != hostVersion.major) {
        std::cerr << "[ClapPluginLibrary] Version mismatch for: " << _path << std::endl;
        std::cerr << "[ClapPluginLibrary] Plugin CLAP version: " << pluginVersion.major << "." << pluginVersion.minor << "." << pluginVersion.revision << std::endl;
        std::cerr << "[ClapPluginLibrary] Host CLAP version: " << hostVersion.major << "." << hostVersion.minor << "." << hostVersion.revision << std::endl;
        std::cerr << "[ClapPluginLibrary] Major version mismatch - plugin may not be compatible" << std::endl;
        std::cerr.flush();
        // Continue anyway for now (as per requirements)
    }

    _entry = entryPtr;
    g_pluginLoadJumpSet = false; // Clear protection after successful struct access

    // Initialize entry - also wrap in sigsetjmp in case init() crashes
    if (_entry->init) {
        g_pluginLoadJumpSet = true;
        int jumpResult = sigsetjmp(g_pluginLoadJumpBuf, 1);

        if (jumpResult != 0) {
            // We jumped back from a signal handler - init() caused a crash
            std::cerr << "[ClapPluginLibrary] Bus error/segfault when calling entry->init() for: " << _path << std::endl;
            std::cerr << "[ClapPluginLibrary] Skipping this plugin" << std::endl;
            std::cerr.flush();
            g_pluginLoadJumpSet = false;
            _entry = nullptr; // Clear entry since init failed
            // Clean up the library handle since we can't use it
            if (_handle) {
                CLAP_LIB_UNLOAD(_handle);
                _handle = nullptr;
            }
            return false;
        }

        bool initResult = _entry->init(_path.string().c_str());
        g_pluginLoadJumpSet = false;

        if (!initResult) {
            std::cerr << "[ClapPluginLibrary] entry->init() returned false for: " << _path << std::endl;
            std::cerr.flush();
            _entry = nullptr;
            return false;
        }
    }

    return true;
}

bool ClapPluginLibrary::initFactory() {
    if (!_entry) {
        std::cerr << "[ClapPluginLibrary] initFactory() called but _entry is null for: " << _path << std::endl;
        std::cerr.flush();
        return false;
    }

    // Get factory from entry
    if (!_entry->get_factory) {
        std::cerr << "[ClapPluginLibrary] Entry does not provide get_factory for: " << _path << std::endl;
        std::cerr.flush();
        return false;
    }

    _factory = static_cast<const clap_plugin_factory_t*>(
        _entry->get_factory(CLAP_PLUGIN_FACTORY_ID)
    );

    if (!_factory) {
        std::cerr << "[ClapPluginLibrary] Failed to get plugin factory for: " << _path << std::endl;
        std::cerr.flush();
        return false;
    }

    return true;
}

const clap_plugin_descriptor_t* ClapPluginLibrary::getDescriptor(const char* pluginId) const noexcept {
    if (!_factory || !pluginId) {
        return nullptr;
    }

    if (!_factory->get_plugin_count || !_factory->get_plugin_descriptor) {
        return nullptr;
    }

    const uint32_t count = _factory->get_plugin_count(_factory);
    for (uint32_t i = 0; i < count; ++i) {
        const clap_plugin_descriptor_t* desc = _factory->get_plugin_descriptor(_factory, i);
        if (desc && desc->id && std::strcmp(desc->id, pluginId) == 0) {
            return desc;
        }
    }

    return nullptr;
}

std::vector<const clap_plugin_descriptor_t*> ClapPluginLibrary::getAllDescriptors() const {
    std::vector<const clap_plugin_descriptor_t*> descriptors;

    if (!_factory || !_factory->get_plugin_count || !_factory->get_plugin_descriptor) {
        return descriptors;
    }

    const uint32_t count = _factory->get_plugin_count(_factory);
    for (uint32_t i = 0; i < count; ++i) {
        const clap_plugin_descriptor_t* desc = _factory->get_plugin_descriptor(_factory, i);
        if (desc) {
            descriptors.push_back(desc);
        }
    }

    return descriptors;
}

