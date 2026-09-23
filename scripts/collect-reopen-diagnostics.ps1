param([string]$OutputPath = (Join-Path (Get-Location) 'dynamicfx-reopen-diagnostics.json'))

$ErrorActionPreference = 'Stop'
$hostsFound = @()
foreach ($directory in Get-ChildItem -LiteralPath (Join-Path $env:ProgramFiles 'Adobe') -Directory -ErrorAction SilentlyContinue) {
    if ($directory.Name -notlike 'Adobe After Effects *') { continue }
    $executable = Join-Path $directory.FullName 'Support Files/AfterFX.exe'
    if (-not (Test-Path -LiteralPath $executable)) { continue }
    $components = @()
    foreach ($name in @('DynamicFx.aex', 'DynamicFxCoverageReader.aex')) {
        $artifact = Join-Path $directory.FullName ('Support Files/Plug-ins/DynamicFx/' + $name)
        $components += [ordered]@{
            name = $name
            present = (Test-Path -LiteralPath $artifact)
            sha256 = $(if (Test-Path -LiteralPath $artifact) { (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash } else { $null })
        }
    }
    $hostsFound += [ordered]@{
        host = $directory.Name
        version = (Get-Item -LiteralPath $executable).VersionInfo.FileVersion
        components = $components
    }
}
$report = [ordered]@{
    collectedAtUtc = [DateTime]::UtcNow.ToString('o')
    windowsVersion = [Environment]::OSVersion.Version.ToString()
    hosts = $hostsFound
    note = 'Read-only inventory. No project, source code, logs, account, machine name or user paths collected.'
}
$resolvedOutput = [IO.Path]::GetFullPath($OutputPath)
if (Test-Path -LiteralPath $resolvedOutput) { throw 'Output already exists; choose a new OutputPath.' }
$json = $report | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($resolvedOutput, $json, (New-Object Text.UTF8Encoding($false)))
Write-Output 'Diagnostic inventory saved. It is local and has not been uploaded.'
