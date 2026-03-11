#pragma once
#include <setjmp.h>
#include <csignal>

/// Plugin crash tracking - Global state for identifying which plugin causes crashes
///
/// These globals are set during plugin loading to help identify problematic plugins
/// in signal handlers (SIGBUS, SIGSEGV).

extern volatile bool g_inPluginLoading;
extern char g_currentPluginPath[1024];

// Jump buffer for recovering from bus errors during plugin loading
extern sigjmp_buf g_pluginLoadJumpBuf;
extern volatile bool g_pluginLoadJumpSet;

