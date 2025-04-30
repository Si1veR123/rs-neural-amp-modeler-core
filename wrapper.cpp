#include "wrapper.h"
#include <iostream>

nam::DSP *get_dsp_from_string_path(const char* str)
{
    if (!str) {
        std::cerr << "Received NULL str" << std::endl;
        return nullptr;
    }

    auto model_path = std::filesystem::path(str);
    if (!std::filesystem::exists(model_path)) {
        std::cout << "NAM model doesn't exist: " << model_path << std::endl;
        std::cout.flush();
        return nullptr;
    }

    try {
        auto dsp = nam::get_dsp(model_path);
        return dsp.release();
    } catch (const std::exception& e) {
        std::cout << "Error creating DSP: " << e.what() << std::endl;
        std::cout.flush();
        return nullptr;
    }
}

// Seems to not show up in bindings without, maybe inlined?
double get_dsp_expected_sample_rate(nam::DSP* dsp)
{
    return dsp->GetExpectedSampleRate();
}

void dsp_process(nam::DSP* dsp, float* input, float* output, int num_frames) {
    dsp->process(input, output, num_frames);
}

void destroy_dsp(nam::DSP* dsp) {
    if (dsp) {
        delete dsp;
    }
}
