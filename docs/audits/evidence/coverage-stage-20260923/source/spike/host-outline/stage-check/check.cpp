#include "../../../src/host/coverage_stage.cpp"
#include <cstring>

namespace {
struct Mock {
    int test;
    int acquire = 0, release = 0, open = 0, dispose = 0, get = 0, free_value = 0, set = 0;
    A_long layer = 123;
    AEGP_LayerParamStage stage = -2;
    AEGP_StreamSuite7 suite{};
    bool arguments = true;
};
thread_local Mock* mock;
constexpr A_Err fault = 71;
auto stream_ref = reinterpret_cast<AEGP_StreamRefH>(static_cast<std::uintptr_t>(16));
auto effect_ref = reinterpret_cast<AEGP_EffectRefH>(static_cast<std::uintptr_t>(32));

SPErr acquire(const char* name, std::int32_t version, const void** result) {
    ++mock->acquire;
    mock->arguments &= std::strcmp(name, kAEGPStreamSuite) == 0 && version == kAEGPStreamSuiteVersion7;
    if (mock->test == 5) return fault;
    *result = mock->test == 6 ? nullptr : &mock->suite;
    return 0;
}
SPErr release(const char* name, std::int32_t version) {
    ++mock->release;
    mock->arguments &= std::strcmp(name, kAEGPStreamSuite) == 0 && version == kAEGPStreamSuiteVersion7;
    return mock->test == 17 ? fault : 0;
}
A_Err open(AEGP_PluginID id, AEGP_EffectRefH effect, PF_ParamIndex index, AEGP_StreamRefH* result) {
    ++mock->open;
    mock->arguments &= id == 9 && effect == effect_ref && index == 7;
    if (mock->test == 8) return fault;
    *result = mock->test == 9 ? nullptr : stream_ref;
    return 0;
}
A_Err dispose(AEGP_StreamRefH stream) {
    ++mock->dispose;
    mock->arguments &= stream == stream_ref;
    return mock->test == 16 ? fault : 0;
}
A_Err get(AEGP_PluginID id, AEGP_StreamRefH stream, AEGP_StreamValue2* value, AEGP_LayerParamStage* stage) {
    ++mock->get;
    mock->arguments &= id == 9 && stream == stream_ref;
    if (mock->test == 10 || (mock->test == 14 && mock->get == 2)) return fault;
    value->streamH = stream;
    value->val.layer_id = mock->layer;
    *stage = mock->stage;
    if (mock->test == 12 && mock->get == 2) value->val.layer_id = 789;
    if (mock->test == 13 && mock->get == 2) *stage = 0;
    return 0;
}
A_Err free_value(AEGP_StreamValue2* value) {
    ++mock->free_value;
    mock->arguments &= value->streamH == stream_ref;
    return mock->test == 15 ? fault : 0;
}
A_Err set(AEGP_PluginID id, AEGP_StreamRefH stream, AEGP_StreamValue2* value, AEGP_LayerParamStage stage) {
    ++mock->set;
    mock->arguments &= id == 9 && stream == stream_ref && value->streamH == stream_ref;
    if (mock->test == 11) return fault;
    mock->layer = value->val.layer_id;
    mock->stage = stage;
    return 0;
}
}

extern "C" std::uint32_t check_coverage_stage(std::int32_t test) {
    Mock state{test};
    mock = &state;
    state.suite.AEGP_GetNewEffectStreamByIndex = open;
    state.suite.AEGP_DisposeStream = dispose;
    state.suite.AEGP_GetStreamLayerParamAndStageValue = get;
    state.suite.AEGP_DisposeStreamValue = free_value;
    state.suite.AEGP_SetStreamLayerParamAndStageValue = set;
    if (test == 7) state.suite.AEGP_SetStreamLayerParamAndStageValue = nullptr;
    SPBasicSuite basic{};
    basic.AcquireSuite = acquire;
    basic.ReleaseSuite = release;
    if (test == 19) basic.ReleaseSuite = nullptr;
    std::int32_t layer = -99, stage = -99, changed = -99;
    const bool read_only = test == 0 || test == 21;
    if (test == 21) state.stage = 7;
    auto error = dynamicfx_coverage_binding(&basic, 9, test == 18 ? nullptr : effect_ref, 7,
        read_only ? 0 : 1, test == 1 || test == 4 ? 123 : 456,
        test == 20 ? 1 : (test == 1 || test == 3 ? -2 : -1), &layer, &stage, &changed);
    const bool succeeds = test <= 4 || test == 21;
    const bool no_host = test >= 18 && test <= 20;
    const int acquisitions = no_host ? 0 : 1;
    const int releases = no_host || test == 5 ? 0 : 1;
    const int opens = no_host || (test >= 5 && test <= 7) ? 0 : 1;
    const int disposals = opens && test != 8 && test != 9 ? 1 : 0;
    const int writes = disposals && !read_only && test != 1 && test != 10 ? 1 : 0;
    const int reads = disposals ? (writes && test != 11 && test != 15 ? 2 : 1) : 0;
    const int values = test == 10 ? 0 : reads - (test == 14 ? 1 : 0);
    std::uint32_t failures = 0;
    if ((error == 0) != succeeds) failures |= 1;
    if (state.acquire != acquisitions || state.release != releases) failures |= 2;
    if (state.open != opens || state.dispose != disposals) failures |= 4;
    if (state.get != reads || state.free_value != values) failures |= 8;
    if (state.set != writes) failures |= 16;
    if (!state.arguments) failures |= 32;
    if (succeeds && (layer != state.layer || stage != state.stage || changed != writes)) failures |= 64;
    mock = nullptr;
    return failures;
}
