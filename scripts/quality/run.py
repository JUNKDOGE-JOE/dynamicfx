#!/usr/bin/env python3
"""Numerical headless GPU evidence; requires numpy + Pillow. Never operates AE.

Build the executable first (README.md), then invoke with --binary. A baseline
binary can be supplied to repeat sampler probes through the pre-fix renderer.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
from datetime import datetime, timezone

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
HEADER = """#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {
    vec2 u_resolution; float u_time; float u_frame;
};
"""


def envelope(body, extra=""):
    return "@dynamicfx 1\n@graph\npass main: input" + extra + " -> output\n@end\n@pass main\n" + HEADER + body + "\n@endpass\n"


def downsample(a, factor):
    h, w, c = a.shape
    return a.reshape(h//factor, factor, w//factor, factor, c).mean(axis=(1, 3))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, default=ROOT / "scripts/out/quality/current")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    binary = args.binary.resolve()
    records, metrics = [], {}
    shader = (ROOT / "examples/siri-glow.glsl").read_text()
    noise = shader[shader.index("uint hashBits"):shader.index("float roundRect")]
    noise_raw = re.sub(r"float keep = [^;]+;", "float keep = 1.0;", noise)
    fixtures = {
        "bilinear2x2": envelope("void main(){outColor=texture(sampler2D(u_in,u_s),vec2(0.5));}"),
        "quarter_texel": envelope("void main(){outColor=texture(sampler2D(u_in,u_s),v_uv+vec2(0.25/u_resolution.x,0));}"),
        "lut": envelope("// @param quality_lut label:\"Quality LUT\" hint:gradient\nlayout(set=0,binding=3) uniform texture2D u_lut;\nvoid main(){outColor=texture(sampler2D(u_lut,u_s),v_uv);}", ", quality_lut"),
        "edge_hard": envelope("void main(){vec2 p=v_uv*u_resolution; float d=length(p-vec2(157.37,91.83))-61.28; float a=step(d,0.0); outColor=vec4(a,a,a,1);}"),
        "edge_aa": envelope("void main(){vec2 p=v_uv*u_resolution; float d=length(p-vec2(157.37,91.83))-61.28; float a=clamp(0.5-d/max(fwidth(d),0.00001),0,1); outColor=vec4(a,a,a,1);}"),
        "noise_raw": envelope(noise_raw+"\nvoid main(){vec2 p=v_uv*vec2(51.3,37.7); float n=0.5+filteredNoise(p,dFdx(p),dFdy(p)); outColor=vec4(n,n,n,1);}"),
        "noise_filtered": envelope(noise+"\nvoid main(){vec2 p=v_uv*vec2(51.3,37.7); float n=0.5+filteredNoise(p,dFdx(p),dFdy(p)); outColor=vec4(n,n,n,1);}"),
        "cells_raw": envelope(noise+"\nvoid main(){vec2 p=v_uv*vec2(16,12); float n=lattice(ivec2(floor(p))); outColor=vec4(n,n,n,1);}"),
        "cells_smooth": envelope(noise+"\nvoid main(){vec2 p=v_uv*vec2(16,12); float n=smoothNoise(p); outColor=vec4(n,n,n,1);}"),
        "precision": "@dynamicfx 1\n@graph\npass seed: input -> t\npass expand: t -> output\n@end\n@pass seed\n"+HEADER+"void main(){float v=v_uv.x*0.01; outColor=vec4(v,v,v,1);}\n@endpass\n@pass expand\n"+HEADER+"void main(){outColor=vec4(texture(sampler2D(u_in,u_s),v_uv).rgb*100,1);}\n@endpass\n",
    }
    for name, text in fixtures.items():
        (out / f"{name}.glsl").write_text(text)

    # The shipped thermal example stores T + fract(T*8) in RG. Interpolating
    # those encoded channels before decoding crosses the fractional wrap.
    packed_seed = "void main(){float t=mix(0.123046875,0.126953125,mod(floor(gl_FragCoord.x),2));outColor=vec4(t,fract(t*8),1,1);}"
    thermal=(ROOT/"examples/apple-thermal.glsl").read_text()
    helper_start=thermal.index("vec2 readTemperature(")
    helper=thermal[helper_start:thermal.index("void main() {",helper_start)]
    for name,body in {
        "packed_bad":"void main(){vec4 c=texture(sampler2D(u_in,u_s),v_uv+vec2(0.5/u_resolution.x,0));float t=(floor(c.r*8-c.g+0.5)+c.g)/8;outColor=vec4(t,t,t,1);}",
        "packed_decoded":helper+"void main(){float t=sampleTemperature(v_uv+vec2(0.5/u_resolution.x,0)).x;outColor=vec4(t,t,t,1);}",
    }.items():
        text="@dynamicfx 1\n@graph\npass seed: input -> t\npass finish: t -> output\n@end\n@pass seed\n"+HEADER+packed_seed+"\n@endpass\n@pass finish\n"+HEADER+body+"\n@endpass\n"
        (out/f"{name}.glsl").write_text(text)

    def render(name, size, logical=None, depth=32, pattern="black", time=0, exe=binary, source=None):
        logical = logical or size
        source = source or out / f"{name}.glsl"
        suffix = f"{name}_{size[0]}x{size[1]}_d{depth}_t{time}" + ("_baseline" if exe != binary else "")
        path = out / f"{suffix}.f32"
        cmd = [str(exe), str(source), str(path), *map(str,size), *map(str,logical), str(time), str(depth), pattern]
        env = dict(os.environ)
        # The historical renderer defaults to DX12 and has no metal override;
        # all is diagnostic-only. The returned adapter must actually be Metal.
        if exe != binary:
            env["DYNAMICFX_BACKEND"] = "all"
        result = subprocess.run(cmd, cwd=ROOT, env=env, text=True, capture_output=True)
        (out/f"{suffix}.log").write_text("COMMAND: "+json.dumps(cmd)+"\nSTDOUT:\n"+result.stdout+"\nSTDERR:\n"+result.stderr+f"\nEXIT: {result.returncode}\n")
        if result.returncode:
            raise RuntimeError(result.stderr)
        record = json.loads(result.stdout.strip().splitlines()[-1])
        records.append(record)
        a = np.fromfile(path,dtype="<f4").reshape(size[1],size[0],4)
        assert np.isfinite(a).all()
        Image.fromarray(np.uint8(np.clip(a[:,:,:3],0,1)*255+0.5)).save(out/f"{suffix}.png")
        return a

    # Known linear-filtering oracles: constant-center sampling, quarter-texel
    # offset at native scale, and LUT magnification. No independent
    # texture-minification probe is run by this suite.
    for label, exe in [("current",binary)]+([("baseline",args.baseline.resolve())] if args.baseline else []):
        for depth in [8,16,32]:
            center = render("bilinear2x2", (2,2), depth=depth, pattern="checker",exe=exe)[0,0,0]
            quarter = render("quarter_texel", (128,4), depth=depth, pattern="checker",exe=exe)[0,:-1,0]
            expected = np.where(np.arange(127)%2==0,0.25,0.75)
            lut = render("lut", (1024,4), depth=depth,exe=exe)[0,:,0]
            uv = (np.arange(1024)+0.5)/1024
            expected_lut = np.clip((uv*256-0.5)/255,0,1)
            errors = {"center_value":float(center),"center_abs_error":float(abs(center-0.5)),
                      "quarter_texel_mae":float(np.abs(quarter-expected).mean()),
                      "lut_mae":float(np.abs(lut-expected_lut).mean()),
                      "lut_distinct_values":int(len(np.unique(lut)))}
            metrics[f"sampling_{label}_{depth}"] = errors
            if label == "current":
                tolerance = 1/255 if depth==8 else 2e-5
                assert errors["center_abs_error"] < tolerance
                assert errors["quarter_texel_mae"] < tolerance
                assert errors["lut_mae"] < tolerance

    # A supersampled hard circle is the independent pixel-coverage reference.
    for factor in [1,2,4]:
        size = (320//factor,180//factor)
        ref = downsample(render("edge_hard", (size[0]*8,size[1]*8),logical=(320,180)),8)
        hard = render("edge_hard",size,logical=(320,180))
        aa = render("edge_aa",size,logical=(320,180))
        hard_error = float(np.abs(hard[:,:,:3]-ref[:,:,:3]).mean())
        aa_error = float(np.abs(aa[:,:,:3]-ref[:,:,:3]).mean())
        metrics[f"edge_preview_{factor}"] = {"hard_mae":hard_error,"aa_mae":aa_error,"reduction":1-aa_error/hard_error}
        assert aa_error < hard_error*0.5

    raw = render("noise_raw",(128,96))
    filtered = render("noise_filtered",(128,96))
    reference = downsample(render("noise_raw",(1024,768)),8)
    raw_rmse = float(np.sqrt(np.mean((raw[:,:,:3]-reference[:,:,:3])**2)))
    filtered_rmse = float(np.sqrt(np.mean((filtered[:,:,:3]-reference[:,:,:3])**2)))
    metrics["noise_bandlimit"] = {"raw_rmse":raw_rmse,"filtered_rmse":filtered_rmse,"reduction":1-filtered_rmse/raw_rmse}
    assert filtered_rmse < raw_rmse
    cells = render("cells_raw",(512,384))
    smooth = render("cells_smooth",(512,384))
    def cell_jump(a):
        dx = abs(a[:,32::32,0]-a[:,31:-1:32,0]).ravel()
        dy = abs(a[32::32,:,0]-a[31:-1:32,:,0]).ravel()
        return float(np.mean(np.concatenate([dx,dy])))
    metrics["cell_boundaries"] = {"raw_mean_jump":cell_jump(cells),"quintic_mean_jump":cell_jump(smooth)}
    assert cell_jump(smooth)<cell_jump(cells)*0.02

    for depth in [8,16,32]:
        a=render("precision",(1024,8),depth=depth)[0,:,0]
        ref=(np.arange(1024)+0.5)/1024
        metrics[f"precision_{depth}"]={"mae":float(abs(a-ref).mean()),"distinct_values":int(len(np.unique(a)))}
    assert metrics["precision_16"]["mae"]<2e-6
    assert metrics["precision_32"]["mae"]<2e-6

    for name in ["packed_bad","packed_decoded"]:
        a=render(name,(128,4))[0,:-1,0]
        metrics[name]={"mae_from_0_125":float(abs(a-0.125).mean())}
    assert metrics["packed_decoded"]["mae_from_0_125"]<1e-6
    assert metrics["packed_bad"]["mae_from_0_125"]>0.05

    # Final complex example: finite output, depth/preview coverage, real times.
    siri=[]
    for t in [0,2,4]:
        a=render("siri",(1280,720),time=t,source=ROOT/"examples/siri-glow.glsl")
        siri.append(a)
    for scale in [2,4]:
        a=render("siri",(1280//scale,720//scale),logical=(1280,720),time=2,source=ROOT/"examples/siri-glow.glsl")
        ref=downsample(siri[1],scale)
        metrics[f"siri_preview_{scale}"]={"mae_vs_full_box":float(abs(a[:,:,:3]-ref[:,:,:3]).mean()),"p99_abs_error":float(np.quantile(abs(a[:,:,:3]-ref[:,:,:3]),0.99))}
    for depth in [8,16]:
        render("siri",(1280,720),time=2,depth=depth,source=ROOT/"examples/siri-glow.glsl")
    render("apple_thermal",(256,256),time=2,source=ROOT/"examples/apple-thermal.glsl",pattern="checker")
    delta=float(abs(siri[0]-siri[2]).mean())
    assert delta>0.001
    metrics["siri_animation_mean_change"] = delta

    # A labeled contact sheet is a navigation aid; numeric values use raw f32.
    tiles=[("Quarter-texel shift: nearest",out/"quarter_texel_128x4_d32_t0_baseline.png"),
           ("Quarter-texel shift: linear",out/"quarter_texel_128x4_d32_t0.png"),
           ("Unfiltered noise",out/"noise_raw_128x96_d32_t0.png"),
           ("Filtered noise",out/"noise_filtered_128x96_d32_t0.png"),
           ("Cell hash",out/"cells_raw_512x384_d32_t0.png"),
           ("Quintic interpolation",out/"cells_smooth_512x384_d32_t0.png")]
    sheet=Image.new("RGB",(1024,720),(20,22,28))
    draw=ImageDraw.Draw(sheet)
    for i,(label,path) in enumerate(tiles):
        if not path.exists(): continue
        x=(i%2)*512; y=(i//2)*240
        draw.text((x+14,y+9),label,fill="white")
        im=Image.open(path).convert("RGB").resize((488,204),Image.Resampling.NEAREST)
        sheet.paste(im,(x+12,y+30))
    sheet.save(out/"comparison.png")
    summary={"status":"PASS","utc":datetime.now(timezone.utc).isoformat(),"platform":platform.platform(),
             "head":subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip(),
             "renderer_sha256":hashlib.sha256((ROOT/"src/render.rs").read_bytes()).hexdigest(),
             "siri_shader_sha256":hashlib.sha256((ROOT/"examples/siri-glow.glsl").read_bytes()).hexdigest(),
             "binary_sha256":hashlib.sha256(binary.read_bytes()).hexdigest(),
             "metrics":metrics,"renders":records,
             "scope":"Headless production frontend/render path on the named adapter. No AE claim. 16-bit output is f32 working data before AE U15 conversion."}
    (out/"summary.json").write_text(json.dumps(summary,indent=2)+"\n")
    print(json.dumps({"status":"PASS","metrics":metrics,"summary":str(out/"summary.json")},indent=2))


if __name__ == "__main__":
    main()
