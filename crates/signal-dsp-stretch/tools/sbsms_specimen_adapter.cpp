// Clean-room adapter for the pinned external SBSMS 2.3.0 research specimen.
// This file contains no SBSMS implementation code or numeric policy.

#include <sbsms.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <limits>
#include <memory>
#include <string>
#include <vector>

#include <sys/resource.h>

namespace {

using _sbsms_::SBSMS;
using _sbsms_::SBSMSInterface;
using _sbsms_::SBSMSQuality;
using _sbsms_::SBSMSQualityStandard;
using _sbsms_::SBSMSRenderer;
using _sbsms_::SBSMSTrack;
using _sbsms_::SampleCountType;
using _sbsms_::TimeType;
using _sbsms_::audio;

constexpr const char *kPinnedRevision =
    "e99cd7e6c6367e476577be34d2fdbe2023904d7e";

[[noreturn]] void fail(const std::string &message) {
  std::cerr << "sbsms specimen adapter: " << message << '\n';
  std::exit(2);
}

std::size_t parse_size(const char *text, const char *name) {
  char *end = nullptr;
  const auto value = std::strtoull(text, &end, 10);
  if (end == text || *end != '\0' || value == 0 ||
      value > std::numeric_limits<std::size_t>::max()) {
    fail(std::string("invalid ") + name);
  }
  return static_cast<std::size_t>(value);
}

double parse_ratio(const char *text) {
  char *end = nullptr;
  const double value = std::strtod(text, &end);
  if (end == text || *end != '\0' || !std::isfinite(value) || value <= 0.0) {
    fail("invalid ratio");
  }
  return value;
}

std::vector<float> read_raw(const std::string &path, std::size_t samples) {
  std::ifstream input(path, std::ios::binary | std::ios::ate);
  if (!input) {
    fail("cannot open input " + path);
  }
  const auto expected = static_cast<std::streamoff>(samples * sizeof(float));
  if (input.tellg() != expected) {
    fail("input byte length does not match frames and channels");
  }
  input.seekg(0);
  std::vector<float> values(samples);
  input.read(reinterpret_cast<char *>(values.data()), expected);
  if (!input) {
    fail("cannot read input " + path);
  }
  return values;
}

void write_raw(const std::string &path, const std::vector<float> &values) {
  std::ofstream output(path, std::ios::binary | std::ios::trunc);
  if (!output) {
    fail("cannot open output " + path);
  }
  output.write(reinterpret_cast<const char *>(values.data()),
               static_cast<std::streamsize>(values.size() * sizeof(float)));
  if (!output) {
    fail("cannot write output " + path);
  }
}

class Input final : public SBSMSInterface {
public:
  Input(const std::vector<float> &interleaved, std::size_t frames,
        std::size_t channels, double ratio)
      : interleaved_(interleaved), frames_(frames), channels_(channels),
        ratio_(ratio), target_(static_cast<SampleCountType>(
                           std::llround(static_cast<double>(frames) * ratio))) {}

  long samples(audio *buffer, long requested) override {
    const auto available = frames_ - position_;
    const auto count = std::min<std::size_t>(available, requested);
    for (std::size_t index = 0; index < count; ++index) {
      buffer[index][0] = interleaved_[(position_ + index) * channels_];
      buffer[index][1] = channels_ == 2
                             ? interleaved_[(position_ + index) * channels_ + 1]
                             : 0.0F;
    }
    position_ += count;
    return static_cast<long>(count);
  }

  float getStretch(float) override { return static_cast<float>(ratio_); }
  float getMeanStretch(float, float) override {
    return static_cast<float>(ratio_);
  }
  float getPitch(float) override { return 1.0F; }
  long getPresamples() override { return 0; }
  SampleCountType getSamplesToInput() override {
    return static_cast<SampleCountType>(frames_);
  }
  SampleCountType getSamplesToOutput() override { return target_; }

private:
  const std::vector<float> &interleaved_;
  std::size_t frames_;
  std::size_t channels_;
  double ratio_;
  SampleCountType target_;
  std::size_t position_ = 0;
};

struct TrackStats final : SBSMSRenderer {
  void startFrame() override {
    ++frames;
    tracks_this_frame = 0;
  }

  void startTime(int, const TimeType &time, int) override {
    current_time = time;
    tracks_this_time = 0;
    ++time_groups;
  }

  void render(int, SBSMSTrack *track) override {
    ++track_visits;
    ++tracks_this_time;
    ++tracks_this_frame;
    births += track->isFirst(current_time) ? 1 : 0;
    deaths += track->isLast(current_time) ? 1 : 0;
  }

  void endTime(int) override {
    maximum_tracks_per_time =
        std::max(maximum_tracks_per_time, tracks_this_time);
  }

