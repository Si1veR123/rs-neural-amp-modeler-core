
#include "dsp.h"
#include "activations.h"
#include "convnet.h"
#include "get_dsp.h"
#include "lstm.h"
#include "util.h"
#include "version.h"
#include "wavenet.h"

#include <filesystem>
#include <string>

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
