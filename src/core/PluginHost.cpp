#include "core/PluginHost.hpp"
#include "clap/ClapPluginInstance.hpp"
#include "clap/ClapRegistry.hpp"
#include "clap/clap.h"
#include "logging/Logging.hpp"
#include <sstream>

PluginHost::PluginHost() {
    _clapRegistry = std::make_unique<ClapRegistry>();
    // Phase 5: Defer plugin scanning until after server starts
    // This prevents Signal from crashing before it can accept connections
    LOG_INFO({"PluginHost"}, "Created (plugin scanning deferred)");
}

PluginHost::~PluginHost() {
    LOG_DEBUG({"PluginHost"}, "Destroyed");
}

void PluginHost::scanPlugins() {
    // Phase 5: Scan for CLAP plugins after server starts
    // Wrap in try-catch to prevent crashes from bad plugins
    LOG_INFO({"PluginHost"}, "Starting CLAP plugin scan...");
    try {
        _clapRegistry->scanDefaultPaths();
        size_t pluginCount = _clapRegistry->listPlugins().size();
        std::ostringstream msg;
        msg << "Plugin scan complete - found " << pluginCount << " CLAP plugin(s)";
        LOG_INFO({"PluginHost"}, msg.str());
    } catch (const std::exception& e) {
        LOG_ERROR({"PluginHost"}, std::string("Exception during plugin scanning: ") + e.what());
        std::ostringstream msg;
        msg << "Continuing with " << _clapRegistry->listPlugins().size() << " successfully loaded plugins";
        LOG_INFO({"PluginHost"}, msg.str());
    } catch (...) {
        LOG_ERROR({"PluginHost"}, "Unknown exception during plugin scanning, continuing anyway");
        std::ostringstream msg;
        msg << "Continuing with " << _clapRegistry->listPlugins().size() << " successfully loaded plugins";
        LOG_INFO({"PluginHost"}, msg.str());
    }
}

std::unique_ptr<PluginInstance> PluginHost::createInstance(const PluginDescriptor& desc) {
    std::ostringstream msg;
    msg << "Creating instance for plugin: " << desc.id << " (format: " << static_cast<int>(desc.format) << ")";
    LOG_DEBUG({"PluginHost"}, msg.str());

    switch (desc.format) {
        case PluginFormat::Clap:
            {
                // Phase 5: Use registry to find plugin
                auto library = _clapRegistry->getLibrary(desc.id);
                if (!library) {
                    LOG_ERROR({"PluginHost"}, std::string("Plugin not found in registry: ") + desc.id);
                    return nullptr;
                }

                const clap_plugin_descriptor* clapDesc = library->getDescriptor(desc.id.c_str());
                if (!clapDesc) {
                    LOG_ERROR({"PluginHost"}, std::string("CLAP descriptor not found for: ") + desc.id);
                    return nullptr;
                }

                auto instance = createClapInstance(library, clapDesc);
                if (instance) {
                    LOG_DEBUG({"PluginHost"}, std::string("Successfully created CLAP instance: ") + desc.id);
                } else {
                    LOG_ERROR({"PluginHost"}, std::string("Failed to create CLAP instance: ") + desc.id);
                }
                return instance;
            }
        case PluginFormat::Vst3:
        case PluginFormat::Au:
        case PluginFormat::Lv2:
        case PluginFormat::Native:
            // Not implemented in Phase 5
            LOG_ERROR({"PluginHost"}, std::string("Plugin format not yet supported: ") + std::to_string(static_cast<int>(desc.format)));
            return nullptr;
        default:
            LOG_ERROR({"PluginHost"}, std::string("Unknown plugin format: ") + std::to_string(static_cast<int>(desc.format)));
            return nullptr;
    }
}

bool PluginHost::isFormatSupported(PluginFormat format) const {
    return format == PluginFormat::Clap;
}

