# 0.0.3 feature-batch host driver (ADR-0014 §5).
#
# Shape copied from run_m1..m7 rather than invented, and the reason is a
# finding this repository already recorded at M0: **cold `AfterFX.exe -r` is
# unreliable on this machine**. The driver boots AE plainly once, waits for its
# main window, and only then forwards scenarios via `-r` into that warm
# instance. Re-learned the hard way on 2026-08-15 — a cold `-r` launch runs the
# script body but its `app.scheduleTask` callbacks never fire, with no error
# anywhere, so a leg logs its setup and then stops.
#
#   pwsh scripts/f003/run_f003.ps1 -Year 2025
#   pwsh -Command "& .\run_f003.ps1 -Year 2025 -Scenarios @('a','c')"
#
# Outputs land in scripts/out/f003/<Year>/ (gitignored); curated evidence is
# copied to docs/audits/evidence/ when results are recorded.
param(
    [int]$Year = 2025,
    [string[]]$Scenarios = @('a', 'b', 'c'),
    [int]$TimeoutSec = 240,
    [int]$IdleWaitSec = 12,
    [switch]$QuitAE
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }

$aex = Join-Path $sf 'Plug-ins\DynamicFx\DynamicFx.aex'
if (-not (Test-Path $aex)) { Write-Host "FATAL: DynamicFx.aex not installed for $Year"; exit 1 }
Write-Host ("artifact SHA-256 " + (Get-FileHash -Algorithm SHA256 $aex).Hash)

$outDir = Join-Path $root "scripts\out\f003\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_F003_OUT = ($outDir -replace '\\', '/')

$map = [ordered]@{
    a = 'f003a_layer.jsx'
    b = 'f003b_gradient.jsx'
    c = 'f003c_marker.jsx'
    f = 'f003f_point3d.jsx'
    g = 'f003g_path.jsx'
    h = 'f003h_range.jsx'
}

function Wait-Sentinel([string]$log, [int]$timeout) {
    $deadline = (Get-Date).AddSeconds($timeout)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'RESULT_DONE' -Quiet)) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

# Warm start (see the header note).
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
    if (-not $map.Contains($s)) { Write-Host "skip unknown scenario '$s'"; continue }
    $jsx = Join-Path $root "scripts\f003\$($map[$s])"
    $log = Join-Path $outDir "f003$s.log"
    Remove-Item -Force $log -ErrorAction SilentlyContinue
    Write-Host "=== f003$s : $($map[$s])"
    Start-Process -FilePath $ae -ArgumentList @('-r', $jsx) | Out-Null
    if (Wait-Sentinel $log $TimeoutSec) {
        Get-Content $log | ForEach-Object { Write-Host "    $_" }
    } else {
        Write-Host "    TIMEOUT after ${TimeoutSec}s (no RESULT_DONE)"
        if (Test-Path $log) { Get-Content $log | ForEach-Object { Write-Host "    $_" } }
    }
    # Driver-side idle window: the 1-second AEGP scan needs main-thread time
    # that a back-to-back script launch would deny it.
    Start-Sleep -Seconds $IdleWaitSec
}

if ($QuitAE) {
    Write-Host '=== quitting AE'
    Start-Process -FilePath $ae -ArgumentList @('-r', (Join-Path $root 'scripts\m1\m1q_quit.jsx')) | Out-Null
    Start-Sleep -Seconds 20
}
