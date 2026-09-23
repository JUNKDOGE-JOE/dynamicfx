#include "AEConfig.h"
#include "AE_GeneralPlug.h"
#include <cstdint>

struct StageReport {
    std::int32_t step, layer, before, limit, after, applied;
};

static_assert(sizeof(A_long) == sizeof(std::int32_t), "stage ABI width");
static_assert(sizeof(AEGP_PluginID) == sizeof(std::int32_t), "plugin ID ABI width");
static_assert(kAEGPStreamSuiteVersion7 == 12, "requires SDK 26.5 StreamSuite7");

extern "C" std::int32_t host_stage_probe(SPBasicSuite* basic, std::int32_t id,
    AEGP_EffectRefH effect, std::int32_t index, std::int32_t action, StageReport* out)
{
    if (!basic || !effect || !out || action < 0 || action > 7) return A_Err_PARAMETER;
    *out = {};
    const AEGP_StreamSuite7* stream = nullptr;
    out->step = 1;
    A_Err err = basic->AcquireSuite(kAEGPStreamSuite, kAEGPStreamSuiteVersion7,
        reinterpret_cast<const void**>(&stream));
    if (err) return err;
    struct Release {
        SPBasicSuite* basic;
        ~Release() { basic->ReleaseSuite(kAEGPStreamSuite, kAEGPStreamSuiteVersion7); }
    } release{basic};
    if (!stream) return A_Err_PARAMETER;
    AEGP_StreamRefH ref = nullptr;
    out->step = 2;
    err = stream->AEGP_GetNewEffectStreamByIndex(id, effect, index, &ref);
    if (err) return err;
    struct DisposeStream {
        const AEGP_StreamSuite7* suite; AEGP_StreamRefH ref;
        ~DisposeStream() { suite->AEGP_DisposeStream(ref); }
    } dispose_stream{stream, ref};
    AEGP_StreamValue2 value{};
    AEGP_LayerParamStage stage = 0, limit = 0;
    out->step = 3;
    err = stream->AEGP_GetStreamLayerParamAndStageValue(id, ref, &value, &stage);
    if (err) return err;
    struct DisposeValue {
        const AEGP_StreamSuite7* suite; AEGP_StreamValue2* value;
        ~DisposeValue() { suite->AEGP_DisposeStreamValue(value); }
    } dispose_value{stream, &value};
    out->layer = value.val.layer_id;
    out->before = stage;
    out->step = 4;
    err = stream->AEGP_GetStreamInputStageCycleSafeLimit(id, ref, &limit);
    if (err) return err;
    out->limit = limit;
    if (action) {
        const AEGP_LayerParamStage requested = action == 6 ? AEGP_LayerParamStage_ALL_EFFECTS :
            (action == 5 || action == 7) ? 1 : (action == 1 || action == 3)
            ? AEGP_LayerParamStage_SOURCE : AEGP_LayerParamStage_ONLY_MASKS;
        out->step = 5;
        if (requested == AEGP_LayerParamStage_ALL_EFFECTS && limit != AEGP_LayerParamStage_ALL_EFFECTS) return A_Err_PARAMETER;
        if (requested > 0 && !(limit == AEGP_LayerParamStage_ALL_EFFECTS || limit >= requested)) return A_Err_PARAMETER;
        if (requested == AEGP_LayerParamStage_ONLY_MASKS &&
            !(limit == AEGP_LayerParamStage_ONLY_MASKS ||
              limit == AEGP_LayerParamStage_ALL_EFFECTS || limit > 0)) return A_Err_PARAMETER;
        out->step = 6;
        err = action <= 2 || action >= 6
            ? stream->AEGP_SetStreamLayerParamStageValue(id, ref, requested)
            : stream->AEGP_SetStreamLayerParamAndStageValue(id, ref, &value, requested);
        if (err) return err;
        out->applied = 1;
    }
    out->step = 7;
    err = stream->AEGP_GetStreamLayerParamStageValue(id, ref, &stage);
    out->after = stage;
    if (!err) out->step = 8;
    return err;
}
