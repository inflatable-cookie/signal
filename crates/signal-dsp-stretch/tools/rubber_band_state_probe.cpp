#include <rubberband/RubberBandStretcher.h>

#include <algorithm>
#include <bit>
#include <cmath>
#include <cstdint>
#include <iostream>
#include <limits>
#include <numeric>
#include <string>
#include <type_traits>
#include <vector>

using RubberBand::RubberBandStretcher;

namespace {

constexpr int sample_rate = 48000;
constexpr int frames = 16384;
constexpr uint64_t hash_offset = 0xcbf29ce484222325ULL;

struct Control {
    std::string id;
    std::vector<float> samples;
};

struct Mode {
    std::string id;
    RubberBandStretcher::Options options;
};

void hash_u64(uint64_t &hash, uint64_t value) {
    for (int shift = 0; shift < 64; shift += 8) {
        hash ^= (value >> shift) & 0xff;
        hash *= 0x100000001b3ULL;
    }
}

template <typename T>
uint64_t sequence_hash(const std::vector<T> &values) {
    uint64_t hash = hash_offset;
    for (const auto value : values) {
        if constexpr (std::is_same_v<T, float>) {
            hash_u64(hash, std::bit_cast<uint32_t>(value));
        } else {
            hash_u64(hash, static_cast<uint64_t>(static_cast<int64_t>(value)));
        }
    }
    return hash;
}

std::vector<float> impulses(const std::vector<std::pair<int, float>> &events) {
    std::vector<float> samples(frames, 0.0f);
    for (const auto &[frame, value] : events) samples[frame] = value;
    return samples;
}

std::vector<float> soft_onset() {
    std::vector<float> samples(frames, 0.0f);
    for (int i = frames / 2; i < frames; ++i) {
        const float attack = std::min(1.0f, float(i - frames / 2) / 512.0f);
        const float envelope = 0.5f - 0.5f * std::cos(float(M_PI) * attack);
        samples[i] = 0.5f * envelope * std::sin(2.0f * float(M_PI) * 440.0f * i / sample_rate);
    }
    return samples;
}

std::vector<float> tonal_impulse() {
    std::vector<float> samples(frames);
    for (int i = 0; i < frames; ++i) {
        samples[i] = 0.2f * std::sin(2.0f * float(M_PI) * 220.0f * i / sample_rate);
    }
    samples[frames / 2] += 0.8f;
    return samples;
}

std::vector<Control> controls() {
    return {
        {"hard-impulse", impulses({{frames / 2, 1.0f}})},
        {"dense-impulses", impulses({{frames / 2 - 128, 1.0f}, {frames / 2 + 128, 0.8f}})},
        {"soft-onset", soft_onset()},
        {"tonal-impulse", tonal_impulse()},
    };
}

std::vector<Mode> modes() {
    return {
        {"r2-default", RubberBandStretcher::OptionProcessOffline |
                           RubberBandStretcher::OptionEngineFaster |
                           RubberBandStretcher::OptionTransientsCrisp |
                           RubberBandStretcher::OptionPhaseLaminar},
        {"r2-no-reset", RubberBandStretcher::OptionProcessOffline |
                            RubberBandStretcher::OptionEngineFaster |
                            RubberBandStretcher::OptionTransientsSmooth |
                            RubberBandStretcher::OptionPhaseLaminar},
        {"r2-no-lamination", RubberBandStretcher::OptionProcessOffline |
                                  RubberBandStretcher::OptionEngineFaster |
                                  RubberBandStretcher::OptionTransientsCrisp |
                                  RubberBandStretcher::OptionPhaseIndependent},
    };
}

template <typename T>
T minimum(const std::vector<T> &values) {
    return values.empty() ? T{} : *std::min_element(values.begin(), values.end());
}

template <typename T>
T maximum(const std::vector<T> &values) {
    return values.empty() ? T{} : *std::max_element(values.begin(), values.end());
}

template <typename T>
void print_values(const std::vector<T> &values) {
    for (size_t index = 0; index < values.size(); ++index) {
        if (index) std::cout << ',';
        std::cout << values[index];
    }
}

void report(const Control &control, const Mode &mode, double ratio) {
    RubberBandStretcher stretcher(sample_rate, 1, mode.options, ratio, 1.0);
    stretcher.setExpectedInputDuration(control.samples.size());
    const float *channels[] = {control.samples.data()};
    stretcher.study(channels, control.samples.size(), true);
    stretcher.calculateStretch();

    const auto increments = stretcher.getOutputIncrements();
    const auto resets = stretcher.getPhaseResetCurve();
    const auto exact = stretcher.getExactTimePoints();
    const auto increment_sum = std::accumulate(increments.begin(), increments.end(), int64_t{});
    const auto negative = std::count_if(increments.begin(), increments.end(), [](int value) { return value < 0; });
    const auto zero = std::count(increments.begin(), increments.end(), 0);
    const auto reset_max = maximum(resets);
    const auto reset_nonzero = std::count_if(resets.begin(), resets.end(), [](float value) { return value > 0.0f; });

    std::cout << control.id << '\t' << ratio << '\t' << mode.id << '\t'
              << stretcher.getEngineVersion() << '\t' << stretcher.getInputIncrement() << '\t'
              << increments.size() << '\t' << minimum(increments) << '\t' << maximum(increments)
              << '\t' << increment_sum << '\t' << negative << '\t' << zero << '\t'
              << std::hex << sequence_hash(increments) << std::dec << '\t'
              << resets.size() << '\t' << reset_max << '\t' << reset_nonzero << '\t'
              << std::hex << sequence_hash(resets) << std::dec << '\t'
              << exact.size() << '\t' << std::hex << sequence_hash(exact) << std::dec << '\t';
    print_values(increments);
    std::cout << '\t';
    print_values(resets);
    std::cout << '\t';
    print_values(exact);
    std::cout << '\n';
}

}  // namespace

int main() {
    std::cout << "control\tratio\tmode\tengine\tinput_increment\toutput_increment_count"
                 "\toutput_increment_min\toutput_increment_max\toutput_increment_sum"
                 "\tnegative_increments\tzero_increments\toutput_increment_hash"
                 "\treset_count\treset_max\treset_nonzero\treset_hash"
                 "\texact_time_count\texact_time_hash\toutput_increments"
                 "\treset_curve\texact_time_points\n";
    for (const auto &control : controls()) {
        for (const double ratio : {1.0, 0.75, 1.25, 1.5}) {
            for (const auto &mode : modes()) report(control, mode, ratio);
        }
    }
}
