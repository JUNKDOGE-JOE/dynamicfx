# M3 persistence harness driver (ADR-0014 §5). Three AE sessions: author+
# save, fresh-process reopen, corrupted-copy recovery; then an aerender leg
# and numeric checks.
#
#   pwsh scripts/m3/run_m3.ps1 -Year 2025             # all three GUI sessions
#   pwsh scripts/m3/run_m3.ps1 -Year 2025 -Aerender   # aerender leg
#   pwsh scripts/m3/run_m3.ps1 -Year 2025 -Checks     # numeric probes
param(
    [int]$Year = 2025,
    [switch]$Aerender,
    [switch]$Checks,
    [switch]$Session4,
    [int]$TimeoutSec = 240,
    [int]$IdleWaitSec = 12
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
$aer = Join-Path $sf 'aerender.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m3\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M3_OUT = ($outDir -replace '\\', '/')

function Wait-Sentinel([string]$log, [int]$timeout) {
    $deadline = (Get-Date).AddSeconds($timeout)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'RESULT_DONE' -Quiet)) { return $true }
        Start-Sleep -Seconds 2
    }
    return $false
}

function Start-WarmAE {
    if (-not (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue)) {
        Write-Host '... starting AfterFX'
        Start-Process -FilePath $ae | Out-Null
        $deadline = (Get-Date).AddSeconds(180)
        while ((Get-Date) -lt $deadline) {
            $p = @(Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle })
            if ($p.Count -gt 0) { Start-Sleep -Seconds 5; return $true }
            Start-Sleep -Seconds 3
        }
        Write-Host '!!! AfterFX did not appear'; return $false
    }
    return $true
}

function Invoke-Scenario([string]$name, [string]$jsx, [bool]$idleAfter) {
    $log = Join-Path $outDir "$name.log"
    if (Test-Path $log) {
        Move-Item $log (Join-Path $outDir ("{0}_{1:yyyyMMdd_HHmmss}.log" -f $name, (Get-Date))) -Force
    }
    Write-Host ">>> $name"
    Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m3" $jsx) | Out-Null
    if (-not (Wait-Sentinel $log $TimeoutSec)) {
        Write-Host "!!! $name TIMEOUT"
        if (Test-Path $log) { Get-Content $log | ForEach-Object { "    $_" } | Write-Host }
        exit 2
    }
    Get-Content $log | ForEach-Object { "    $_" } | Write-Host
    if ($idleAfter) {
        Write-Host "... idle window ${IdleWaitSec}s"
        Start-Sleep -Seconds $IdleWaitSec
    }
}

function Stop-AEAndWait {
    Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m3" 'm3q_quit.jsx') | Out-Null
    $deadline = (Get-Date).AddSeconds(90)
    while ((Get-Date) -lt $deadline) {
        if (-not (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue)) { Write-Host '<<< AE exited'; return $true }
        Start-Sleep -Seconds 3
    }
    Write-Host '!!! AE did not exit in 90s'; return $false
}

if (-not $Aerender -and -not $Checks) {
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) {
        Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }

    # Session 1: author, publish, keyframe, save.
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm3a' 'm3a_setup.jsx' $true
    Invoke-Scenario 'm3b' 'm3b_save.jsx' $false
    if (-not (Stop-AEAndWait)) { exit 4 }

    # Session 2: fresh process, reopen, render with no Compile click.
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm3c' 'm3c_reopen.jsx' $true
    Invoke-Scenario 'm3d' 'm3d_names.jsx' $false
    if (-not (Stop-AEAndWait)) { exit 4 }

    # Session 3: corrupted copy, fail closed, recover via the expression.
    & python (Join-Path "$root\scripts\m3" 'corrupt_snapshot.py') $outDir
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'CORRUPT LEG SKIP: no snapshot signature found'
    } else {
        if (-not (Start-WarmAE)) { exit 3 }
        Invoke-Scenario 'm3e' 'm3e_corrupt_open.jsx' $true
        Invoke-Scenario 'm3f' 'm3f_recover.jsx' $false
        if (-not (Stop-AEAndWait)) { exit 4 }
    }

    if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_plugin.log') -Force }
    Write-Host 'GUI sessions complete.'
}

