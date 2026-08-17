# Audit 07: Performance, SmartRender, and MFR

- Milestone: M7 — Performance, SmartRender, and MFR
- Audit state: Complete (entered 2026-08-13, exited 2026-08-14; TR-M7-001…006)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md)

## Outcome

M7 EXITED. Measurement-first end to end: per-render span instrumentation (`DYNAMICFX_PERF=1`) → AE 2025 baseline (TR-M7-001) → optimizations landed only with before/after pairs and green M1-M6 batteries. Cumulative result vs baseline (total p50 per render): every non-temporal scene −36…−53%; temporal `@window 16` −81% (17.6 → 2.7 ms, ~1.6× a plain render); 4K/32-bpc halved (137 → 72 ms); ROI requests deliver only the asked rect (~3.5× on 11×11 downstream requests at 4K) with pixels identical by construction. Two queue items closed by measurement instead of code (aerender re-resolution no longer reproduces; preview invalidation already correct — the WYSIWYG constraint holds, TR-M7-003). MFR stance confirmed against measured concurrency (TR-M7-005). Budgets enforced and all five ROADMAP exit criteria verified on the exit artifact (TR-M7-006).

Optimization item 1 (GPU resource reuse) landed as TR-M7-002: −36…−53% total p50 on every non-temporal scene, −81% on temporal `@window 16` — windowed re-simulation now costs ~1.6× a plain render. The full M1-M6 host regression battery ran green on the changed artifact (88 probes, `fails=0` everywhere), including the M6 temporal-law suite that directly gates the ping-pong-reuse/zero-texture redesign.

## Visible evidence

