# M0 transport spike driver (ADR-0009). Runs scenario JSX files in one AE
# session via `AfterFX.exe -r`, polling each scenario log for RESULT_DONE.
#
#   pwsh scripts/spike/run_spike.ps1 -Year 2025                 # s0..s5 + quit
#   pwsh scripts/spike/run_spike.ps1 -Year 2025 -Scenarios s1   # one scenario
#   pwsh scripts/spike/run_spike.ps1 -Year 2025 -Aerender       # aerender legs only
#
# Outputs land in scripts/out/spike/<Year>/ (gitignored); curated evidence is
# copied to docs/audits/evidence/ manually when results are recorded.
param(
    [int]$Year = 2025,
    [string[]]$Scenarios = @('s0', 's1', 's2', 's3', 's4', 's5', 's9'),
    [switch]$Aerender,
    [int]$TimeoutSec = 420,
    # Sequence-payload size (KB) for the probe's flatten() in scenario s4.
    # Read by the plugin at process start via DFX_PROBE_KB, so it only takes
    # effect on a COLD AfterFX launch (the probe run must start a fresh AE).
    [int]$ProbeKb = 0
)
$ErrorActionPreference = 'Continue'
$env:DFX_PROBE_KB = "$ProbeKb"
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
$aer = Join-Path $sf 'aerender.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\spike\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_SPIKE_OUT = ($outDir -replace '\\', '/')

$map = [ordered]@{
    s0 = 's0_init.jsx'
    s1 = 's1_expr_ceiling.jsx'
    s2 = 's2_expr_roundtrip.jsx'
    s3 = 's3_undo_dirty.jsx'
    s4 = 's4_probe_plugin.jsx'
    s5 = 's5_aerender_setup.jsx'
    s9 = 's9_quit.jsx'
}

function Wait-Sentinel([string]$log, [int]$timeout) {
    $deadline = (Get-Date).AddSeconds($timeout)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'RESULT_DONE' -Quiet)) { return $true }
        Start-Sleep -Seconds 2
    }
    return $false
}

if (-not $Aerender) {
    # Preserve any prior probe-plugin log, then let the plugin start fresh.
    $probeLog = Join-Path $env:TEMP 'dynamicfx_probe.log'
    if (Test-Path $probeLog) {
        Move-Item $probeLog (Join-Path $outDir ("probe_plugin_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }
    # Warm start: cold `AfterFX.exe -r <script>` proved unreliable on this
    # machine (2026-08-12: process never appeared). Boot AE plainly first,
    # wait for the main window, then forward every scenario via -r, which is
    # verified to execute in the warm instance.
    if (-not (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue)) {
        Write-Host '... warm-starting AfterFX'
        Start-Process -FilePath $ae | Out-Null
        $bootDeadline = (Get-Date).AddSeconds(180)
        $booted = $false
        while ((Get-Date) -lt $bootDeadline) {
            $p = @(Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle })
            if ($p.Count -gt 0) { $booted = $true; break }
            Start-Sleep -Seconds 3
        }
        if (-not $booted) { Write-Host '!!! AfterFX main window did not appear in 180s'; exit 3 }
        Start-Sleep -Seconds 5
        Write-Host '... AfterFX warm'
    }
    foreach ($s in $Scenarios) {
        if (-not $map.Contains($s)) { Write-Host "skip unknown scenario $s"; continue }
        $jsx = Join-Path "$root\scripts\spike" $map[$s]
        $log = Join-Path $outDir "$s.log"
        if (Test-Path $log) {
            Move-Item $log (Join-Path $outDir ("{0}_{1:yyyyMMdd_HHmmss}.log" -f $s, (Get-Date))) -Force
        }
        Write-Host ">>> $s ($($map[$s]))"
        Start-Process -FilePath $ae -ArgumentList '-r', $jsx | Out-Null
        $tmo = if ($s -eq 's1' -or $s -eq 's2') { [Math]::Max($TimeoutSec, 600) } else { $TimeoutSec }
        if (Wait-Sentinel $log $tmo) {
            Write-Host "<<< $s done"
            Get-Content $log | ForEach-Object { "    $_" } | Write-Host
        } else {
            Write-Host "!!! $s TIMEOUT after ${tmo}s"
            if (Test-Path $log) { Get-Content $log | ForEach-Object { "    $_" } | Write-Host }
            exit 2
        }
    }
    # Snapshot the probe-plugin log alongside the scenario logs.
    if (Test-Path $probeLog) { Copy-Item $probeLog (Join-Path $outDir 'probe_plugin.log') -Force }
    Write-Host "GUI pass complete. Wait for AfterFX.exe to exit before -Aerender."
}

if ($Aerender) {
    if (-not (Test-Path $aer)) { Write-Host "FATAL: $aer not found"; exit 1 }
    if (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue) {
        Write-Host 'WARN: AfterFX still running; aerender proceeds in parallel.'
    }
    foreach ($name in @('s4', 's5')) {
        $aep = Join-Path $outDir "$name.aep"
        if (-not (Test-Path $aep)) { Write-Host "skip aerender ${name}: $aep missing"; continue }
        Write-Host ">>> aerender $name"
        & $aer -project $aep *> (Join-Path $outDir "aerender_$name.txt")
        Write-Host "<<< aerender $name exit=$LASTEXITCODE"
    }
    # Refresh the probe-plugin log snapshot: aerender pids appended to it.
    $probeLog = Join-Path $env:TEMP 'dynamicfx_probe.log'
    if (Test-Path $probeLog) { Copy-Item $probeLog (Join-Path $outDir 'probe_plugin.log') -Force }
    Get-ChildItem $outDir -Filter '*_ar_*.png' | ForEach-Object { Write-Host "rendered: $($_.Name) $($_.Length)b" }
}
