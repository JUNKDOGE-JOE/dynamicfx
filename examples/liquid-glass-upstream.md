# Liquid Glass upstream attribution

The optical/reflection/glare path in `liquid-glass.glsl` is adapted from
[iyinchao/liquid-glass-studio](https://github.com/iyinchao/liquid-glass-studio),
commit `f7b28c36305a862f5cffed3ddd51511cf1204f56`,
`src/shaders/fragment-main.glsl` final STEP 9 branch. Copyright (c) 2024 Charles
Yin; [MIT license](licenses/liquid-glass-studio-MIT.txt).

Its LCH conversion helpers credit
[GLSL-Color-Functions](https://github.com/Rachmanin0xFF/GLSL-Color-Functions),
copyright (c) 2022 Adam Lastowka; [MIT license](licenses/glsl-color-functions-MIT.txt).

Changes for DynamicFX:

- Replace analytic SDF shapes with a bounded, separable distance transform of
  the host's positive alpha support. It measures distance to transparent raster
  samples, capped at 64 logical pixels. This is raster geometry, not an exact
  analytic SDF reconstruction. A small spatial filter stabilizes its normals.
- Store distance fields as packed IEEE float bits so 8-bpc intermediates retain
  precision. Packed values are fetched as texels and decoded before smoothing.
- Map WebGL uniforms and textures into the existing five-pass DynamicFX ABI.
  Refraction distance is exposed in logical pixels: upstream `u_refDistance`
  times `sqrt(2)*1000`. Normals use unit length and consistent screen orientation.
- Preserve the upstream Snell-angle displacement, RGB dispersion, Fresnel/glare
  band formulas, directional glare and LCH tint. Clamp domains, negative bases of
  non-integral powers, and excessive offsets to keep extreme settings finite.
  Evaluate the Snell angle difference with its algebraically equivalent sine/
  cosine ratio; the GPU inverse-trig version differed from the CPU reference by
  0.002992 pixels, reduced to 0.00000423 pixels with this form.
- Decode associated RGB for optical color work and restore it afterward; retain
  input alpha. AE applies shape coverage once. Positive partial opacity is not
  thresholded away to define a new output alpha.
- Expose an explicit Linear RGB Input toggle for linear-light AE projects;
  the default follows the upstream sRGB interpretation. Keep Amount=0 exact.

No web framework, DOM capture, sample photograph or video from the upstream
project is included. The original native coverage plug-in is unchanged.
