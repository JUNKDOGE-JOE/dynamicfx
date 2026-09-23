$ErrorActionPreference = 'Stop'
if (Get-Process AfterFX,aerender -ErrorAction SilentlyContinue) { throw 'Close AE and aerender before replacing the probe' }
$probeSource = Join-Path $PSScriptRoot 'DynamicFxHostShapeProbe-v007.aex'
$probeTarget = 'C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\DynamicFxHostShapeProbe.aex'
$productionTarget = 'C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\DynamicFx.aex'
$oldHash = '795B7C6F3ED54F3FE9B18082FCF61354AFCF016CC27C43DAD19B5724D35CA600'
$newHash = 'FC98A3C4CA372DA81F092F6B9F35976EC9E28AF3E393171D85A0EBEDA23AEFE5'
if ((Get-FileHash -LiteralPath $probeSource -Algorithm SHA256).Hash -ne $newHash) { throw 'Candidate hash mismatch' }
if ((Get-FileHash -LiteralPath $probeTarget -Algorithm SHA256).Hash -ne $oldHash) { throw 'Installed probe differs from the verified baseline' }
$productionBefore = (Get-FileHash -LiteralPath $productionTarget -Algorithm SHA256).Hash
$backup = Join-Path $PSScriptRoot 'installed-probe-v006.aex'
if (Test-Path -LiteralPath $backup) { throw 'Backup already exists; inspect the previous install outcome' }
Copy-Item -LiteralPath $probeTarget -Destination $backup
if ((Get-FileHash -LiteralPath $backup -Algorithm SHA256).Hash -ne $oldHash) { throw 'Backup hash mismatch' }
Copy-Item -LiteralPath $probeSource -Destination $probeTarget -Force
if ((Get-FileHash -LiteralPath $probeTarget -Algorithm SHA256).Hash -ne $newHash) { throw 'Installed candidate hash mismatch' }
if ((Get-FileHash -LiteralPath $productionTarget -Algorithm SHA256).Hash -ne $productionBefore) { throw 'Production plugin changed unexpectedly' }
$record = [pscustomobject]@{Time=(Get-Date).ToString('o');Probe=$probeTarget;OldSHA256=$oldHash;NewSHA256=$newHash;ProductionSHA256=$productionBefore;AEClosed=$true}
$record | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'install-v007.json') -Encoding utf8
$record | ConvertTo-Json


