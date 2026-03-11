#pragma once

/// ClapPluginLibrary - Represents a loaded CLAP plugin library
///
/// Thread: Control thread only
/// Ownership: Shared ownership via std::shared_ptr (kept alive by registry and instances)
///
/// This class handles loading and managing a CLAP plugin library (.clap bundle or .so/.dylib).
/// It provides access to the CLAP entry point and plugin factory.

#include "clap/clap.h"
#include <string>
#include <filesystem>
#include <memory>
#include <vector>

/// CLAP plugin library wrapper
class ClapPluginLibrary {
public:
    /// Load a CLAP plugin library from a path
    /// @param path Path to .clap bundle or .so/.dylib file
    explicit ClapPluginLibrary(const std::filesystem::path& path);
    ~ClapPluginLibrary();

    // Non-copyable, movable
    ClapPluginLibrary(const ClapPluginLibrary&) = delete;
    ClapPluginLibrary& operator=(const ClapPluginLibrary&) = delete;
    ClapPluginLibrary(ClapPluginLibrary&&) noexcept;
    ClapPluginLibrary& operator=(ClapPluginLibrary&&) noexcept;

    /// Check if library is valid and loaded
    bool isValid() const noexcept { return _valid; }

    /// Get the CLAP entry point (for initializing CLAP host)
    const clap_plugin_entry_t* getEntry() const noexcept { return _entry; }

    /// Get the CLAP plugin factory
    /// @return Factory pointer, or nullptr if not available
    const clap_plugin_factory_t* getFactory() const noexcept { return _factory; }

    /// Get plugin descriptor by ID
    /// @param pluginId CLAP plugin ID string
    /// @return Descriptor pointer, or nullptr if not found
    const clap_plugin_descriptor_t* getDescriptor(const char* pluginId) const noexcept;

    /// Get all plugin descriptors from this library
    /// @return Vector of descriptor pointers
    std::vector<const clap_plugin_descriptor_t*> getAllDescriptors() const;

    /// Get the library path
    const std::filesystem::path& getPath() const noexcept { return _path; }

private:
    std::filesystem::path _path;
    void* _handle; // Platform-specific library handle (dlopen/LoadLibrary)
    const clap_plugin_entry_t* _entry;
    const clap_plugin_factory_t* _factory;
    bool _valid;

    // Helper methods
    bool loadLibrary();
    void unloadLibrary();
    bool initClapEntry();
    bool initFactory();
};

