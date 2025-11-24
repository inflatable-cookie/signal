#include "clap/ClapPluginLibrary.hpp"
#include "clap/clap.h"
#include <iostream>
#include <cstring>

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

// CLAP entry point function signature
// This matches the CLAP SDK ABI
typedef const clap_plugin_entry* (*clap_entry_func_t)(clap_version_t);

ClapPluginLibrary::ClapPluginLibrary(const std::filesystem::path& path)
    : _path(path)
    , _handle(nullptr)
    , _entry(nullptr)
    , _factory(nullptr)
    , _valid(false)
{
    if (loadLibrary()) {
        _valid = true;
        std::cout << "[ClapPluginLibrary] Loaded: " << path << std::endl;
    } else {
        std::cerr << "[ClapPluginLibrary] Failed to load: " << path << std::endl;
    }
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
    // Phase 5: Real CLAP loading
    // For now, we'll implement a minimal version that:
    // 1. Loads the shared library
    // 2. Finds the CLAP entry point
    // 3. Initializes the factory
    //
    // Note: This requires CLAP SDK headers to be available
    // For Phase 5, we'll implement the structure but may need to
    // add CLAP SDK as a dependency or include headers

    _handle = CLAP_LIB_LOAD(_path);
    if (!_handle) {
        #if defined(_WIN32)
            DWORD error = GetLastError();
            std::cerr << "[ClapPluginLibrary] LoadLibrary failed: " << error << std::endl;
        #else
            const char* error = dlerror();
            std::cerr << "[ClapPluginLibrary] dlopen failed: " << (error ? error : "unknown") << std::endl;
        #endif
        return false;
    }

    // Find CLAP entry point
    if (!initClapEntry()) {
        unloadLibrary();
        return false;
    }

    // Initialize factory
    if (!initFactory()) {
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
    // Phase 5: Find CLAP entry point
    // CLAP plugins export a function called "clap_entry" that returns the entry point
    // For .clap bundles, we need to find the actual library inside the bundle first

    // TODO: Handle .clap bundle structure (macOS: Contents/PlugIns/plugin.so)
    // For now, assume direct .so/.dylib loading

    clap_entry_func_t entryFunc = reinterpret_cast<clap_entry_func_t>(
        CLAP_LIB_SYMBOL(_handle, "clap_entry")
    );

    if (!entryFunc) {
        std::cerr << "[ClapPluginLibrary] clap_entry symbol not found" << std::endl;
        return false;
    }

    // Call entry function to get CLAP entry point
    _entry = entryFunc(CLAP_VERSION);

    if (!_entry) {
        std::cerr << "[ClapPluginLibrary] clap_entry returned nullptr (version mismatch?)" << std::endl;
        return false;
    }

    // Initialize entry
    if (_entry->init) {
        _entry->init(_path.string().c_str());
    }

    return true;
}

bool ClapPluginLibrary::initFactory() {
    if (!_entry) {
        return false;
    }

    // Get factory from entry
    if (!_entry->get_factory) {
        std::cerr << "[ClapPluginLibrary] Entry does not provide get_factory" << std::endl;
        return false;
    }

    _factory = static_cast<const clap_plugin_factory*>(
        _entry->get_factory(CLAP_PLUGIN_FACTORY_ID)
    );

    if (!_factory) {
        std::cerr << "[ClapPluginLibrary] Failed to get plugin factory" << std::endl;
        return false;
    }

    return true;
}

const clap_plugin_descriptor* ClapPluginLibrary::getDescriptor(const char* pluginId) const noexcept {
    if (!_factory || !pluginId) {
        return nullptr;
    }

    if (!_factory->get_plugin_count || !_factory->get_plugin_descriptor) {
        return nullptr;
    }

    const uint32_t count = _factory->get_plugin_count(_factory);
    for (uint32_t i = 0; i < count; ++i) {
        const clap_plugin_descriptor* desc = _factory->get_plugin_descriptor(_factory, i);
        if (desc && desc->id && std::strcmp(desc->id, pluginId) == 0) {
            return desc;
        }
    }

    return nullptr;
}

std::vector<const clap_plugin_descriptor*> ClapPluginLibrary::getAllDescriptors() const {
    std::vector<const clap_plugin_descriptor*> descriptors;

    if (!_factory || !_factory->get_plugin_count || !_factory->get_plugin_descriptor) {
        return descriptors;
    }

    const uint32_t count = _factory->get_plugin_count(_factory);
    for (uint32_t i = 0; i < count; ++i) {
        const clap_plugin_descriptor* desc = _factory->get_plugin_descriptor(_factory, i);
        if (desc) {
            descriptors.push_back(desc);
        }
    }

    return descriptors;
}

