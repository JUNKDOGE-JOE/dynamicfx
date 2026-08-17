# M4 multi-pass harness driver (ADR-0014 §5). Session 1 runs the graph
# scenarios; session 2 re-runs the two-pass chain with DYNAMICFX_NO_ALIAS=1
# for the ADR-0020 A/B obligation (env vars only apply on a cold start).
#
#   pwsh scripts/m4/run_m4.ps1 -Year 2025           # both sessions
#   pwsh scripts/m4/run_m4.ps1 -Year 2025 -Checks   # numeric probes
param(
    [int]$Year = 2025,
    [switch]$Checks,
    [int]$TimeoutSec = 240,
    [int]$IdleWaitSec = 12
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m4\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M4_OUT = ($outDir -replace '\\', '/')

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
    Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m4" $jsx) | Out-Null
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
    Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m4" 'm4q_quit.jsx') | Out-Null
    $deadline = (Get-Date).AddSeconds(90)
    while ((Get-Date) -lt $deadline) {
        if (-not (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue)) { Write-Host '<<< AE exited'; return $true }
        Start-Sleep -Seconds 3
    }
    Write-Host '!!! AE did not exit in 90s'; return $false
}

if (-not $Checks) {
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) {
        Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }

    # Session 1: aliasing on (default).
    $env:DYNAMICFX_NO_ALIAS = ''
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm4a' 'm4a_twopass.jsx' $true
    Invoke-Scenario 'm4b' 'm4b_verify_two.jsx' $true
    Invoke-Scenario 'm4c' 'm4c_verify_three.jsx' $true
    Invoke-Scenario 'm4d' 'm4d_verify_raw.jsx' $true
    Invoke-Scenario 'm4e' 'm4e_verify_env1.jsx' $true
    Invoke-Scenario 'm4f' 'm4f_verify_bad.jsx' $false
    if (-not (Stop-AEAndWait)) { exit 4 }

    # Session 2: aliasing off (cold start picks up the env var).
    $env:DYNAMICFX_NO_ALIAS = '1'
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm4g' 'm4g_noalias.jsx' $true
    Invoke-Scenario 'm4h' 'm4h_noalias_verify.jsx' $false
    if (-not (Stop-AEAndWait)) { exit 4 }
    $env:DYNAMICFX_NO_ALIAS = ''

    if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_plugin.log') -Force }
    Write-Host 'GUI sessions complete.'
}

if ($Checks) {
    $py = 'python'
    $png = Join-Path $root 'scripts\spike\check_png.py'
    Write-Host '--- numeric checks ---'
    # probe = file, x, y, r, g, b, tol
    foreach ($probe in @(
            @('m4b_two.png', 160, 120, 191, 191, 255, 3),
            @('m4b_two.png', 32, 120, 242, 191, 255, 3),
            @('m4c_three.png', 160, 120, 64, 64, 0, 3),
            @('m4d_raw.png', 160, 120, 64, 64, 0, 3),
            @('m4e_env1.png', 160, 120, 64, 64, 0, 3),
            @('m4f_bad.png', 160, 120, 10, 200, 30, 2),
            @('m4h_noalias.png', 160, 120, 191, 191, 255, 3),
            @('m4h_noalias.png', 32, 120, 242, 191, 255, 3))) {
        $file = $probe[0]; $x = $probe[1]; $y = $probe[2]
        $r = $probe[3]; $g = $probe[4]; $b = $probe[5]; $tol = $probe[6]
        $path = Join-Path $outDir $file
        if (-not (Test-Path $path)) { Write-Host "CHECK $file SKIP (missing)"; continue }
        & $py $png $path $x $y $r $g $b $tol
        Write-Host "CHECK $file ($x,$y) expect ($r,$g,$b) exit=$LASTEXITCODE"
    }
}
