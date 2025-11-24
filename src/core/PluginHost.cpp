#include "core/PluginHost.hpp"
#include "clap/ClapPluginInstance.hpp"
#include <iostream>

PluginHost::PluginHost() {
    std::cout << "[PluginHost] Created" << std::endl;
}

PluginHost::~PluginHost() {
    std::cout << "[PluginHost] Destroyed" << std::endl;
}

std::unique_ptr<PluginInstance> PluginHost::createInstance(const PluginDescriptor& desc) {
    std::cout << "[PluginHost] Creating instance for plugin: " << desc.id << " (format: " << static_cast<int>(desc.format) << ")" << std::endl;
    switch (desc.format) {
        case PluginFormat::Clap:
            {
                auto instance = createClapInstance(desc);
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
            // Not implemented in Phase 4
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

