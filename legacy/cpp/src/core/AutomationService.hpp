#pragma once

/// AutomationService - Manages automation curves for engine parameters
///
/// Thread: Control thread (main thread)
/// Ownership: Owned by EngineHost
/// Communication:
///   - Updated by IPC thread (AutomationDomain handlers)
///   - Read by audio thread via lock-free atomic operations
///   - Provides evaluation API for real-time automation
///
/// Architecture note: Automation targets can be:
/// - Node parameters (nodeId:parameter)
/// - Channel parameters (channelId:parameter)
/// Targets come from Pulse's graph model and represent processing elements,
/// not track-level concepts.

// Forward declaration
struct AutomationData;
enum class AutomationCurveType;

#include <atomic>
#include <unordered_map>
#include <mutex>
#include <string>
#include <vector>

/// Automation curve point (time in samples, value, interpolation shape)
struct AutomationCurvePoint {
    uint64_t timeSamples;
    float value;
    std::string shape; // "step", "linear", "easeIn", "easeOut", "sCurve"
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
    /// @deprecated Use getParameterValue() after beginBlock() for real-time safe access
    float getCurrentValue(const std::string& targetId, const std::string& parameter) const;

    /// Get parameter value for current block (lock-free, called after beginBlock())
    /// Returns the automation value for the current block, or default if no automation exists
    /// @param targetId Target identifier (node ID, track ID, etc.)
    /// @param parameterId Parameter identifier within the target
    /// @return Parameter value (normalised 0.0..1.0 for most parameters, -1.0..1.0 for spatial.balance)
    float getParameterValue(const std::string& targetId, const std::string& parameterId) const;

    /// Update current values for all targets based on sample position (called periodically)
    /// @deprecated Use beginBlock() instead
    void updateCurrentValues(uint64_t samplePosition);

    /// Begin automation block evaluation (called once per audio block on audio thread)
    /// Pre-computes all parameter values for the block and stores them for lock-free reads
    /// @param blockStartSamples Start sample position of the block
    /// @param blockSize Number of samples in the block
    /// @param sampleRate Sample rate (for timebase conversion if needed)
    void beginBlock(uint64_t blockStartSamples, int blockSize, double sampleRate);

    /// Load automation snapshot from Pulse (converts events to curves)
    /// Called on control thread when automation snapshot is received
    /// @param snapshot Automation snapshot with events (nodeId:paramId format)
    void loadSnapshot(const AutomationData& snapshot);

    /// Set transport position (for timebase conversion)
    /// Called on control thread when transport position changes
    void setTransportPosition(uint64_t positionSamples);

private:
    /// Get or create automation state for a target
    TargetAutomationState* getOrCreateState(const std::string& targetId, const std::string& parameter);

    /// Evaluate a curve at a given sample position
    float evaluateCurve(const std::vector<AutomationCurvePoint>& points, uint64_t samplePosition) const;

    /// Apply interpolation shape to t (0.0 to 1.0)
    float applyInterpolationShape(float t, const std::string& shape) const;

    mutable std::mutex _mutex; // Protects _curves map structure
    std::unordered_map<std::string, std::unique_ptr<TargetAutomationState>> _curves;

    /// Key format: "targetId:parameter"
    std::string makeKey(const std::string& targetId, const std::string& parameter) const;

    /// Block snapshot for lock-free reads (updated in beginBlock, read-only from audio thread)
    struct BlockSnapshot {
        std::unordered_map<std::string, float> values; // key = "targetId:parameterId"
        uint64_t blockStartSamples = 0;
        int blockSize = 0;
    };
    std::atomic<BlockSnapshot*> _activeSnapshot; // Atomic pointer swap for lock-free reads
    BlockSnapshot _snapshotA; // Double-buffer A
    BlockSnapshot _snapshotB; // Double-buffer B
    bool _useSnapshotA = true; // Toggle between A and B

    /// Transport position (for timebase conversion)
    std::atomic<uint64_t> _transportPositionSamples;
};
