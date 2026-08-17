# ADR-0014: Windows AE 2023-2026 build, install, and test protocol

- Status: Accepted
- Date: 2026-08-11
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §17, §19, §20
- Related decisions: [ADR-0004](0004-breaking-rewrite-and-host-matrix.md), [ADR-0009](0009-staged-format-adr-acceptance.md)
- Related tests/audits: host matrix in [../TEST_MATRIX.md](../TEST_MATRIX.md); every milestone audit

## Context

ADR-0004 fixes Windows AE 2023-2026 as the initial matrix with independent per-year evidence. What is not yet fixed: the build identity rules, the GPU backend a support claim rests on, and the harness discipline that keeps four years of evidence affordable. Leaving the wgpu backend floating turns adapter differences into unreproducible bug reports; leaving verification manual guarantees the matrix rots into `NOT_RUN`.

## Decision

1. **Build identity.** Target is `x86_64-pc-windows-msvc`, `cargo build --release`, one AEX artifact serving all four AE years. The Rust toolchain is pinned in-repo (`rust-toolchain.toml`, added with the first M1 runtime commit). Every evidence record states commit/working-tree baseline, toolchain, artifact size and SHA-256.
2. **Install boundary.** Installation is per-year into `Support Files\Plug-ins\DynamicFx\DynamicFx.aex`, driven by an installer that requires an explicit `2023|2024|2025|2026` argument, refuses while `AfterFX.exe` or `aerender.exe` runs, and refuses when a shared `MediaCore` copy exists (ADR-0004). The installed artifact's hash and destination are recorded whenever a host result cites it — this is how "stale AEX tested by accident" is ruled out.
3. **GPU backend policy.** The supported and tested wgpu backend on Windows is **DirectX 12**. Adapter selection requests high performance; adapter name, backend, and driver identity are logged and included in host evidence. Other backends (Vulkan/GL) may exist behind an explicit `DYNAMICFX_BACKEND` diagnostic override but never carry a support claim. A machine without a usable DX12 adapter yields the documented `Unavailable` state: diagnostic plus pass-through (architecture §17), never a crash.
4. **Host protocol per AE year.** A support claim for a year means these scenario families each have complete evidence on that year, mapped 1:1 to test-matrix host rows and gated by milestone: load/effect discovery; single `addProperty("DynamicFx")`; Language defaults to GLSL; expression-only first frame; invalid-source diagnostic and pass-through; keyframed parameters (M2+); save/reopen and clone/registry behavior (M3+); two-pass graph (M4+); 16/32-bpc (M5+); temporal (M6+); SmartRender/MFR (M7+); aerender parity for the year's applicable rows. No result generalizes across years (ADR-0004).
5. **Harness discipline.** Host evidence is produced by the scripted harness (JSX scenarios driven through `AfterFX.exe -r`, sentinel-file completion, aerender legs, numeric image comparison), extending the M0 spike driver pattern in `scripts/spike/`. Raw outputs live in gitignored `out/` directories; curated logs/PNGs/AEPs are copied under `docs/audits/evidence/` and referenced from test-matrix result records. Manual UI steps are reserved for scenarios scripting cannot reach and must be written down step-exact when used. `PENDING_LOG` is never `PASS` (existing evidence policy).
6. **Host identity.** Every host record includes the full AE version/build as reported by the host (`app.version`/About), the OS build, and for GPU-relevant rows the adapter/backend/driver from §3.
7. **Host availability.** The current dev machine provides AE 2025 and 2026 only. AE 2023/2024 rows are `BLOCKED` with the named condition "host not installed on dev machine" until provisioned. Milestone exits require the years their roadmap criteria name; the full four-year matrix is a release gate, not an M1 gate.

## Alternatives considered

- One AEX per AE year: rejected; nothing in the SDK usage requires it and it quadruples artifact/identity bookkeeping.
- Vulkan as the primary Windows backend: rejected; DX12 is the platform-native baseline, matches the capability floor the reference competitor ships against, and reduces adapter variance in evidence.
- Manual verification with screenshots: rejected; four years times the acceptance matrix is not sustainable manually, and ADR-0009 already makes the harness a first-class M1 deliverable.
- Blocking all milestones until 2023/2024 hosts exist: rejected; per-year rows stay visible as `BLOCKED` instead of silently narrowing the product claim.

## Consequences

### Benefits

- "Works on AE X" always has a reproducible artifact identity behind it.
- Backend variance stops masquerading as plugin bugs; evidence names the adapter.
- The harness makes per-year regression cost roughly constant as milestones accumulate.

### Costs and risks

- DX12-only narrows support on exotic setups (old GPUs, some VMs); they get a clean diagnostic, not rendering.
- Pinned toolchain and hash bookkeeping add friction to every host run — accepted as the price of trustworthy evidence.
- 2023/2024 stay unverified until hosts are provisioned; the risk that host-specific topology issues surface late is real and stays visible in the matrix.

## Revisit conditions

Evidence that a target AE year or a material user population cannot run DX12 justifies promoting a second backend through a superseding ADR with its own capability floor and test rows. Changes to the install boundary or year matrix already require superseding ADR-0004.

## Verification obligations

- Installer refusal tests: running hosts, missing year argument, MediaCore copy present.
- First M1 host record demonstrates the full identity chain: commit → toolchain → artifact hash → install path → AE build → adapter/backend.
- Harness produces sentinel-complete logs and numeric image comparisons for every scenario row it claims.
- 2023/2024 rows carry the named `BLOCKED` condition until real runs replace them.