[TR-M7-001](../TEST_MATRIX.md#tr-m7-001--performance-baseline-benchmark-matrix): 8 scenes × 25 render-queue frames on AE 2025, 275 per-render span lines, median/p95 tables in [evidence/m7-perf/ae2025/summary.md](evidence/m7-perf/ae2025/summary.md). Headline baseline numbers (total p50 per render): 720p/8-bpc 1-pass 3.6 ms; 4K/8-bpc 1-pass 26.7 ms; thermal 6-pass 3.9 ms (720p) / 33.8 ms (4K); temporal `@window 16` 17.6 ms; 4K/32-bpc 137 ms. GPU execution is never the bottleneck (max 20 ms p50 at 4K float); boundary conversion + upload + readback carry 80-90% of every heavy scene.

## Baseline

- M6 exit artifact `C7023854…` verified on AE 2025 (see [06-temporal-feedback.md](06-temporal-feedback.md)).
- Inherited measurement targets: windowed-render cost (n ≤ W plan executions per frame), aerender resolving the definition per frame (19 rebuilds / 19 renders measured), every smart render drawing the full frame regardless of ROI, per-render diagnostic logging, per-frame GPU resource recreation (input/readback/intermediates), the idle-invalidation "state hash nudge" candidate (competitor study), and the M5 log-level policy note.

## Code paths

- `src/render.rs`: `PerfBreakdown` (upload/gpu/readback spans); `FrameCache` + `ensure_frame_cache` (item 1: textures/readback/sampler/temporal pair reused across renders, keyed by token/depth/size/plan shape); `execute_plan` now runs the whole ADR-0025 window internally — one input upload per frame, per-iteration submits, iteration-0 History bound to a never-written zero texture (black start without clears), readback on the final iteration only; 256-aligned rows upload without the repack copy.
- `src/lib.rs`: `perf_log_enabled()` env gate (`DYNAMICFX_PERF=1`, cold-start `OnceLock`); conversion-in/out spans; `Local.frame_cache` + reusable conversion scratch (split-borrowed under the instance lock held for the whole render — the MFR-safety argument for the cache); one machine-parsable `perf:` line per successful render (depth, dims, passes, iters, frame, six spans).
- `scripts/m7/`: `m7bench.jsx` + `m7_lib.jsxinc` (6 comps / 9 instances, 8 scene renders, StateToken readiness polling, SCENE epoch markers), `run_m7.ps1` (cold-start driver; refuses a warm AE), `summarize_perf.py` (signature-first scene bucketing → median/p95 markdown+CSV), `m7q_quit.jsx`.

## Contracts fixed or changed

None yet. SmartRender-GPU, MFR flags beyond `SUPPORTS_THREADED_RENDERING`, caching identities, or ROI scheduling changes that touch durable contracts require ADRs per policy.

## Measurement plan (Phase 1, fixed before any optimization)

**What is timed.** Every render logs one machine-parsable `perf:` line when `DYNAMICFX_PERF=1` (a cold-start env gate, zero cost otherwise), carrying: depth, dimensions, pass count, temporal iterations, and wall-clock spans for the four stages — AE→working conversion, GPU execute (split upload / submit+wait / readback), working→AE write-back — plus the per-render total. Wall-clock is meaningful because the execute path is synchronous end-to-end; per-render lines under MFR measure per-render latency, while scene throughput comes from the harness's RQ wall-clock per scene.

**The benchmark matrix** (`scripts/m7/`), 25 RQ frames per scene on AE 2025 (MFR at host defaults):

| Scene | Passes | Content | Sizes | Depths |
|---|---|---|---|---|
| gradient | 1 | uv ramp (M1-class) | 1280×720, 3840×2160 | 8, 32 |
| thermal | 6 | the user-approved thermal-A benchmark (blur chains + field + composite, 10 params) | 1280×720, 3840×2160 | 8 |
| temporal | 1×W | accumulator `@window 16` (windowed re-simulation cost) | 1280×720 | 8 |
| multi | 4×1 | four gradient instances stacked (instance scaling) | 1280×720 | 8 |

**Evidence format.** `summarize_perf.py` reduces the perf log per scene signature (dims/passes/iters/depth) to count, median, p95 per stage → a markdown table + CSV committed under `evidence/m7-perf/`. Every performance claim cites: baseline commit, artifact hash, host (AE build, OS, GPU/driver), and the raw log. Optimizations land only with a before/after pair of this exact matrix plus the M1-M6 correctness suites green on the changed artifact.

**Optimization queue** (ordered by expected value, each gated on the baseline numbers): ~~per-frame GPU resource churn~~ (done — TR-M7-002), ~~aerender per-frame snapshot re-resolution~~ (closed by measurement — see findings), ~~per-render log policy~~ (done — zero always-on appends per render, `DYNAMICFX_VERBOSE_LOG` opt-in; bench pair flat-to-better, temporal 3.3→2.7 ms p50, m5/m6 gates green: [summary_logpolicy_after.md](evidence/m7-perf/ae2025/summary_logpolicy_after.md)), ~~preview invalidation after idle compile~~ (closed by measurement — TR-M7-003: the token-mirror stream write already invalidates; WYSIWYG holds in the adversarial construction), ~~ROI-aware scheduling~~ (done — TR-M7-004: uv-preserving final-pass scissor, identical pixels by construction, ~3.5× on small downstream requests at 4K; input-side ROI deliberately NOT taken — full-frame input upload is the correctness floor while shaders may sample anywhere; a future opt-in "local sampling" shader annotation could lift it and would need an ADR), MFR eligibility beyond the current stance.

**Harness fix recorded:** the "save before closing" modal that deadlocked three m7 runs correlates 100% with COLD `AfterFX -r` launches (a late AE startup module requests a project close after our script has dirtied the untitled project; the panel was ruled out by source inspection). `run_m7.ps1` now warm-starts AE first and sends `-r` after, like every other driver; a Win32 watchdog (session tooling) logs AE-owned dialogs and cancels only prompt-sized `#32770`s, sparing the identically-classed 958×645 splash.

**Standing constraint (user, 2026-08-13): preview must be WYSIWYG.** Interactive preview, render queue, and aerender share one render path and one definition identity (content-keyed), so a stale result can never be served by construction; the M3/M6 suites' interactive-vs-aerender equality probes are the permanent gate. Optimizations may change cost, never pixels. Known cost accepted with item 1: cached GPU resources persist per instance while it exists (a 4K float instance holds ~0.5 GB VRAM across its texture set); an eviction policy joins the queue if real projects surface pressure.

## Commands and exact host steps

Phase 1 baseline: `pwsh scripts/m7/run_m7.ps1 -Year 2025` (cold AE, `DYNAMICFX_PERF=1`), then `-QuitAE`, then `-Summarize`.

## Observed evidence

TR-M7-001 (baseline) recorded with full host identity, artifact hash, commands, and raw logs; failed runs 1 and 3 preserved alongside. See the result record in [TEST_MATRIX.md](../TEST_MATRIX.md#tr-m7-001--performance-baseline-benchmark-matrix).

## Findings and failures

- **Definition publication latency (measured, runs 1+3):** with 9 scripted instances created back-to-back, the idle bridge took ~25 s wall before the earliest render-queue clones could resolve a definition. A 20 s blind wait produced exactly 50 `token=0 compiled=false` passthrough renders (the first two scenes, black output, no diagnostic anywhere in the UI); a 60 s blind wait failed the same way when a host modal ("save before closing", source outside our scripts — a third-party panel is resident) blocked the idle window. Evidence: [dynamicfx_plugin_run1_20s.log](evidence/m7-perf/ae2025/dynamicfx_plugin_run1_20s.log). Fix shipped in the fixture: poll each instance's StateToken stream (property 5; non-zero ⇔ the exact value render clones resolve) and start rendering only at `READY 9/9`. Consequences kept open for M7: idle-bridge throughput joins the optimization queue, and "silent black passthrough of a not-yet-published instance in a batch render" is a product-behavior risk worth a deliberate decision (fail loudly or wait, rather than emit black).
- **Where the time actually goes (baseline):** GPU execution is never the bottleneck on the target machine (RTX 5080): 1-pass 4K float GPU p50 is 20 ms while the render total is 137 ms. The dominating spans everywhere are AE↔working conversion and upload/readback (80-90% of heavy scenes). Temporal `@window 16` costs 17.6 ms vs 3.6 ms single-pass at the same size — almost entirely repeated per-iteration upload/prep (11.5 ms of it), confirming per-render GPU resource churn as the top queue item.
- **Host overhead floor:** RQ wall-clock at 720p/8-bpc is ~48 ms/frame while the plugin spends 3.6 ms — AE-side checkout/composite/PSD-write bounds small-frame throughput; per-render optimization should be judged on the span table, not scene fps.
- **Always-on log I/O:** every render currently appends 3 lines (2 `smart cmd` + 1 `render enter`) regardless of `DYNAMICFX_PERF` — three serialized file opens per render under MFR. Scheduled as the log-policy queue item.
- **Harness lessons recorded:** back-to-back scenes overlap under timestamp slack, so `summarize_perf.py` buckets signature-first (scene tag ⇒ expected depth/dims/passes) with iters/frame semantics for the temporal/multi boundary; the driver refuses a warm AE because the perf gate reads the environment once at cold start.
- **aerender re-resolution no longer reproduces (item 2 closed by measurement):** on the TR-M7-002 artifact, a fresh aerender of the M6 project (25 frames, windowed temporal) logs exactly 1 `definition rebuilt from snapshot` and 1 `pipelines built` for 22 renders — the process registry and persistent clone Locals serve every subsequent frame. The M6-era 19-rebuilds/19-renders figure predates the M6 snapshot-source fix (the flattened sequence carried only `passes[0]` text, destabilizing clone identity). No code change; queue item closed.

## Known limitations

Inherited set listed under Baseline.

## Residual risks

- Optimizations that change render flow must re-run the M1-M6 numeric suites on the host, not just unit tests.

## Decision changes

None.

## Next exact action

M7 is complete; the milestone sequence (M0-M7) is exhausted. Next per the product target: expand the Windows host matrix beyond AE 2025 — install the exit artifact on AE 2026 (present on this machine) and run the M1-M7 driver batteries with `-Year 2026`, recording per-year rows (no result may be copied across years). Recorded follow-ups that remain independent of that: color-annotation defaults (color pool params default white), historical-input temporal windows (ADR-0025 §4 v1), tight-buffer uv semantics note, frame-cache eviction if real projects surface VRAM pressure, and the publication-latency product decision (silent black passthrough vs fail-loudly/wait).

## Reproduction

Follow the `CLAUDE.md` reading order; confirm no TR-M7 rows exist yet.
