# 液态玻璃

需要 Windows AE **26.5 或以上**，以及同一构建的 `DynamicFx.aex` 和
`DynamicFxCoverageReader.aex`。这是开发分支的新功能，已发布的 0.1.1
安装包不包含它。[覆盖图安装说明](host-coverage.md)。

## 使用

1. 把 [应用脚本](../examples/apply-liquid-glass.jsx) 和
   [shader](../examples/liquid-glass.glsl) 放在同一个文件夹。
2. 在 AE 合成里选中一个或多个未锁定的形状图层。
3. 运行 `apply-liquid-glass.jsx`。脚本添加材质并打开形状图层的调整层开关。
4. 继续编辑原来的矩形、圆形、贝塞尔路径、蒙版和关键帧，玻璃会跟随形状。

也可以给已经打开调整层开关的形状图层应用
[liquid-glass.ffx](../examples/liquid-glass.ffx)。无需选择其他图层、蒙版或路径。
后台初始化完成后即可预览；AE 尚未返回空闲状态时可能短暂显示 E60。
自动生成的内部读取层保持隐藏，不需要手动编辑。

| 参数 | 作用 |
|---|---|
| Amount | 整体强度；0 精确恢复输入 |
| Glass Thickness | 边缘折射宽度，单位为合成像素 |
| Refraction | 背景折射位移，单位为合成像素 |
| IOR | 折射率，默认 1.4 |
| Dispersion | 边缘色散 |
| Frost Radius | 磨砂半径，范围 0–12 像素 |
| Fresnel Range / Strength / Hardness | 边缘反光的范围、强度和硬度 |
| Glare Range / Strength / Hardness / Convergence / Opposite | 定向高光的范围、强度、硬度、聚集程度和背面强度 |
| Glare Angle | 高光方向 |
| Glass Tint / Tint Amount | 玻璃颜色与染色程度 |
| Sampling Margin | 画布外采样余量；默认 80 像素，大幅折射时需相应增加 |
| Linear RGB Input | 线性光工程开启；普通 sRGB 输入保持关闭 |

形状填充的透明度仍由 AE 正常合成；shader 不会再乘一次覆盖图。
挖孔内和完全位于形状外的背景保持不变。每个形状独立绑定，重命名不影响材质。
8/16/32 bpc 均可用；16/32 bpc 适合需要更多颜色层次和高光范围的工程。

## 实现与验证

材质移植自 MIT 开源的 [Liquid Glass Studio](https://github.com/iyinchao/liquid-glass-studio)，
保留折射、色散、Fresnel 反光、定向高光和 LCH 染色公式。
上游的规则几何替换为原始图层的覆盖图，详见[来源与修改记录](../examples/liquid-glass-upstream.md)。

五个 pass 分别完成两步距离计算、两步背景模糊和最终着色。距离计算保留
最多 64 个合成像素的边界范围，并平滑法线；这是栅格覆盖图的距离，不是
对矢量曲线的精确重建。中间场保存浮点位模式，解码后才过滤，8/16/32 bpc
距离结果一致。低分辨率预览仍受实际采样分辨率限制，最终画质以 Full 为准。

AE 26.5x89 实机验证覆盖三个项目位深、Full/Half/Quarter、三个动画时刻（27 帧），
并检查半透明、挖孔、外部像素和重命名贝塞尔 FFX 复用。详细构建和原始
记录见[测试矩阵](TEST_MATRIX.md)。图像导出以正式渲染队列为准：本次宿主
的 32 bpc `saveFrameToPng` 输出偏暗，原生采样和正式输出正常，未通过修改
shader 曝光补偿它。