if ($Session4) {
    # Session 4 (same installed artifact): duplicate isolation, torn token,
    # undo convergence, save/dirty.
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm3g' 'm3g_duplicate.jsx' $false
    Invoke-Scenario 'm3h1' 'm3h1_torn.jsx' $true
    Invoke-Scenario 'm3h2' 'm3h2_verify_correct.jsx' $true
    Invoke-Scenario 'm3h3' 'm3h3_undo.jsx' $true
    Invoke-Scenario 'm3h4' 'm3h4_converge.jsx' $true
    Invoke-Scenario 'm3h5' 'm3h5_dirty.jsx' $false
    if (-not (Stop-AEAndWait)) { exit 4 }
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_session4.log') -Force }
    Write-Host 'Session 4 complete.'
}

if ($Aerender) {
    if (-not (Test-Path $aer)) { Write-Host "FATAL: $aer not found"; exit 1 }
    $aep = Join-Path $outDir 'm3.aep'
    if (-not (Test-Path $aep)) { Write-Host "skip aerender: $aep missing"; exit 1 }
    Write-Host '>>> aerender m3'
    & $aer -project $aep *> (Join-Path $outDir 'aerender_m3.txt')
    Write-Host "<<< aerender exit=$LASTEXITCODE"
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_aerender.log') -Force }
}

if ($Checks) {
    $py = 'python'
    $png = Join-Path $root 'scripts\spike\check_png.py'
    $rgbPsd = Join-Path $root 'scripts\m1\check_psd_rgb.py'
    Write-Host '--- numeric checks: expect (51,51,0) at center, t=0.4 ---'
    foreach ($file in @('m3c_reopen.png', 'm3d_t04.png', 'm3f_recover.png')) {
        $path = Join-Path $outDir $file
        if (-not (Test-Path $path)) { Write-Host "CHECK $file SKIP (missing)"; continue }
        & $py $png $path 160 120 51 51 0 3
        Write-Host "CHECK $file (160,120) expect (51,51,0) exit=$LASTEXITCODE"
    }
    foreach ($probe in @(
            @('m3g_layer1.png', 51, 51, 0, 3),
            @('m3g_layer2.png', 115, 115, 0, 3),
            @('m3h1_torn.png', 51, 51, 0, 3),
            @('m3h4_undo.png', 51, 51, 0, 3),
            @('m3h3_invalid.png', 10, 200, 30, 2))) {
        $file = $probe[0]; $r = $probe[1]; $g = $probe[2]; $b = $probe[3]; $tol = $probe[4]
        $path = Join-Path $outDir $file
        if (-not (Test-Path $path)) { Write-Host "CHECK $file SKIP (missing)"; continue }
        & $py $png $path 160 120 $r $g $b $tol
        Write-Host "CHECK $file (160,120) expect ($r,$g,$b) exit=$LASTEXITCODE"
    }
    $corrupt = Join-Path $outDir 'm3e_corrupt.png'
    if (Test-Path $corrupt) {
        & $py $png $corrupt 160 120
        Write-Host 'INFO m3e_corrupt.png value above (passthrough solid or already recovered; m3f is the criterion)'
    }
    $psd = Get-ChildItem $outDir -Filter 'm3_ar_*.psd' | Select-Object -First 1
    if ($psd) {
        & $py $rgbPsd $psd.FullName 160 120 51 51 0 4
        Write-Host "CHECK aerender PSD (160,120) expect (51,51,0) exit=$LASTEXITCODE"
    } else {
        Write-Host 'CHECK aerender PSD: missing'
    }
}
