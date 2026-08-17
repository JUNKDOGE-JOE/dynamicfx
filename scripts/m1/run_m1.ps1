# M1 first-frame harness driver (ADR-0014 §5). Runs scenario JSX files in a
# warm AE session via `AfterFX.exe -r`, polling each scenario log for
# RESULT_DONE, with driver-side sleeps between legs so the AEGP idle observer
# gets main-thread time to observe scripted expression writes.
#
#   pwsh scripts/m1/run_m1.ps1 -Year 2025            # GUI pass (a,b,c,d,e) + quit
#   pwsh scripts/m1/run_m1.ps1 -Year 2025 -Aerender  # aerender leg only
#   pwsh scripts/m1/run_m1.ps1 -Year 2025 -Checks    # numeric PNG/PSD checks only
#
# Outputs land in scripts/out/m1/<Year>/ (gitignored); curated evidence is
# copied to docs/audits/evidence/ when results are recorded.
param(
    [int]$Year = 2025,
    [string[]]$Scenarios = @('a', 'b', 'c', 'd', 'e', 'q'),
    [switch]$Aerender,
    [switch]$Checks,
    [int]$TimeoutSec = 240,
    [int]$IdleWaitSec = 12
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
$aer = Join-Path $sf 'aerender.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m1\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M1_OUT = ($outDir -replace '\\', '/')

$map = [ordered]@{
    a = 'm1a_apply.jsx'
    b = 'm1b_write.jsx'
    c = 'm1c_check.jsx'
    d = 'm1d_invalid.jsx'
    e = 'm1e_verify.jsx'
    q = 'm1q_quit.jsx'
}
# Driver-side idle windows AFTER these scenarios (AE must be idle for the
# 1-second idle scan to observe scripted writes and publish the token).
$idleAfter = @('b', 'd')

function Wait-Sentinel([string]$log, [int]$timeout) {
    $deadline = (Get-Date).AddSeconds($timeout)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'RESULT_DONE' -Quiet)) { return $true }
        Start-Sleep -Seconds 2
    }
    return $false
}

if (-not $Aerender -and -not $Checks) {
    # Preserve any prior plugin log, then let the plugin log fresh.
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) {
        Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }
    # Warm start: cold `AfterFX.exe -r` proved unreliable on this machine
    # (M0 spike finding); boot plainly, wait for the main window, then
    # forward scenarios via -r.
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
        $jsx = Join-Path "$root\scripts\m1" $map[$s]
        $log = Join-Path $outDir "m1$s.log"
        if (Test-Path $log) {
            Move-Item $log (Join-Path $outDir ("m1{0}_{1:yyyyMMdd_HHmmss}.log" -f $s, (Get-Date))) -Force
        }
        Write-Host ">>> m1$s ($($map[$s]))"
        Start-Process -FilePath $ae -ArgumentList '-r', $jsx | Out-Null
        if ($s -eq 'q') { Start-Sleep -Seconds 5; Write-Host '<<< quit requested'; continue }
        if (Wait-Sentinel $log $TimeoutSec) {
            Write-Host "<<< m1$s done"
            Get-Content $log | ForEach-Object { "    $_" } | Write-Host
        } else {
            Write-Host "!!! m1$s TIMEOUT after ${TimeoutSec}s"
            if (Test-Path $log) { Get-Content $log | ForEach-Object { "    $_" } | Write-Host }
            exit 2
        }
        if ($idleAfter -contains $s) {
            Write-Host "... idle window ${IdleWaitSec}s (observer)"
            Start-Sleep -Seconds $IdleWaitSec
        }
    }
    if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_plugin.log') -Force }
    Write-Host 'GUI pass complete.'
}

if ($Aerender) {
    if (-not (Test-Path $aer)) { Write-Host "FATAL: $aer not found"; exit 1 }
    $aep = Join-Path $outDir 'm1_ar.aep'
    if (-not (Test-Path $aep)) { Write-Host "skip aerender: $aep missing"; exit 1 }
    Write-Host '>>> aerender m1_ar'
    & $aer -project $aep *> (Join-Path $outDir 'aerender_m1.txt')
    Write-Host "<<< aerender exit=$LASTEXITCODE"
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_plugin.log') -Force }
}

if ($Checks) {
    $py = 'python'
    $png = Join-Path $root 'scripts\spike\check_png.py'
    Write-Host '--- numeric checks (8-bit expectations, tolerance 3) ---'
    # Gradient: R=(x+0.5)/W*255, G=(y+0.5)/H*255, B=0 at 320x240.
    foreach ($probe in @(
            @(16, 16, 13, 18, 0),
            @(160, 120, 128, 128, 0),
            @(304, 224, 243, 239, 0))) {
        $x = $probe[0]; $y = $probe[1]; $r = $probe[2]; $g = $probe[3]; $b = $probe[4]
        & $py $png (Join-Path $outDir 'm1c_gui.png') $x $y $r $g $b 3
        Write-Host "CHECK gradient ($x,$y) expect ($r,$g,$b) exit=$LASTEXITCODE"
    }
    # Pass-through: the solid color must come back untouched.
    & $py $png (Join-Path $outDir 'm1d_invalid.png') 160 120 10 200 30 2
    Write-Host "CHECK passthrough (160,120) expect (10,200,30) exit=$LASTEXITCODE"
    # aerender: report which of the two references the PSD matches.
    $psd = Get-ChildItem $outDir -Filter 'm1_ar_*.psd' | Select-Object -First 1
    if ($psd) {
        $rgb = Join-Path $root 'scripts\m1\check_psd_rgb.py'
        & $py $rgb $psd.FullName 160 120 128 128 0 4
        $asGradient = $LASTEXITCODE
        & $py $rgb $psd.FullName 160 120 10 200 30 4
        $asPassthrough = $LASTEXITCODE
        Write-Host "CHECK aerender PSD: gradient_match=$(if ($asGradient -eq 0) {1} else {0}) passthrough_match=$(if ($asPassthrough -eq 0) {1} else {0})"
    } else {
        Write-Host 'CHECK aerender PSD: missing'
    }
}
