# M2 keyframed-parameters harness driver (ADR-0014 §5). Same warm-AE `-r`
# pattern as scripts/m1/run_m1.ps1, with driver-side idle windows after the
# scripted source writes.
#
#   pwsh scripts/m2/run_m2.ps1 -Year 2025           # GUI pass (a..d) + quit
#   pwsh scripts/m2/run_m2.ps1 -Year 2025 -Checks   # numeric PNG probes only
param(
    [int]$Year = 2025,
    [string[]]$Scenarios = @('a', 'b', 'c', 'd', 'q'),
    [switch]$Checks,
    [int]$TimeoutSec = 240,
    [int]$IdleWaitSec = 12
)
$ErrorActionPreference = 'Continue'
# `pwsh -File` passes "e,f,g" as one string; normalize to a flat list.
$Scenarios = @($Scenarios | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m2\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M2_OUT = ($outDir -replace '\\', '/')

$map = [ordered]@{
    a = 'm2a_write.jsx'
    b = 'm2b_keyframe.jsx'
    c = 'm2c_edit.jsx'
    d = 'm2d_verify.jsx'
    e = 'm2e_annotation.jsx'
    f = 'm2f_alias.jsx'
    g = 'm2g_verify.jsx'
    h = 'm2h_kinds.jsx'
    h2 = 'm2h2_values.jsx'
    i = 'm2i_overflow.jsx'
    j = 'm2j_overflow_verify.jsx'
    q = 'm2q_quit.jsx'
}
$idleAfter = @('a', 'c', 'e', 'f', 'h', 'i')

function Wait-Sentinel([string]$log, [int]$timeout) {
    $deadline = (Get-Date).AddSeconds($timeout)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern 'RESULT_DONE' -Quiet)) { return $true }
        Start-Sleep -Seconds 2
    }
    return $false
}

if (-not $Checks) {
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) {
        Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }
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
        $jsx = Join-Path "$root\scripts\m2" $map[$s]
        $log = Join-Path $outDir "m2$s.log"
        if (Test-Path $log) {
            Move-Item $log (Join-Path $outDir ("m2{0}_{1:yyyyMMdd_HHmmss}.log" -f $s, (Get-Date))) -Force
        }
        Write-Host ">>> m2$s ($($map[$s]))"
        Start-Process -FilePath $ae -ArgumentList '-r', $jsx | Out-Null
        if ($s -eq 'q') { Start-Sleep -Seconds 5; Write-Host '<<< quit requested'; continue }
        if (Wait-Sentinel $log $TimeoutSec) {
            Write-Host "<<< m2$s done"
            Get-Content $log | ForEach-Object { "    $_" } | Write-Host
        } else {
            Write-Host "!!! m2$s TIMEOUT after ${TimeoutSec}s"
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

if ($Checks) {
    $py = 'python'
    $png = Join-Path $root 'scripts\spike\check_png.py'
    Write-Host '--- numeric checks (8-bit expectations) ---'
    # probe = file, x, y, r, g, b, tol
    foreach ($probe in @(
            @('m2b_t0.png', 160, 120, 0, 0, 0, 2),
            @('m2b_t04.png', 160, 120, 102, 102, 102, 3),
            @('m2b_t08.png', 160, 120, 204, 204, 204, 3),
            @('m2d_t04.png', 160, 120, 102, 102, 102, 3),
            @('m2e_default.png', 160, 120, 128, 128, 128, 2),
            @('m2g_t04.png', 160, 120, 128, 128, 128, 3),
            @('m2h_kinds.png', 32, 120, 77, 77, 77, 3),
            @('m2h_kinds.png', 96, 120, 255, 255, 255, 2),
            @('m2h_kinds.png', 160, 120, 255, 128, 64, 3),
            @('m2h_kinds.png', 224, 120, 191, 64, 0, 3),
            @('m2h_kinds.png', 288, 120, 64, 64, 64, 3),
            @('m2i_overflow.png', 160, 120, 10, 200, 30, 2))) {
        $file = $probe[0]; $x = $probe[1]; $y = $probe[2]
        $r = $probe[3]; $g = $probe[4]; $b = $probe[5]; $tol = $probe[6]
        $path = Join-Path $outDir $file
        if (-not (Test-Path $path)) { Write-Host "CHECK $file SKIP (missing)"; continue }
        & $py $png $path $x $y $r $g $b $tol
        Write-Host "CHECK $file ($x,$y) expect ($r,$g,$b) exit=$LASTEXITCODE"
    }
}