  void endFrame() override {
    maximum_track_visits_per_frame_callback =
        std::max(maximum_track_visits_per_frame_callback, tracks_this_frame);
  }

  std::uint64_t frames = 0;
  std::uint64_t time_groups = 0;
  std::uint64_t track_visits = 0;
  std::uint64_t births = 0;
  std::uint64_t deaths = 0;
  std::uint64_t maximum_tracks_per_time = 0;
  std::uint64_t maximum_track_visits_per_frame_callback = 0;
  std::uint64_t maximum_track_visits_per_output_read = 0;
  std::uint64_t tracks_this_time = 0;
  std::uint64_t tracks_this_frame = 0;
  TimeType current_time = 0;
};

void write_stats(const std::string &path, const TrackStats &stats,
                 std::size_t source_frames, std::size_t output_frames,
                 std::size_t channels, double ratio, double elapsed_seconds,
                 std::uint64_t peak_rss_bytes) {
  std::ofstream output(path, std::ios::trunc);
  if (!output) {
    fail("cannot open stats " + path);
  }
  output << "specimen_revision\t" << kPinnedRevision << '\n'
         << "source_frames\t" << source_frames << '\n'
         << "output_frames\t" << output_frames << '\n'
         << "channels\t" << channels << '\n'
         << "ratio\t" << ratio << '\n'
         << "elapsed_seconds\t" << elapsed_seconds << '\n'
         << "peak_rss_bytes\t" << peak_rss_bytes << '\n'
         << "synthesis_frames\t" << stats.frames << '\n'
         << "time_groups\t" << stats.time_groups << '\n'
         << "track_visits\t" << stats.track_visits << '\n'
         << "track_births\t" << stats.births << '\n'
         << "track_deaths\t" << stats.deaths << '\n'
         << "maximum_tracks_per_time\t" << stats.maximum_tracks_per_time
         << '\n'
         << "maximum_track_visits_per_frame_callback\t"
         << stats.maximum_track_visits_per_frame_callback << '\n'
         << "maximum_track_visits_per_output_read\t"
         << stats.maximum_track_visits_per_output_read << '\n';
}

} // namespace

int main(int argc, char **argv) {
  if (argc == 2 && std::string(argv[1]) == "--version") {
    std::cout << "sbsms-specimen-adapter 2.3.0 " << kPinnedRevision << '\n';
    return 0;
  }
  if (argc != 7) {
    fail("usage: INPUT.raw OUTPUT.raw FRAMES CHANNELS RATIO STATS.tsv");
  }

  const std::size_t frames = parse_size(argv[3], "frames");
  const std::size_t channels = parse_size(argv[4], "channels");
  if (channels != 1 && channels != 2) {
    fail("channels must be 1 or 2");
  }
  const double ratio = parse_ratio(argv[5]);
  const std::size_t target =
      static_cast<std::size_t>(std::llround(static_cast<double>(frames) * ratio));
  const auto input_values = read_raw(argv[1], frames * channels);
  Input input(input_values, frames, channels, ratio);
  SBSMSQuality quality(&SBSMSQualityStandard);
  SBSMS engine(static_cast<int>(channels), &quality, true);
  TrackStats stats;
  engine.addRenderer(&stats);

  constexpr std::size_t kChunk = 4096;
  std::vector<float> output_values(target * channels);
  auto buffer = std::make_unique<audio[]>(kChunk);
  std::size_t produced = 0;
  const auto started = std::chrono::steady_clock::now();
  while (produced < target) {
    const auto count = std::min(kChunk, target - produced);
    const auto visits_before = stats.track_visits;
    const long received = engine.read(&input, buffer.get(), static_cast<long>(count));
    if (received != static_cast<long>(count)) {
      fail("specimen returned a short output block");
    }
    for (std::size_t index = 0; index < count; ++index) {
      for (std::size_t channel = 0; channel < channels; ++channel) {
        output_values[(produced + index) * channels + channel] =
            buffer[index][channel];
      }
    }
    stats.maximum_track_visits_per_output_read =
        std::max(stats.maximum_track_visits_per_output_read,
                 stats.track_visits - visits_before);
    produced += count;
  }
  const double elapsed = std::chrono::duration<double>(
                             std::chrono::steady_clock::now() - started)
                             .count();
  rusage usage{};
  if (getrusage(RUSAGE_SELF, &usage) != 0) {
    fail("getrusage failed");
  }
#if defined(__APPLE__)
  const auto peak_rss_bytes = static_cast<std::uint64_t>(usage.ru_maxrss);
#else
  const auto peak_rss_bytes =
      static_cast<std::uint64_t>(usage.ru_maxrss) * 1024U;
#endif

  engine.removeRenderer(&stats);
  write_raw(argv[2], output_values);
  write_stats(argv[6], stats, frames, target, channels, ratio, elapsed,
              peak_rss_bytes);
  return 0;
}
