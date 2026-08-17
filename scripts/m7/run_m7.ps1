# M7 performance baseline driver (audit 07). The benchmark needs AE COLD
# with DYNAMICFX_PERF=1 in the environment (cold-start gate), so the
# scenario stage refuses to reuse a warm AE it did not start.
#
#   pwsh scripts/m7/run_m7.ps1 -Year 2025              # benchmark matrix (AE left open)
#   pwsh scripts/m7/run_m7.ps1 -Year 2025 -QuitAE      # quit + collect plugin log
#   pwsh scripts/m7/run_m7.ps1 -Year 2025 -Summarize   # median/p95 tables
param(
    [int]$Year = 2025,
    [switch]$QuitAE,
    [switch]$Summarize,
    [int]$TimeoutSec = 1500
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m7\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M7_OUT = ($outDir -replace '\\', '/')

function Wait-Sentinel([string]$log, [int]$timeout) {
    $deadline = (Get-Date).AddSeconds($timeout)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'RESULT_DONE' -Quiet)) { return $true }
        Start-Sleep -Seconds 5
    }
    return $false
}

if ($QuitAE) {
    for ($try = 1; $try -le 3; $try++) {
        Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m7" 'm7q_quit.jsx') | Out-Null
        $deadline = (Get-Date).AddSeconds(45)
        while ((Get-Date) -lt $deadline) {
            if (-not (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue)) {
                Write-Host '<<< AE exited'
                $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
                if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_plugin.log') -Force }
                exit 0
            }
            Start-Sleep -Seconds 3
        }
        Write-Host "... quit attempt $try did not land; retrying"
    }
    Write-Host '!!! AE did not exit'; exit 4
}

if ($Summarize) {
    & python (Join-Path "$root\scripts\m7" 'summarize_perf.py') $outDir 2>&1 | ForEach-Object { Write-Host $_ }
    exit $LASTEXITCODE
}

# ---- benchmark matrix ----
if (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue) {
    Write-Host 'FATAL: AfterFX is already running; the perf gate needs a cold start. Run -QuitAE first.'
    exit 5
}
# Archive prior artifacts (leftover files raise modal overwrite prompts).
Get-ChildItem $outDir -Filter '*.psd' -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem $outDir -Filter 'm7.aep' -ErrorAction SilentlyContinue | Remove-Item -Force
$plugLog = Join-Path $env:TEMP 'dynamicfx.log'
if (Test-Path $plugLog) {
    Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
}
$benchLog = Join-Path $outDir 'm7bench.log'
if (Test-Path $benchLog) {
    Move-Item $benchLog (Join-Path $outDir ("m7bench_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
}

$env:DYNAMICFX_PERF = '1'
Write-Host '>>> m7bench (cold AE, DYNAMICFX_PERF=1)'
# Warm-start FIRST (plain launch, no -r), exactly like the m1-m6 drivers:
# AE's late startup modules (home screen etc.) then initialize against a
# clean untitled project. Cold `-r` launches let our script dirty the
# project before those modules finish, and one of them requests a project
# close — the "save before closing" modal that deadlocked three runs
# (measured 2026-08-13). The perf env is inherited either way.
Write-Host '... warm-starting AfterFX (DYNAMICFX_PERF=1)'
Start-Process -FilePath $ae | Out-Null
$deadline = (Get-Date).AddSeconds(180)
$warm = $false
while ((Get-Date) -lt $deadline) {
    $p = @(Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle })
    if ($p.Count -gt 0) { $warm = $true; Start-Sleep -Seconds 20; break }
    Start-Sleep -Seconds 3
}
if (-not $warm) { Write-Host '!!! AfterFX did not appear'; exit 2 }
$launched = $false
for ($try = 1; $try -le 4 -and -not $launched; $try++) {
    if ($try -gt 1) { Write-Host "... relaunch (attempt $try)"; Start-Sleep -Seconds 8 }
    Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m7" 'm7bench.jsx') | Out-Null
    $appear = (Get-Date).AddSeconds(40)
    while ((Get-Date) -lt $appear) {
        if (Test-Path $benchLog) { $launched = $true; break }
        Start-Sleep -Seconds 1
    }
}
if (-not $launched) { Write-Host '!!! m7bench never started'; exit 2 }
if (-not (Wait-Sentinel $benchLog $TimeoutSec)) {
    Write-Host '!!! m7bench TIMEOUT'
    if (Test-Path $benchLog) { Get-Content $benchLog | ForEach-Object { "    $_" } | Write-Host }
    exit 2
}
Get-Content $benchLog | ForEach-Object { "    $_" } | Write-Host
Write-Host 'BENCH_DONE (AE left open; use -QuitAE to collect the plugin log, then -Summarize)'
exit 0
