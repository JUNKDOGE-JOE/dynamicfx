# M6 temporal harness driver (ADR-0014 §5, ADR-0023/0024). One warm session
# runs the scheduleTask fixture; -Aerender runs the fresh-process leg on the
# saved project; -Checks runs the numeric gate.
#
#   pwsh scripts/m6/run_m6.ps1 -Year 2025                 # scenarios (AE stays open for the screenshot)
#   pwsh scripts/m6/run_m6.ps1 -Year 2025 -QuitAE         # quit the warm session
#   pwsh scripts/m6/run_m6.ps1 -Year 2025 -Aerender       # fresh-process leg
#   pwsh scripts/m6/run_m6.ps1 -Year 2025 -Checks         # numeric probes
param(
    [int]$Year = 2025,
    [switch]$Checks,
    [switch]$Aerender,
    [switch]$QuitAE,
    [int]$TimeoutSec = 420,
    [int]$IdleWaitSec = 12
)
$ErrorActionPreference = 'Continue'
$root = 'E:\Code\AePlugin_Dynamicfx'
$sf = "C:\Program Files\Adobe\Adobe After Effects $Year\Support Files"
$ae = Join-Path $sf 'AfterFX.exe'
$aerenderExe = Join-Path $sf 'aerender.exe'
if (-not (Test-Path $ae)) { Write-Host "FATAL: $ae not found"; exit 1 }
$outDir = Join-Path $root "scripts\out\m6\$Year"
New-Item -ItemType Directory -Force $outDir | Out-Null
$env:DFX_M6_OUT = ($outDir -replace '\\', '/')

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
function Invoke-Scenario([string]$name, [string]$jsx) {
    $log = Join-Path $outDir "$name.log"
    if (Test-Path $log) {
        Move-Item $log (Join-Path $outDir ("{0}_{1:yyyyMMdd_HHmmss}.log" -f $name, (Get-Date))) -Force
    }
    Write-Host ">>> $name"
    $launched = $false
    for ($try = 1; $try -le 4 -and -not $launched; $try++) {
        if ($try -gt 1) { Write-Host "... relaunch $name (attempt $try)"; Start-Sleep -Seconds 8 }
        Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m6" $jsx) | Out-Null
        $appear = (Get-Date).AddSeconds(25)
        while ((Get-Date) -lt $appear) {
            if (Test-Path $log) { $launched = $true; break }
            Start-Sleep -Seconds 1
        }
    }
    if (-not $launched) { Write-Host "!!! $name never started"; exit 2 }
    if (-not (Wait-Sentinel $log $TimeoutSec)) {
        Write-Host "!!! $name TIMEOUT"
        if (Test-Path $log) { Get-Content $log | ForEach-Object { "    $_" } | Write-Host }
        exit 2
    }
    Get-Content $log | ForEach-Object { "    $_" } | Write-Host
    Start-Sleep -Seconds 4
}

if ($QuitAE) {
    for ($try = 1; $try -le 3; $try++) {
        Start-Process -FilePath $ae -ArgumentList '-r', (Join-Path "$root\scripts\m6" 'm6q_quit.jsx') | Out-Null
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

if ($Aerender) {
    $aep = Join-Path $outDir 'm6.aep'
    if (-not (Test-Path $aep)) { Write-Host 'FATAL: m6.aep missing (run scenarios first)'; exit 1 }
    Get-ChildItem $outDir -Filter 'm6_ar_*.psd' -ErrorAction SilentlyContinue | Remove-Item -Force
    Write-Host '>>> aerender fresh-process leg'
    & $aerenderExe -project $aep -comp 'm6rq' -s 0 -e 24 -OMtemplate 'Photoshop' -output (Join-Path $outDir 'm6_ar_[#####].psd') 2>&1 |
        Select-Object -Last 12 | ForEach-Object { "    $_" } | Write-Host
    $made = @(Get-ChildItem $outDir -Filter 'm6_ar_*.psd' -ErrorAction SilentlyContinue).Count
    Write-Host "AERENDER_FRAMES $made"
    exit $(if ($made -ge 25) { 0 } else { 3 })
}

if (-not $Checks) {
    # Archive prior artifacts (leftover files raise modal overwrite prompts).
    Get-ChildItem $outDir -Filter '*.psd' -ErrorAction SilentlyContinue | Remove-Item -Force
    Get-ChildItem $outDir -Filter 'm6.aep' -ErrorAction SilentlyContinue | Remove-Item -Force
    $plugLog = Join-Path $env:TEMP 'dynamicfx.log'
    if (Test-Path $plugLog) {
        Move-Item $plugLog (Join-Path $outDir ("dynamicfx_pre_{0:yyyyMMdd_HHmmss}.log" -f (Get-Date))) -Force
    }
    if (-not (Start-WarmAE)) { exit 3 }
    Invoke-Scenario 'm6all' 'm6all.jsx'
    Invoke-Scenario 'm6shot' 'm6shot.jsx'
    Write-Host 'SCENARIOS_DONE (AE left open for the screenshot; use -QuitAE afterwards)'
    exit 0
}

# ---- numeric gate ----
& python (Join-Path "$root\scripts\m6" 'check_m6.py') $outDir 2>&1 | ForEach-Object { Write-Host $_ }
exit $LASTEXITCODE
