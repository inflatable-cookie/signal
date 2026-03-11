#include "Logging.hpp"
#include <cstdlib>
#include <iostream>
#include <sstream>

static uint8_t g_debugLevel = 4; // Default to Info (4)

void initLogging() {
    const char* envLevel = std::getenv("DEBUG_LEVEL");
    if (envLevel != nullptr) {
        int parsed = std::atoi(envLevel);
        if (parsed >= 1 && parsed <= 8) {
            g_debugLevel = static_cast<uint8_t>(parsed);
        }
    }
}

uint8_t getDebugLevel() {
    return g_debugLevel;
}

bool shouldLog(LogLevel level) {
    return static_cast<uint8_t>(level) <= g_debugLevel;
}

const char* levelName(LogLevel level) {
    switch (level) {
        case LogLevel::Core: return "Core";
        case LogLevel::Error: return "Error";
        case LogLevel::Warn: return "Warn";
        case LogLevel::Info: return "Info";
        case LogLevel::Debug: return "Debug";
        case LogLevel::Verbose: return "Verbose";
        case LogLevel::Trace: return "Trace";
        case LogLevel::All: return "All";
        default: return "Unknown";
    }
}

void log(LogLevel level, std::initializer_list<std::string> areas, const std::string& message) {
    if (!shouldLog(level)) {
        return;
    }

    // Format: [Signal][LevelName][Area1][Area2] message...
    std::ostringstream prefix;
    prefix << "[Signal][" << levelName(level);

    for (const auto& area : areas) {
        prefix << "][" << area;
    }

    prefix << "] " << message;

    // Output to appropriate stream based on level
    if (level == LogLevel::Core || level == LogLevel::Error) {
        std::cerr << prefix.str() << std::endl;
    } else {
        std::cout << prefix.str() << std::endl;
    }
}

