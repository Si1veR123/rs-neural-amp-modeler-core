#include "wrapper.h"

std::unique_ptr<nam::DSP> get_dsp_from_string_path(const char* str)
{
    auto model_path = std::filesystem::path(str);
    return nam::get_dsp(model_path);
}

// Seems to not show up in bindings without, maybe inlined?
double get_dsp_expected_sample_rate(nam::DSP* dsp)
{
    return dsp->GetExpectedSampleRate();
}

void dsp_process(nam::DSP* dsp, float* input, float* output, int num_frames) {
    dsp->process(input, output, num_frames);
}
