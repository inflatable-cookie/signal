#include <algorithm>
#include <cmath>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <string>
#include <vector>

#define private public
#include "signalsmith-stretch.h"
#undef private

using Stretch = signalsmith::stretch::SignalsmithStretch<float>;
using Complex = std::complex<float>;

struct Channels {
	float *samples;
	float *operator[](int) const { return samples; }
};

static float quantized(double sample) {
	double upper = double(INT16_MAX)/32768.0;
	sample = std::clamp(sample, -1.0, upper);
	return float(std::round(sample*32768.0)/32768.0);
}

static std::vector<float> control(const std::string &name, int frames, int sampleRate) {
	std::vector<float> result(frames);
	const double frequencies[] = {110.0, 164.8138, 220.0, 329.6276};
	for (int i = 0; i < frames; ++i) {
		double sample = 0;
		if (name == "chord") {
			for (int tone = 0; tone < 4; ++tone) {
				sample += std::sin(2*M_PI*frequencies[tone]*i/sampleRate)*(0.16 - tone*0.015);
			}
		} else {
			int tone = name == "110" ? 0 : 2;
			sample = std::sin(2*M_PI*frequencies[tone]*i/sampleRate)*(0.16 - tone*0.015);
		}
		result[i] = quantized(sample);
	}
	return result;
}

int main(int argc, char **argv) {
	if (argc != 2) return 2;
	const int sampleRate = 8000;
	const int inputFrames = sampleRate*2;
	const int inputStep = 120;
	const int outputStep = 240;
	const int traceStep = 64;
	auto input = control(argv[1], inputFrames, sampleRate);

	Stretch stretch(0);
	stretch.presetDefault(1, sampleRate);
	int seekLength = stretch.outputSeekLength(0.5f);
	stretch.outputSeek(Channels{input.data()}, seekLength);

	std::vector<std::vector<Complex>> inputHistory;
	std::vector<float> output(outputStep);
	for (int step = 0; step <= traceStep; ++step) {
		std::vector<Complex> previousOutput(stretch.bands);
		std::vector<float> previousEnergy(stretch.bands);
		for (int b = 0; b < stretch.bands; ++b) {
			previousOutput[b] = stretch.bandsForChannel(0)[b].output;
			previousEnergy[b] = stretch.predictionsForChannel(0)[b].energy;
		}

		int inputOffset = seekLength + step*inputStep;
		stretch.process(Channels{input.data() + inputOffset}, inputStep, Channels{output.data()}, outputStep);
		auto *bands = stretch.bandsForChannel(0);
		auto *predictions = stretch.predictionsForChannel(0);
		std::vector<Complex> current(stretch.bands);
		for (int b = 0; b < stretch.bands; ++b) current[b] = predictions[b].input;
		inputHistory.push_back(current);

		if (step == traceStep) {
			std::cout << std::setprecision(9);
			std::cout << "META\t" << stretch.blockSamples() << "\t" << stretch.intervalSamples()
				<< "\t" << stretch.stft.fftSamples() << "\t" << stretch.bands << "\t"
				<< Stretch::STFT::modified << "\t" << stretch.bandToFreq(0)*sampleRate << "\t"
				<< (stretch.bandToFreq(1) - stretch.bandToFreq(0))*sampleRate << "\t" << inputOffset << "\n";
			auto &auxiliary = inputHistory[inputHistory.size() - 3];
			for (int b = 0; b < stretch.bands; ++b) {
				Complex horizontal = previousOutput[b]*current[b]*std::conj(auxiliary[b]);
				horizontal /= std::max(previousEnergy[b], predictions[b].energy) + 1e-15f;
				std::cout << "BIN\t" << b << "\t" << stretch.bandToFreq(b)*sampleRate
					<< "\t" << current[b].real() << "\t" << current[b].imag()
					<< "\t" << horizontal.real() << "\t" << horizontal.imag()
					<< "\t" << bands[b].output.real() << "\t" << bands[b].output.imag() << "\n";
			}
		}
	}
}
