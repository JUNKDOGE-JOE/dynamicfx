# Windows 0.1.0 artifact backfill

Status on 2026-09-08: **NOT_RUN**. This document is a procedure, not Windows
build or host evidence. The 0.1.0 Windows AEX has not been built or tested on
this Mac. Metal results and older Windows releases do not verify this change
on Windows/DX12. Append a Windows asset to the existing regular **v0.1.0**
release only after the gates below pass; do not move the tag or substitute
current `main`.

The governing decisions are [ADR-0044](adr/0044-wgsl-and-010-release.md) and
[ADR-0014](adr/0014-windows-host-protocol.md). Historical artifact and freeze
rules are recorded in [TR-REL-003](TEST_MATRIX.md#tr-rel-003--003-release-verification)
and [TR-REL-006](TEST_MATRIX.md#tr-rel-006--006-release-verification).
The reviewed build path is Cargo plus [build.rs](../build.rs), followed by
AE-only installation. There is no Windows release workflow in this checkout.
[install.bat](../scripts/install.bat) is an installer, not a release builder.

## 1. Obtain the release source and exact dependency lock

Use a native x64 Windows build machine. The Rust MSVC target needs a Microsoft
C++ toolchain and Windows SDK. Open the installed Visual Studio Developer
PowerShell, then locate tools with `Get-Command`; do not copy a compiler path
from another machine. These requirements and shell discovery are documented
by [Rust](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html)
and [Microsoft](https://learn.microsoft.com/en-us/visualstudio/ide/reference/command-prompt-powershell?view=visualstudio).

1. Run `gh release view v0.1.0 --repo JUNKDOGE-JOE/dynamicfx --json
   tagName,isDraft,isPrerelease,assets,body`; retain the response. Require an
   existing regular, published release. Download its macOS ZIP and
   `SHA256SUMS.txt` into a fresh evidence directory with `gh release download`.
   Match the ZIP's `Get-FileHash -Algorithm SHA256` against its exact checksum
   row before extracting to `$Evidence/released-macos`. Select the actual
   asset name from that response.
2. Clone to a new directory using `git clone -c core.autocrlf=false --branch
   v0.1.0 --single-branch https://github.com/JUNKDOGE-JOE/dynamicfx.git`.
   Record `git rev-parse 'v0.1.0^{commit}'` and `git rev-parse 'v0.1.0^{tree}'`;
   require HEAD to equal that tag. Keep the tracked checkout clean.
3. Copy **the released package's `build/Cargo.lock`** to the checkout root.
   The lock is gitignored, so a tag checkout or GitHub source ZIP alone is
   insufficient. Record its SHA-256; do not run `cargo update` or resolve a
   new lock. Compare every `source_files` hash from the package's
   `build/source-identity.json` with the checkout plus copied lock.
4. Record the tag identity independently of the macOS build's `source_commit`
   baseline. If the macOS build preceded a release commit, document the
   build-input hash comparison; do not describe a baseline commit label as
   proof of identical source. An unexplained input mismatch stops backfill.
   A Windows source fix needs a reviewed release decision; it cannot be
   hidden behind the same tag.

Use PowerShell with `$ErrorActionPreference = 'Stop'`. Check `$LASTEXITCODE`
after **every native command**; PowerShell 5.1 does not convert all native
failures into terminating exceptions. Select actual absolute `$Work` and
`$Evidence` paths and `Set-Location $Work` before the following commands.

## 2. Build, test, then freeze the Windows executable

Read the pinned toolchain from `rust-toolchain.toml` (0.1.0: **1.97.1**).
Use the named version, not whatever `stable` means on the future build day.
Record `rustc -Vv`, Cargo, MSVC and Windows SDK versions, OS build, and all
build environment overrides. In particular, use the locked crate's built-in
AE bindings: `after-effects-sys` switches to locally regenerated bindings
when `AESDK_ROOT` is nonempty. Do not accidentally build against a private SDK.

```powershell
$Repo = 'JUNKDOGE-JOE/dynamicfx'
function Native-OK { if ($LASTEXITCODE -ne 0) { throw "Native command failed: $LASTEXITCODE" } }
foreach ($name in @('AESDK_ROOT','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_DIR','CARGO_BUILD_TARGET')) {
    if ([Environment]::GetEnvironmentVariable($name)) { throw "Review and clear inherited build override: $name" }
}
rustup toolchain install 1.97.1 --profile default --target x86_64-pc-windows-msvc
Native-OK
cargo +1.97.1 -V; Native-OK
rustc +1.97.1 -Vv | Tee-Object "$Evidence/rustc.txt"; Native-OK
if (-not ((Get-Content "$Evidence/rustc.txt") -contains 'host: x86_64-pc-windows-msvc')) {
    throw 'Use the native Windows x64 MSVC toolchain'
}
cargo +1.97.1 fetch --locked --target x86_64-pc-windows-msvc
Native-OK
cargo +1.97.1 test --locked --target x86_64-pc-windows-msvc 2>&1 | Tee-Object "$Evidence/tests-default.txt"
Native-OK
cargo +1.97.1 test --locked --target x86_64-pc-windows-msvc --features editor 2>&1 | Tee-Object "$Evidence/tests-editor.txt"
Native-OK
python scripts/check_governance.py; Native-OK
cargo +1.97.1 build --release --locked --target x86_64-pc-windows-msvc 2>&1 | Tee-Object "$Evidence/build-default.txt"
Native-OK
$Dll = Join-Path $Work 'target/x86_64-pc-windows-msvc/release/dynamicfx.dll'
$Frozen = Join-Path $Evidence 'DynamicFx.aex'
Copy-Item $Dll $Frozen
$FrozenHash = (Get-FileHash $Frozen -Algorithm SHA256).Hash
dumpbin.exe /headers $Frozen | Tee-Object "$Evidence/pe-headers.txt"; Native-OK
dumpbin.exe /exports $Frozen | Tee-Object "$Evidence/pe-exports.txt"; Native-OK
```

Inspect the complete results, not only the last line: no failed/skipped required
test or unexplained warning. The final build uses default features; `editor`
remains off. Verify an AMD64 PE DLL and the `EffectMain` export. Retain the
actual build's generated `pipl.bin` and resource/compiler log; confirm the
byte-file PiPL repair in `build.rs` was used. AE load must not produce a PiPL
or global-out-flags mismatch. The PiPL cache generation is a separate identity
from Cargo's package version; do not rewrite its bytes during packaging.

Record `FrozenHash`, size, tag commit/tree, lock SHA-256, source-file hash map,
exact build command, feature set, tools and environment in
`windows-build.json`. Hash the installed and packaged AEX against this frozen
file. Historical builds were not byte-reproducible: rebuilding from the same
source is not a substitute for preserving the tested bytes. If any executable
change or signing happens later, it creates a new candidate requiring the
host gate again.

## 3. Install the frozen candidate and run real Windows AE

Resolve the actual `AfterFX.exe` from the installed application's location;
record its full path and file version. The corresponding `aerender.exe` and
per-year `Plug-ins` directory must belong to that same AE installation. Save
normal work, close AE and aerender, back up an older AEX **outside all scanned
plug-in directories**, then copy `$Frozen` as
`Support Files/Plug-ins/DynamicFx/DynamicFx.aex`. Record the destination hash
equal to `$FrozenHash` before starting AE. Do not use shared MediaCore or keep
duplicate DynamicFx copies. Use normal elevation only where installation
permissions require it.

The existing `install.bat YEAR` assumes Adobe's default installation path and
reads `target/release/dynamicfx.dll`, with a **debug fallback**. It does not
consume the explicit-target output above or `$Frozen`. Therefore do not invoke
it blindly for this procedure. Either copy the frozen AEX to the verified
destination directly, or first stage those exact bytes at the installer's
release input, assert the hash, and verify the installed hash afterward.
The script also requires both hosts closed and refuses a detected MediaCore
copy; preserve those checks when installing manually.

The current [WGSL acceptance driver](../scripts/wgsl/ae_acceptance.py) has
portable fixture/JSX generation but a macOS-oriented normal transport. Its
`--dry-run --name UNIQUE_TAG` mode writes reviewable JSX without contacting AE;
its ordinary setup/capture commands are not a Windows transport. Emit the
same scenario bodies, add a local JSON result/sentinel wrapper following
[scripts/macos/ae_smoke.py](../scripts/macos/ae_smoke.py)'s `execute`, and run
with the actual installed `AfterFX.exe -r SCRIPT.jsx`. Adobe documents this
[Windows script interface](https://helpx.adobe.com/after-effects/desktop/automate-in-after-effects/automate-animation/scripts.html).
Keep the wrapper as evidence tooling outside the tagged plugin source, and
retain its exact bytes. Enable AE's scripting file-write preference and start
from an empty temporary project. Each invocation needs a unique result path,
a bounded completion wait, `ok: true`, and numerical checking. The emitted
raw dry script only returns data; by itself it does **not** write a completion
record. A launcher exit code, missing sentinel or timeout is not PASS, and a
mutation with unknown outcome must not be blindly retried.

The driver's output folder is named `ae2026`; that label does not establish
which Windows host executed it. Record the actual host version and archive
each year's results separately.

For each installed year being claimed, execute these driver modes **serially**,
with a fresh tag per invocation and an AE-idle boundary where noted:

1. `setup`; idle; `state`; `capture` at Full, Half and Quarter. Inspect both
   languages and invalid-source statuses. GLSL defaults to position 1; WGSL
   is position 2. Expected invalid diagnostics are E17 and E21 respectively.
2. `resources --phase none`; `resources --phase assign`; idle;
   `resources --phase assigned`. This checks real Layer, Gradient and Path
   inputs including none/assigned fallback behavior.
3. `keyframes`; `save`; quit AE normally; relaunch the same year and use AE
   File > Open to open the saved `native-wgsl-010.aep`; idle; `state`,
   `capture`, `keys-read` and assigned resources again. The driver's `reopen`
   mode requires its owned test project already open, so it is not the cold
   start step. Verify saved language, source, values, key streams and samples.
4. Perform real UI language/source change, Undo and Redo on a temporary
   instance; record step-by-step state and rendering. Scripting a value back
   is not an Undo test. View both shipped WGSL examples and GLSL regressions.
5. Run `queue`, then close the interactive host normally and invoke that
   installation's `aerender.exe -project QUEUE_PROJECT -v ERRORS_AND_PROGRESS`.
   Read `QUEUE_PROJECT` from the queue JSON; do not guess its path. Require
   independent completion and all 24 expected PSD frames. Use the actual
   installed version's `aerender -help` if invocation options differ.
6. Cover the existing canvas/expansion, SmartRender and MFR host obligations
   from ADR-0014 on this candidate. Retain visible Full/Half/Quarter evidence;
   `sampleImage` at a requested setting alone does not prove physical output
   size. Record DX12 adapter and driver, AE `app.version`/build, OS, depth,
   MFR configuration, plugin hash and test project identity for each leg.

Feed completed JSON records to
[check_acceptance.py](../scripts/wgsl/check_acceptance.py), for example
`python scripts/wgsl/check_acceptance.py CAPTURE.json RESOURCES.json KEYS.json`;
check `--help` for the tagged CLI before running. Validate the independent
render with `--queue QUEUE.json --exports OUTPUT_DIRECTORY`. The checker
certifies only supplied records; it cannot establish missing UI, MFR, or host
year obligations. The existing M2/M3 PowerShell drivers contain historical
absolute checkout paths and do not reliably aggregate every missing-output
case into a failing exit code. Review/adapt a copy for additional legacy
scenarios; do not label their uninspected exit-zero result a regression PASS.

Publish only the Windows AE years actually verified. Old 2025/2026 evidence
does not cover the 0.1.0 binary; 2023/2024 remain NOT_RUN or explicitly BLOCKED
until real host runs exist. A DX12 failure is not permission to substitute a
different backend silently. Document any missing `FLOAT32_FILTERABLE` resource
capability rather than treating skipped Path/Gradient cases as passed.

## 4. Package the tested bytes and preserve dependency notices

Use the historical Windows root layout and name
`DynamicFX-0.1.0-win-x64.zip`. Include exactly the frozen `DynamicFx.aex`,
`README.md`, Windows-specific `INSTALL.txt`, `LICENSE`, `examples/`,
`skills/dynamicfx-shaders/`, `build/Cargo.lock`, `build/windows-build.json`,
`third-party/`, and an internal `SHA256SUMS`. The installation text must name
the verified host subset and AE-only path. Do not copy macOS installation or
signature claims into it. Include relevant authoring documents or rewrite
their missing local links to the exact v0.1.0 source tag.

Generate notices from the **Windows target's** locked dependency graph,
retaining the released reviewed supplements where applicable:

```powershell
python scripts/release/dependency_notices.py --expected-version 0.1.0 `
    --target x86_64-pc-windows-msvc --toolchain 1.97.1 `
    --supplements "$Evidence/released-macos/third-party/manifest.json" `
    --out "$Evidence/windows-third-party"
Native-OK
python scripts/release/preflight.py --expected-version 0.1.0
Native-OK
```

The notice collector may require additional Windows-only license texts; resolve
missing notices with their actual upstream provenance before packaging. Do not
copy the macOS dependency inventory and claim it is the Windows graph. Review
publication content for private paths, credentials and unrelated projects.
`preflight.py --archive` and `package_macos.py` validate/package macOS bundles,
so they are **not** Windows ZIP validators.

Create internal SHA-256 entries for every regular staged file except
`SHA256SUMS` itself, using relative `/` paths. Create the ZIP once from the
stage (no source rebuild), extract into a new directory, reject duplicate,
absolute/traversing or unexpected member names, and verify every checksum
against extracted bytes. Check that the extracted AEX is byte-identical to
the installed host-tested `$Frozen`. Keep PDBs, raw host dumps and machine
logs as private evidence unless separately reviewed for publication.

## 5. Append to the same release, download again, and close the record

Record the ZIP SHA-256 and size, Windows AEX SHA-256 and size, tag commit,
lock identity, exact AE/OS/GPU subset and host evidence summary. Retain the
pre-upload asset list and macOS checksums. Upload the new Windows ZIP without
`--clobber`; an existing asset with that name must be investigated first:

```powershell
$WinZip = Join-Path $Evidence 'DynamicFX-0.1.0-win-x64.zip'
$WinHash = (Get-FileHash $WinZip -Algorithm SHA256).Hash.ToLowerInvariant()
gh release upload v0.1.0 $WinZip --repo $Repo
Native-OK
```

Prepare an updated `SHA256SUMS.txt` by preserving **all existing rows** and
appending the Windows ZIP row. Review the complete file before replacing
only that checksum asset with `gh release upload v0.1.0 SHA256SUMS.txt --repo
JUNKDOGE-JOE/dynamicfx --clobber`. Preserve the existing release description
and add the Windows verification date, exact supported/tested subset and
remaining limitations using a reviewed notes file (`gh release edit ...
--notes-file ...`). Keep the regular-release status and the original tag.
Do not replace the macOS ZIP or retroactively expand its acceptance claims.

Download the **uploaded** Windows ZIP and updated checksum file with
`gh release download v0.1.0 --repo JUNKDOGE-JOE/dynamicfx --pattern ASSET_NAME
--dir NEW_EMPTY_DIRECTORY`, using a new directory for both files. Match the
ZIP hash to the pre-upload candidate, extract it, and match
`DynamicFx.aex` to `$FrozenHash`. Do not reuse the local candidate as a
purported download verification.

Also verify the downloaded ZIP against the downloaded external checksum row,
every internal checksum, unchanged macOS checksum rows/assets, and the remote
tag's unchanged resolved commit. Update the project test matrix/release audit
with concrete evidence after successful execution; do not amend this original
NOT_RUN procedure into a fabricated historical PASS. Backfill is complete only
when source/lock, tested AEX, installed AEX, packaged AEX and re-downloaded AEX
have the recorded identity chain and the release accurately states the actual
Windows coverage.
