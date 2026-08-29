$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$probeRoot = $PSScriptRoot
$outDir = Join-Path $probeRoot 'out'
$sourceDll = Join-Path $probeRoot 'target/release/dynamicfx_probe.dll'
$builds = @(
    @{ Legs = ''; Artifact = 'base' }
    @{ Legs = 'u1'; Artifact = 'u1' }
    @{ Legs = 'u2'; Artifact = 'u2' }
    @{ Legs = 'u2b'; Artifact = 'u2b' }
    @{ Legs = 'u1,u2,u2b'; Artifact = 'u1-u2-u2b' }
    @{ Legs = 'u1nil'; Artifact = 'u1nil' }
    @{ Legs = 'u3'; Artifact = 'u3' }
    @{ Legs = 'u4'; Artifact = 'u4' }
    @{ Legs = 'u146'; Artifact = 'u146' }
    @{ Legs = 'u3,u4,u146'; Artifact = 'u3-u4-u146' }
)

function Get-Sha256Hex([string] $path) {
    $stream = [System.IO.File]::OpenRead($path)
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        return [System.BitConverter]::ToString($sha256.ComputeHash($stream)).Replace('-', '')
    }
    finally {
        $sha256.Dispose()
        $stream.Dispose()
    }
}

$hadLegs = Test-Path Env:DFXP_LEGS
$previousLegs = $env:DFXP_LEGS
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

Push-Location $probeRoot
try {
    $artifacts = @()
    foreach ($build in $builds) {
        $env:DFXP_LEGS = $build.Legs
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed for DFXP_LEGS='$($build.Legs)'"
        }

        $destination = Join-Path $outDir "DynamicFxProbe-$($build.Artifact).aex"
        Copy-Item -LiteralPath $sourceDll -Destination $destination -Force
        $artifacts += Get-Item -LiteralPath $destination
    }

    $sums = $artifacts |
        Sort-Object Name |
        ForEach-Object {
            "$(Get-Sha256Hex $_.FullName)  $($_.Name)"
        }
    Set-Content -LiteralPath (Join-Path $outDir 'SHA256SUMS.txt') -Value $sums -Encoding ascii
}
finally {
    if ($hadLegs) {
        $env:DFXP_LEGS = $previousLegs
    }
    else {
        Remove-Item Env:DFXP_LEGS -ErrorAction SilentlyContinue
    }
    Pop-Location
}
