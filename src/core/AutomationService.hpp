#pragma once

/// AutomationService - Manages automation curves for engine parameters
///
/// Thread: Control thread (main thread)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (AutomationDomain handlers)
///   - Read by audio thread via lock-free atomic operations
///   - Provides evaluation API for real-time automation

#include <atomic>
#include <unordered_map>
#include <mutex>
#include <string>
#include <vector>

/// Automation curve point (time in samples, value)
struct AutomationCurvePoint {
    uint64_t timeSamples;
    float value;
};

/// Automation curve for a parameter
struct AutomationCurve {
    std::string targetId;
    std::string parameter;
    std::vector<AutomationCurvePoint> points; // Sorted by timeSamples
};

/// Per-target automation state (atomic for lock-free audio thread access)
struct TargetAutomationState {
    std::string targetId;
    std::string parameter;
    std::atomic<bool> hasCurve;
    std::atomic<float> currentValue; // Current automation value (updated by control thread)

    // Curve data (read-only from audio thread after update)
    std::vector<AutomationCurvePoint> points; // Protected by mutex during updates
    mutable std::mutex pointsMutex; // Mutable to allow locking in const methods

    TargetAutomationState()
        : hasCurve(false)
        , currentValue(1.0f) // Default unity gain
    {
    }
};

class AutomationService {
public:
    AutomationService();
    ~AutomationService();

    /// Set automation curves for session (replaces all existing curves)
    void setCurvesForSession(const std::vector<AutomationCurve>& curves);

    /// Update a single curve
    void updateCurve(const AutomationCurve& curve);

    /// Evaluate automation value for a target at a given sample position
    /// Returns the automation value, or 1.0 (unity) if no automation exists
    float evaluateAt(const std::string& targetId, const std::string& parameter, uint64_t samplePosition) const;

    /// Get current automation value for a target (cached, updated by control thread)
    float getCurrentValue(const std::string& targetId, const std::string& parameter) const;

    /// Update current values for all targets based on sample position (called periodically)
    void updateCurrentValues(uint64_t samplePosition);

private:
    /// Get or create automation state for a target
    TargetAutomationState* getOrCreateState(const std::string& targetId, const std::string& parameter);

    /// Evaluate a curve at a given sample position
    float evaluateCurve(const std::vector<AutomationCurvePoint>& points, uint64_t samplePosition) const;

    mutable std::mutex _mutex; // Protects _curves map structure
    std::unordered_map<std::string, std::unique_ptr<TargetAutomationState>> _curves;

    /// Key format: "targetId:parameter"
    std::string makeKey(const std::string& targetId, const std::string& parameter) const;
};

