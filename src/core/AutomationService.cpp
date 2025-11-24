#include "core/AutomationService.hpp"
#include <algorithm>
#include <iostream>
#include <sstream>

AutomationService::AutomationService() {
    std::cout << "[AutomationService] Initialised" << std::endl;
}

AutomationService::~AutomationService() = default;

std::string AutomationService::makeKey(const std::string& targetId, const std::string& parameter) const {
    return targetId + ":" + parameter;
}

TargetAutomationState* AutomationService::getOrCreateState(
    const std::string& targetId,
    const std::string& parameter
) {
    std::lock_guard<std::mutex> lock(_mutex);
    std::string key = makeKey(targetId, parameter);
    auto it = _curves.find(key);
    if (it != _curves.end()) {
        return it->second.get();
    }

    // Create new state
    auto state = std::make_unique<TargetAutomationState>();
    state->targetId = targetId;
    state->parameter = parameter;
    TargetAutomationState* ptr = state.get();
    _curves[key] = std::move(state);
    return ptr;
}

void AutomationService::setCurvesForSession(const std::vector<AutomationCurve>& curves) {
    std::lock_guard<std::mutex> lock(_mutex);

    // Clear existing curves
    _curves.clear();

    // Add new curves
    for (const auto& curve : curves) {
        std::string key = makeKey(curve.targetId, curve.parameter);
        auto state = std::make_unique<TargetAutomationState>();
        state->targetId = curve.targetId;
        state->parameter = curve.parameter;

        {
            std::lock_guard<std::mutex> pointsLock(state->pointsMutex);
            state->points = curve.points;
        }

        state->hasCurve.store(!curve.points.empty(), std::memory_order_release);
        if (!curve.points.empty()) {
            state->currentValue.store(curve.points[0].value, std::memory_order_release);
        }

        _curves[key] = std::move(state);
    }

    std::cout << "[AutomationService] Set " << curves.size() << " automation curves" << std::endl;
}

void AutomationService::updateCurve(const AutomationCurve& curve) {
    std::string key = makeKey(curve.targetId, curve.parameter);
    TargetAutomationState* state = getOrCreateState(curve.targetId, curve.parameter);

    {
        std::lock_guard<std::mutex> pointsLock(state->pointsMutex);
        state->points = curve.points;
    }

    state->hasCurve.store(!curve.points.empty(), std::memory_order_release);
    if (!curve.points.empty()) {
        state->currentValue.store(curve.points[0].value, std::memory_order_release);
    }
}

float AutomationService::evaluateCurve(
    const std::vector<AutomationCurvePoint>& points,
    uint64_t samplePosition
) const {
    if (points.empty()) {
        return 1.0f; // Default unity gain
    }

    // Find points before and after this position
    auto before = std::find_if(
        points.rbegin(),
        points.rend(),
        [samplePosition](const AutomationCurvePoint& p) {
            return p.timeSamples <= samplePosition;
        }
    );

    auto after = std::find_if(
        points.begin(),
        points.end(),
        [samplePosition](const AutomationCurvePoint& p) {
            return p.timeSamples >= samplePosition;
        }
    );

    if (before != points.rend() && after != points.end()) {
        const AutomationCurvePoint& p1 = *before;
        const AutomationCurvePoint& p2 = *after;

        if (p1.timeSamples == samplePosition) {
            return p1.value;
        }

        if (p1.timeSamples < samplePosition && p2.timeSamples > samplePosition) {
            // Handle step shape: hold value until next point
            if (p1.shape == "step") {
                return p1.value;
            }

            // Calculate interpolation factor t (0.0 to 1.0)
            float t = static_cast<float>(samplePosition - p1.timeSamples) /
                      static_cast<float>(p2.timeSamples - p1.timeSamples);

            // Apply shape-based interpolation
            float shapedT = applyInterpolationShape(t, p1.shape);
            return p1.value + (p2.value - p1.value) * shapedT;
        }
    }

    if (before != points.rend()) {
        // After last point - hold last value
        return before->value;
    }

    if (after != points.end()) {
        // Before first point - use first value
        return after->value;
    }

    return 1.0f; // Default
}

float AutomationService::applyInterpolationShape(float t, const std::string& shape) const {
    // Clamp t to [0.0, 1.0]
    t = std::max(0.0f, std::min(1.0f, t));

    if (shape == "step") {
        // Step: hold value until next point (return 0.0 to use start value)
        return 0.0f;
    } else if (shape == "linear") {
        // Linear: straight line
        return t;
    } else if (shape == "easeIn") {
        // Ease In: slower start, faster end (quadratic)
        return t * t;
    } else if (shape == "easeOut") {
        // Ease Out: faster start, slower end (inverse quadratic)
        return 1.0f - (1.0f - t) * (1.0f - t);
    } else if (shape == "sCurve") {
        // S-Curve: smooth both ends (cubic ease-in-out)
        if (t < 0.5f) {
            return 2.0f * t * t;
        } else {
            return 1.0f - 2.0f * (1.0f - t) * (1.0f - t);
        }
    }

    // Default to linear if shape is unknown
    return t;
}

float AutomationService::evaluateAt(
    const std::string& targetId,
    const std::string& parameter,
    uint64_t samplePosition
) const {
    std::string key = makeKey(targetId, parameter);

    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _curves.find(key);
    if (it == _curves.end()) {
        // No automation - return default based on parameter type
        if (parameter == "pan") {
            return 0.0f; // Default centre pan
        }
        return 1.0f; // Default unity gain for other parameters
    }

    const TargetAutomationState* state = it->second.get();
    if (!state->hasCurve.load(std::memory_order_acquire)) {
        // No curve - return default based on parameter type
        if (parameter == "pan") {
            return 0.0f; // Default centre pan
        }
        return 1.0f; // Default unity gain for other parameters
    }

    // Read points (with lock)
    std::vector<AutomationCurvePoint> points;
    {
        // pointsMutex is mutable, so we can lock it even on const state
        std::lock_guard<std::mutex> pointsLock(const_cast<std::mutex&>(state->pointsMutex));
        points = state->points;
    }

    return evaluateCurve(points, samplePosition);
}

float AutomationService::getCurrentValue(
    const std::string& targetId,
    const std::string& parameter
) const {
    std::string key = makeKey(targetId, parameter);

    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _curves.find(key);
    if (it == _curves.end()) {
        return 1.0f;
    }

    const TargetAutomationState* state = it->second.get();
    return state->currentValue.load(std::memory_order_acquire);
}

void AutomationService::updateCurrentValues(uint64_t samplePosition) {
    std::lock_guard<std::mutex> lock(_mutex);

    for (auto& pair : _curves) {
        TargetAutomationState* state = pair.second.get();
        if (!state->hasCurve.load(std::memory_order_acquire)) {
            continue;
        }

        std::vector<AutomationCurvePoint> points;
        {
            std::lock_guard<std::mutex> pointsLock(state->pointsMutex);
            points = state->points;
        }

        float value = evaluateCurve(points, samplePosition);
        state->currentValue.store(value, std::memory_order_release);
    }
}

