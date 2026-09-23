$ErrorActionPreference = 'Stop'
if (Get-Process AfterFX,aerender -ErrorAction SilentlyContinue) { throw 'Close AE and aerender before replacing the probe' }
$probeSource = Join-Path $PSScriptRoot 'DynamicFxHostShapeProbe-v006.aex'
$probeTarget = 'C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\DynamicFxHostShapeProbe.aex'
$productionTarget = 'C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\DynamicFx.aex'
$oldHash = '5DA92F37B1697FE1249E65E9795CDD636FE2D6B31991E0D7B01A6F7585562333'
$newHash = '795B7C6F3ED54F3FE9B18082FCF61354AFCF016CC27C43DAD19B5724D35CA600'
if ((Get-FileHash -LiteralPath $probeSource -Algorithm SHA256).Hash -ne $newHash) { throw 'Candidate hash mismatch' }
if ((Get-FileHash -LiteralPath $probeTarget -Algorithm SHA256).Hash -ne $oldHash) { throw 'Installed probe differs from the verified baseline' }
$productionBefore = (Get-FileHash -LiteralPath $productionTarget -Algorithm SHA256).Hash
$backup = Join-Path $PSScriptRoot 'installed-probe-v005.aex'
if (Test-Path -LiteralPath $backup) { throw 'Backup already exists; inspect the previous install outcome' }
Copy-Item -LiteralPath $probeTarget -Destination $backup
if ((Get-FileHash -LiteralPath $backup -Algorithm SHA256).Hash -ne $oldHash) { throw 'Backup hash mismatch' }
Copy-Item -LiteralPath $probeSource -Destination $probeTarget -Force
if ((Get-FileHash -LiteralPath $probeTarget -Algorithm SHA256).Hash -ne $newHash) { throw 'Installed candidate hash mismatch' }
if ((Get-FileHash -LiteralPath $productionTarget -Algorithm SHA256).Hash -ne $productionBefore) { throw 'Production plugin changed unexpectedly' }
$record = [pscustomobject]@{Time=(Get-Date).ToString('o');Probe=$probeTarget;OldSHA256=$oldHash;NewSHA256=$newHash;ProductionSHA256=$productionBefore;AEClosed=$true}
$record | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'install-v006.json') -Encoding utf8
$record | ConvertTo-Json


