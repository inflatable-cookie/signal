#pragma once

/// EngineSelfTest - Offline render sanity check for diagnostics
///
/// Thread: Control thread (IPC handler context)
/// Ownership: Stateless helper functions
///
/// This module provides a minimal self-test harness that runs synthetic
/// scenarios offline without touching the live engine graph or audio device.

#include <string>
#include <vector>

struct EngineSelfTestScenarioResult {
    std::string id;
    bool ok = false;
    float maxAbsSample = 0.0f;
};

struct EngineSelfTestResult {
    bool ok = false;
    std::vector<EngineSelfTestScenarioResult> scenarios;
};

/// Runs a small set of synthetic scenarios offline, without touching the live engine graph.
/// Returns a result summary with pass/fail status for each scenario.
EngineSelfTestResult runEngineSelfTest();

