#include "core/PluginCrashTracking.hpp"
#include <cstring>

// Global variables for tracking plugin loading state (for crash reporting)
volatile bool g_inPluginLoading = false;
char g_currentPluginPath[1024] = {0};

// Jump buffer for recovering from bus errors
sigjmp_buf g_pluginLoadJumpBuf;
volatile bool g_pluginLoadJumpSet = false;

