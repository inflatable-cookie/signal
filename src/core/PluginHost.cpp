#include "core/PluginHost.hpp"
#include "clap/ClapPluginInstance.hpp"
#include "clap/ClapRegistry.hpp"
#include "clap/clap.h"
#include <iostream>

PluginHost::PluginHost() {
    _clapRegistry = std::make_unique<ClapRegistry>();
    // Phase 5: Defer plugin scanning until after server starts
    // This prevents Signal from crashing before it can accept connections
    std::cout << "[PluginHost] Created (plugin scanning deferred)" << std::endl;
}

PluginHost::~PluginHost() {
    std::cout << "[PluginHost] Destroyed" << std::endl;
}

void PluginHost::scanPlugins() {
    // Phase 5: Scan for CLAP plugins after server starts
    // Wrap in try-catch to prevent crashes from bad plugins
    std::cout << "[PluginHost] Starting CLAP plugin scan..." << std::endl;
    std::cout.flush();
    try {
        _clapRegistry->scanDefaultPaths();
        size_t pluginCount = _clapRegistry->listPlugins().size();
        std::cout << "[PluginHost] Plugin scan complete - found " << pluginCount << " CLAP plugin(s)" << std::endl;
        std::cout.flush();
    } catch (const std::exception& e) {
        std::cerr << "[PluginHost] Exception during plugin scanning: " << e.what() << std::endl;
        std::cerr.flush();
        std::cerr << "[PluginHost] Continuing with " << _clapRegistry->listPlugins().size() << " successfully loaded plugins" << std::endl;
        std::cerr.flush();
    } catch (...) {
        std::cerr << "[PluginHost] Unknown exception during plugin scanning, continuing anyway" << std::endl;
        std::cerr.flush();
        std::cerr << "[PluginHost] Continuing with " << _clapRegistry->listPlugins().size() << " successfully loaded plugins" << std::endl;
        std::cerr.flush();
    }
}

std::unique_ptr<PluginInstance> PluginHost::createInstance(const PluginDescriptor& desc) {
    std::cout << "[PluginHost] Creating instance for plugin: " << desc.id << " (format: " << static_cast<int>(desc.format) << ")" << std::endl;

    switch (desc.format) {
        case PluginFormat::Clap:
            {
                // Phase 5: Use registry to find plugin
                auto library = _clapRegistry->getLibrary(desc.id);
                if (!library) {
                    std::cerr << "[PluginHost] Plugin not found in registry: " << desc.id << std::endl;
                    return nullptr;
                }

                const clap_plugin_descriptor* clapDesc = library->getDescriptor(desc.id.c_str());
                if (!clapDesc) {
                    std::cerr << "[PluginHost] CLAP descriptor not found for: " << desc.id << std::endl;
                    return nullptr;
                }

                auto instance = createClapInstance(library, clapDesc);
                if (instance) {
                    std::cout << "[PluginHost] Successfully created CLAP instance: " << desc.id << std::endl;
                } else {
                    std::cerr << "[PluginHost] Failed to create CLAP instance: " << desc.id << std::endl;
                }
                return instance;
            }
        case PluginFormat::Vst3:
        case PluginFormat::Au:
        case PluginFormat::Lv2:
        case PluginFormat::Native:
            // Not implemented in Phase 5
            std::cerr << "[PluginHost] Plugin format not yet supported: " << static_cast<int>(desc.format) << std::endl;
            return nullptr;
        default:
            std::cerr << "[PluginHost] Unknown plugin format: " << static_cast<int>(desc.format) << std::endl;
            return nullptr;
    }
}

bool PluginHost::isFormatSupported(PluginFormat format) const {
    return format == PluginFormat::Clap;
}

