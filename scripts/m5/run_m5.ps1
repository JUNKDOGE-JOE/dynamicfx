# M5 pixel-format harness driver (ADR-0014 §5, ADR-0021). One warm session
# builds the fixture project (idle observer compiles), then renders the
# depth legs to PSD; -Checks runs the numeric probe matrix.
#
#   pwsh scripts/m5/run_m5.ps1 -Year 2025           # scenarios
#   pwsh scripts/m5/run_m5.ps1 -Year 2025 -Checks   # numeric probes
param(
    [int]$Year = 2025,
    [switch]$Checks,
    [string[]]$Scenarios = @(),
    [int]$TimeoutSec = 300,
    [int]$IdleWaitSec = 12
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m5\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M5_OUT = ($outDir -replace '\\', '/')
$Scenarios = @($Scenarios | ForEach-Object { $_ -split ',' })

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
            if ($p.Count -gt 0) { Start-Sleep -Seconds 15; return $true }
            Start-Sleep -Seconds 3
        }
        Write-Host '!!! AfterFX did not appear'; return $false
    }
    return $true
}
function Invoke-Scenario([string]$name, [string]$jsx, [bool]$idleAfter) {
    if ($Scenarios.Count -gt 0 -and $Scenarios -notcontains $name) { return }
    $log = Join-Path $outDir "$name.log"
    if (Test-Path $log) {
        Move-Item $log (Join-Path $outDir ("{0}_{1:yyyyMMdd_HHmmss}.log" -f $name, (Get-Date))) -Force
    }
    Write-Host ">>> $name"
    # AE refuses a -r script while another is still winding down ("second
    # script did not run" dialog) and heavy startups (plugin cache rebuild
    # after a PIPL version bump) keep the engine busy well past the window
    # appearing. If the scenario's log never materializes, the script was
    # rejected — relaunch after a settle.
    $launched = $false
    for ($try = 1; $try -le 4 -and -not $launched; $try++) {
        if ($try -gt 1) { Write-Host "... relaunch $name (attempt $try)"; Start-Sleep -Seconds 8 }
        Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m5" $jsx) | Out-Null
        $appear = (Get-Date).AddSeconds(25)
        while ((Get-Date) -lt $appear) {
            if (Test-Path $log) { $launched = $true; break }
            Start-Sleep -Seconds 1
        }
    }
    if (-not $launched) { Write-Host "!!! $name never started (script rejected repeatedly)"; exit 2 }
    if (-not (Wait-Sentinel $log $TimeoutSec)) {
        Write-Host "!!! $name TIMEOUT"
        if (Test-Path $log) { Get-Content $log | ForEach-Object { "    $_" } | Write-Host }
        exit 2
    }
    Get-Content $log | ForEach-Object { "    $_" } | Write-Host
    # Let the script engine wind down before the next -r launch.
    Start-Sleep -Seconds 4
    if ($idleAfter) {
        Write-Host "... idle window ${IdleWaitSec}s"
        Start-Sleep -Seconds $IdleWaitSec
    }
}
function Stop-AEAndWait {
    for ($try = 1; $try -le 3; $try++) {
        Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m5" 'm5q_quit.jsx') | Out-Null
        $deadline = (Get-Date).AddSeconds(45)
        while ((Get-Date) -lt $deadline) {
            if (-not (Get-Process -Name 'AfterFX' -ErrorAction SilentlyContinue)) { Write-Host '<<< AE exited'; return $true }
            Start-Sleep -Seconds 3
        }
        Write-Host "... quit attempt $try did not land; retrying"
    }
    Write-Host '!!! AE did not exit'; return $false
}

if (-not $Checks) {
    # Archive prior PSD artifacts: a leftover file makes the render queue
    # raise a modal overwrite prompt that deadlocks the scripted session.
    Get-ChildItem $outDir -Filter '*.psd' -ErrorAction SilentlyContinue | ForEach-Object {
        Move-Item $_.FullName ($_.FullName -replace '\.psd$', ("_{0:yyyyMMdd_HHmmss}.psd.bak" -f (Get-Date))) -Force
    }
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) {
        Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm5x' 'm5x_cmprobe.jsx' $false
    Invoke-Scenario 'm5all' 'm5all.jsx' $false
    if ($Scenarios.Count -eq 0) {
        Stop-AEAndWait | Out-Null
        if (Test-Path $plugLog) { Copy-Item $plugLog (Join-Path $outDir 'dynamicfx_plugin.log') -Force }
    }
    Write-Host 'SCENARIOS_DONE'
    exit 0
}

# ---- numeric gates: sampleImage probes parsed from m5all.log ----
$py = 'python'
& $py (Join-Path "$root\scripts\m5" 'check_probes.py') (Join-Path $outDir 'm5all.log') 2>&1 |
    ForEach-Object { Write-Host $_ }
$probeExit = $LASTEXITCODE

# Visible artifacts: record the PSD renders' pixel values and real file
# depth (the OM Depth key is read-only on this host; recorded, not gated).
$chk = Join-Path "$root\scripts\m5" 'check_deep.py'
foreach ($base in @('m5_chain16')) {
    $p = Join-Path $outDir "$base`_00000.psd"
    if (Test-Path $p) {
        & $py $chk 'pixel' $p 100 120 2>&1 | ForEach-Object { Write-Host $_ }
    } else {
        Write-Host "CHECK $base artifact missing RECORD"
    }
}
exit $probeExit
