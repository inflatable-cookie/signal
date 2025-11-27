#ifndef LOOPHOLE_SIGNAL_LOGGING_HPP
#define LOOPHOLE_SIGNAL_LOGGING_HPP

#include <string>
#include <initializer_list>
#include <cstdint>

/**
 * Unified log level system (1-8) matching Aura and Pulse.
 * Higher numbers mean more verbose logging.
 */
enum class LogLevel : uint8_t {
    Core = 1,
    Error = 2,
    Warn = 3,
    Info = 4,
    Debug = 5,
    Verbose = 6,
    Trace = 7,
    All = 8
};

/**
 * Initialize logging system.
 * Reads DEBUG_LEVEL from environment variable, defaults to 4 (Info).
 */
void initLogging();

/**
 * Get current DEBUG_LEVEL.
 */
uint8_t getDebugLevel();

/**
 * Check if a log at the given level should be emitted.
 */
bool shouldLog(LogLevel level);

/**
 * Log a message with the given level and areas.
 * Format: [Signal][LevelName][Area1][Area2] message...
 */
void log(LogLevel level, std::initializer_list<std::string> areas, const std::string& message);

// Convenience inline functions for each log level
// Usage: LOG_INFO({"Engine", "Transport"}, "Message");
inline void LOG_CORE(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Core, areas, msg);
}
inline void LOG_ERROR(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Error, areas, msg);
}
inline void LOG_WARN(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Warn, areas, msg);
}
inline void LOG_INFO(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Info, areas, msg);
}
inline void LOG_DEBUG(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Debug, areas, msg);
}
inline void LOG_VERBOSE(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Verbose, areas, msg);
}
inline void LOG_TRACE(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::Trace, areas, msg);
}
inline void LOG_ALL(std::initializer_list<std::string> areas, const std::string& msg) {
    log(LogLevel::All, areas, msg);
}

#endif // LOOPHOLE_SIGNAL_LOGGING_HPP

