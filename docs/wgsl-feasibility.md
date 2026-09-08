# WGSL 适配可行性与生产接入结果

**WGSL 生产前端已实现，CPU 与 Apple Silicon / Metal 验证通过；0.1.0
候选已安装，真实 Undo 验收失败，修复前不可发布。** 当前代码已将 WGSL 加入
Language 菜单，GLSL 保持默认值。这个结论来自生产前端的直接反射与
渲染，不再仅依赖早期 spike 的临时接口适配器。

用户后续授权正式接入与 0.1.0 发布，[ADR-0044](adr/0044-wgsl-and-010-release.md)
已接受。冻结编译源码为 `73d8b516163b36f539a2b30a8e163d07d576b9c5`；
ARM 候选可执行文件 SHA-256 为
`91c8afdcdc6186ca2efa24ca1c06cbcbb619a659b62e77fbe69fe988f9323a22`。
身份与签名见[构建记录](audits/evidence/wgsl-010-20260908/build/build-record.json)。
候选已通过临时目录暂存和管理员认证安装到 AE 26.3x87。217 项渲染、
资源与关键帧断言通过，但一次真实 Language Undo 未恢复语言，撤销记录
被后台名称/值写入污染。该候选判定失败，修复和重新验收正在进行；
发布与真实 iOS 27 Siri 实现均未开始。
[审计 08](audits/08-wgsl-010.md) 与
[TR-WGSL-002](TEST_MATRIX.md#tr-wgsl-002--production-wgsl-and-host-integration-010)
记录生产验收边界。

Undo 修复已冻结于 `f4ba578`，默认版和 editor 版各 213 项测试通过。
新候选 `661e89af…` 的[构建记录](audits/evidence/wgsl-010-20260908/build-661e/README.md)
已归档；管理员安装仍在等待系统认证，尚无该字节的真实宿主 PASS。

## 当前生产验证

环境为 macOS 26.5.2 / build 25F84、arm64、Apple M5 / Metal；Rust 1.97.1，
Naga/wgpu 29.0.4。生产路径是 **WGSL → Naga IR → SPIR-V → wgpu → Metal**，
与 GLSL 共用图执行、上传和渲染器。新证据位于
[wgsl-010-20260908](audits/evidence/wgsl-010-20260908/README.md)。

| 检查 | 已观察结果 | 原始证据 |
|---|---|---|
| 完整 CPU 测试，默认 / editor | 各 205 passed，0 failed | [默认日志](audits/evidence/wgsl-010-20260908/build/tests-default.log)、[editor 日志](audits/evidence/wgsl-010-20260908/build/tests-editor.log) |
| 生产 GLSL/WGSL GPU 对照 | 18 组逐字节一致、有限值、最大误差 0 | [result.json](audits/evidence/wgsl-010-20260908/gpu-production/result.json) |
| 两个正式 WGSL 示例 | 40 次渲染、96 项检查通过 | [summary.json](audits/evidence/wgsl-010-20260908/examples/summary.json) |
| 原生 uniform 布局边界 | 24B、96B 显式填充、64KiB 尾部字段读取均为最大误差 0 | [独立 GPU 结果](audits/evidence/wgsl-010-20260908/independent-review/summary.json) |
| 0.1.0 候选构建与打包 | arm64、版本 0.1.0、ad-hoc 签名已记录 | [候选元数据](audits/evidence/wgsl-010-20260908/build/package-final-candidate.json) |

18 组对照覆盖单/双 pass、8/16/32 GPU 工作格式和 Full/Half/Quarter。
逻辑尺寸为 321×239，实际目标为 321×239、161×120、81×60；双 pass
真实读取中间纹理和原始输入。命令为
`python3 scripts/wgsl/compare_gpu.py --out scripts/out/010/wgsl-gpu-final`。
它复用历史算法 fixture，但经 `frontend_for(LanguageId::WGSL)` 调用
生产前端，**没有调用 spike 的 GLSL schema 适配器**。

两个[正式示例](../examples/README.md)用连续解析场、输入采样、参数与
`fwidth` 覆盖展示作者接口，另测倒序时间请求和重复帧。
[放大对照图](audits/evidence/wgsl-010-20260908/examples/contact-sheet.png)
来自 `scripts/quality/run_wgsl_examples.py` 的真实输出。缩小图与 Full
box 平均值的差异只是描述指标，不是完整抗锯齿证明。8-bpc 双 pass
中间值裁剪/量化可产生约 0.0284 的最大差异；柔和光场优先使用 16/32-bpc。

U15/16-bpc headless 运行检查的是 `Rgba32Float` 工作缓冲区，**不等于
AE 16-bpc 世界或边界转换验收**。同一设备上的等价算法逐位相同也不
保证其他 GPU、任意算法或色彩管理设置逐位相同。各运行保留自己的
源码与 runner 身份；最终成对 GPU 运行的十项共享源码/依赖哈希与
冻结构建记录一致。[归档清单](audits/evidence/wgsl-010-20260908/manifest.json)
映射 271 个文件（含后续追加的候选包检查），存储及解压后的原始哈希均已核对。

## 生产接口已经怎样接入

| 边界 | 当前实现 |
|---|---|
| 语言身份 | 永久 ID 2，菜单 `[GLSL, WGSL]`；ID 1 和 GLSL 默认值不变 |
| 前端 | [wgsl.rs](../src/frontend/wgsl.rs) 直接解析、验证原始 Naga 模块 |
| 参数反射 | [shared.rs](../src/frontend/shared.rs) 与 GLSL 共用注释、类型、偏移和块跨度逻辑，无临时 GLSL schema |
| 图与资源 | 保留 `@dynamicfx 1`、清单顺序、固定 binding、Layer/Gradient/Path 与 temporal 限制 |
| 持久化 | 保存语言 ID、精确完整源码、绑定计划；schema 未变；CPU 已验证恢复及语言切换的槽位继承 |
| 缓存 | token 指纹包含 LanguageId；artifact/pipeline 在当前进程重建，无新增持久 shader cache |
| 诊断 | E21 为 WGSL 解析/语言验证；E18/E19/E20 保持 ABI/参数/产物含义 |

编译器版本隔离来自进程与 artifact 生命周期。没有把架构图中的全部
ModuleHash/ArtifactHash 域或 `frontendVersion()` 声称为已实现，也没有
借语言接入修改持久化编码。完整作者语法见
[WGSL 指南](../skills/dynamicfx-shaders/wgsl.md)。

每个模块恰有一个 `@fragment fn main`，输出 location 0 `vec4<f32>`。
可选 UV 是 location 0 `vec2<f32>`，使用默认 perspective/center 插值；
允许 `@builtin(position) vec4<f32>`。生成器可省略输入纹理、sampler
或 UV；声明的资源即使未使用也校验。

group 0 / binding 2 的 uniform 结构以 `u_resolution: vec2<f32>`、
`u_time: f32`、`u_frame: f32` 开头，偏移 0/8/12。用户成员为 f32、i32、
vec2f、vec3f、vec4f，复选框使用 `i32 + hint:bool`。原生 `@align`/`@size`
偏移与跨度照实上传，块在 GPU 分配前限制为 65,536 字节。独立 GPU
探针确实读取了偏移 65,532 的最后一个 scalar，不是只检查“能编译”。

binding 0 是非数组、非 multisampled 的 `texture_2d<f32>`，binding 1
是普通 sampler，额外输入按清单顺序占用 3/4/5。错误维度、类型、
地址空间、组、重复/无供给 binding、storage、override、额外 stage/entry、
MRT 和其他 fragment builtin 被拒绝。f16/subgroup 等未经设备协商的
可选能力由基线 Naga capabilities 拒绝。局部数学类型不受用户 uniform
类型表限制，但仍须通过语言与能力验证。

## Envelope、源码与参数兼容

`// @param`、ParamId ASCII 规则和 `alias:` 槽位继承保持不变。相同兼容
成员跨 pass 共享参数，注释在整份源码中唯一。Layer/Gradient/Path 是
图输入，不是 uniform 成员。CPU 已验证定义映射、canvas 和 temporal
限制；真实 AE 资源和关键帧行为仍待当前候选验收。

WGSL 行首属性与 envelope 指令共享 `@`，pass 内沿用既有转义：

```text
@pass draw
@@group(0) @binding(0) var u_input: texture_2d<f32>;
@@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f { ... }
@endpass
```

只双写每行第一个 `@`；行内属性和 `// @param` 不变。裸 WGSL 不双写。
未转义的行首属性报 E6，不能静默放宽 grammar。生产测试覆盖 unescape、
清单/模块顺序与 CRLF；Naga 错误包含 pass-local 行号和 UTF-8 字节列号，
宿主另报原始 body 起始行。它**没有声称删除 `@@` 后的列号就是完整
expression 的列号**。快照保存精确提交文本，不是解封装后的单个 pass。

## 保留的失败与限制

首次生产对照因 GLSL fixture 漏带 `@param` 默认值而失败：两种算法
收到不同参数。修正仪器，使两份源码带有相同元数据后，同一生产
runner 得到全部零误差，没有为此修改 runtime。
[失败结果](audits/evidence/wgsl-010-20260908/gpu-first-instrument-failure/result.json)
和原始输出保留。示例[日志](audits/evidence/wgsl-010-20260908/examples/renders.log)
保留最初 sandbox 无法枚举 Metal 的失败，以及获得 GPU 访问权限后的成功。

共享注释规则仍要求 vec3 颜色使用三个数值分量；HEX 展开为四分量，应
搭配 vec4，不能把旧限制误记为 WGSL 修复。WGSL 也不是画质开关：同算法
GLSL/WGSL 输出相同，细腻程度仍由采样、频率、覆盖、精度和 alpha 策略
决定。网上 WGSL 应用的自有 WebGPU/compute 资源需要移植到 fragment ABI。

## 历史 spike 的证据范围

早期研究基于 `d4477ab` 加当时工作树，使用
[spike/wgsl](../spike/wgsl/Cargo.toml)。它从 WGSL IR 生成临时 GLSL 接口
schema 复用旧 reflector，再把原始 WGSL IR 交给生产渲染器。**那是
可行性仪器，不是当前候选前端。** 当时“Language 菜单只有 GLSL”的
结论仅适用于该历史阶段。

- [tests.log](../spike/wgsl/evidence/tests.log)：89 passed，其中 8 项 WGSL
  专项、81 项纳入的生产模块测试，不是完整插件测试总量。
- [gpu.log](../spike/wgsl/evidence/gpu.log)：256×128 单/双 pass × 三种
  工作格式共六组逐字节一致，最大误差 0；最高 float 分量 1.5989501。
- [field.metal](../spike/wgsl/evidence/field.metal) 与
  [mix.metal](../spike/wgsl/evidence/mix.metal)：MSL 2.0 生成成功，使用真实
  binding 并关闭 fake missing binding；不是独立 Xcode 编译器验收。
- [初次 GPU 失败](../spike/wgsl/evidence/gpu-initial.log)、
  [vec3 HEX 默认值失败](../spike/wgsl/evidence/initial-hex-default-failure.log)
  和依赖/源码 [run.json](../spike/wgsl/evidence/run.json) 保持原样。

历史结果支持可行性判断；生产结论由前述新证据独立支撑。原先的工程量
估算与“先评审 surface ADR”建议已由 ADR-0044 和实际实现取代。

## 仍待完成的宿主与发布验收

AE 2026 的菜单/default、WGSL 发布、E21 失败恢复、语言切换、关键帧、
真实 Undo/Redo、save/reopen、Layer/Gradient/Path、temporal 随机时间请求、
Full/Half/Quarter 和独立 aerender，须在本次候选上完成，当前不标记
PASS。此前 0.0.6 GLSL-only 原生构建的 AE 结果不能替代它。
Windows 0.1.0 构建与 Windows/DX12、其他 AE 年份、Intel/Rosetta 仍 NOT_RUN。

候选为 ARM-only、ad-hoc 签名，未 Developer ID 签名或 notarize。发布
还需精确安装身份、源码/版本/tag、归档与重新下载哈希/签名检查。
Windows 资产按授权稍后独立补齐。
[TR-REL-010](TEST_MATRIX.md#tr-rel-010--010-publication) 仍记录发布待验。

**下一步：**修复用户操作的状态发布与后台可撤销写入，构建新候选并
重新验收 AE 2026；随后才冻结验收字节用于 0.1.0 发布。真实 iOS 27 Siri
编写在发布后开始。
