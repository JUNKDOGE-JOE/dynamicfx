#include "AEConfig.h"
#include "AE_GeneralPlug.h"
#include <cstdint>

static_assert(sizeof(A_long) == sizeof(std::int32_t), "layer ID ABI width");
static_assert(sizeof(AEGP_PluginID) == sizeof(std::int32_t), "plugin ID ABI width");
static_assert(kAEGPStreamSuiteVersion7 == 12, "requires SDK 26.5 StreamSuite7");

namespace {
struct Binding {
    std::int32_t layer;
    std::int32_t stage;
};

struct Resources {
    SPBasicSuite* basic;
    const AEGP_StreamSuite7* suite = nullptr;
    AEGP_StreamRefH stream = nullptr;
    AEGP_StreamValue2 value{};
    bool acquired = false;
    bool has_value = false;

    A_Err close(A_Err error) {
        auto retain_error = [&error](A_Err next) { if (!error) error = next; };
        if (has_value) {
            has_value = false;
            retain_error(suite->AEGP_DisposeStreamValue(&value));
        }
        if (stream) {
            auto ref = stream;
            stream = nullptr;
            retain_error(suite->AEGP_DisposeStream(ref));
        }
        if (acquired) {
            acquired = false;
            retain_error(basic->ReleaseSuite(kAEGPStreamSuite, kAEGPStreamSuiteVersion7));
        }
        return error;
    }

    ~Resources() { close(A_Err_NONE); }

    A_Err open(AEGP_PluginID id, AEGP_EffectRefH effect, A_long index) {
        A_Err error = basic->AcquireSuite(kAEGPStreamSuite, kAEGPStreamSuiteVersion7,
            reinterpret_cast<const void**>(&suite));
        if (error) return error;
        acquired = true;
        if (!suite || !suite->AEGP_GetNewEffectStreamByIndex ||
            !suite->AEGP_GetStreamLayerParamAndStageValue ||
            !suite->AEGP_SetStreamLayerParamAndStageValue ||
            !suite->AEGP_DisposeStream || !suite->AEGP_DisposeStreamValue) return A_Err_PARAMETER;
        return suite->AEGP_GetNewEffectStreamByIndex(id, effect, index, &stream);
    }

    A_Err read(AEGP_PluginID id, Binding& binding) {
        if (has_value) {
            has_value = false;
            A_Err error = suite->AEGP_DisposeStreamValue(&value);
            if (error) return error;
        }
        value = {};
        AEGP_LayerParamStage stage = 0;
        A_Err error = suite->AEGP_GetStreamLayerParamAndStageValue(id, stream, &value, &stage);
        if (error) return error;
        has_value = true;
        binding = {static_cast<std::int32_t>(value.val.layer_id), static_cast<std::int32_t>(stage)};
        return A_Err_NONE;
    }
};
}

// The official header owns the vtable layout; Suite7 inserts entries inside Suite6.
extern "C" std::int32_t dynamicfx_coverage_binding(SPBasicSuite* basic, std::int32_t id,
    AEGP_EffectRefH effect, std::int32_t index, std::int32_t write,
    std::int32_t layer, std::int32_t stage, std::int32_t* actual_layer,
    std::int32_t* actual_stage, std::int32_t* changed)
{
    if (!basic || !basic->AcquireSuite || !basic->ReleaseSuite || !effect || index <= 0 ||
        !actual_layer || !actual_stage || !changed || (write != 0 && write != 1) ||
        (write && !(stage == 0 || stage == -1 || stage == -2))) return A_Err_PARAMETER;
    *actual_layer = 0;
    *actual_stage = 0;
    *changed = 0;
    Resources resources{basic};
    A_Err error = resources.open(id, effect, index);
    if (error) return resources.close(error);
    if (!resources.stream) return resources.close(A_Err_PARAMETER);
    Binding actual{};
    error = resources.read(id, actual);
    if (error) return resources.close(error);
    if (write && (actual.layer != layer || actual.stage != stage)) {
        resources.value.val.layer_id = layer;
        error = resources.suite->AEGP_SetStreamLayerParamAndStageValue(id, resources.stream,
            &resources.value, stage);
        if (error) return resources.close(error);
        *changed = 1;
        error = resources.read(id, actual);
        if (error) return resources.close(error);
        if (actual.layer != layer || actual.stage != stage) return resources.close(A_Err_GENERIC);
    }
    *actual_layer = actual.layer;
    *actual_stage = actual.stage;
    return resources.close(A_Err_NONE);
}
