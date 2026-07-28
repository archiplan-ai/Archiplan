#Requires -Version 5
$ErrorActionPreference = 'Stop'

# Archiplan installer. Resolves the latest GitHub release of
# archiplan-ai/Archiplan, downloads the archi tarball for Windows x64,
# verifies its checksum and installs archi.exe to %USERPROFILE%\.local\bin:
#
#   irm https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.ps1 | iex
#
# Pin a version with $env:ARCHI_VERSION; install from a fork or mirror with
# $env:ARCHI_REPO.

$Repo = if ($env:ARCHI_REPO) { $env:ARCHI_REPO } else { 'archiplan-ai/Archiplan' }

$arch = $env:PROCESSOR_ARCHITECTURE
if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
switch ($arch) {
    'AMD64'  { $plat = 'windows-x64' }
    'x86_64' { $plat = 'windows-x64' }
    default {
        Write-Error "Unsupported platform: Windows-$arch. Supported: Windows x86_64."
        exit 1
    }
}

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Version = $env:ARCHI_VERSION
if (-not $Version) {
    try {
        $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" `
            -Headers @{ 'User-Agent' = 'archiplan-installer'; 'Accept' = 'application/vnd.github+json' }
        $Version = $rel.tag_name -replace '^v', ''
    } catch {
        Write-Error ("Could not resolve the latest archi release from {0}: {1}`n" -f $Repo, $_.Exception.Message +
            "See https://github.com/$Repo/releases, then retry pinned:`n" +
            "  `$env:ARCHI_VERSION = 'x.y.z'; irm https://raw.githubusercontent.com/$Repo/main/release/install.ps1 | iex")
        exit 1
    }
}

$tarball = "archi-$Version-$plat.tar.gz"
$base = "https://github.com/$Repo/releases/download/v$Version"
$tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ("archi-install-" + [Guid]::NewGuid().ToString('N')))
try {
    Write-Host "Downloading $tarball..."
    $dst = Join-Path $tmp.FullName $tarball
    $shaDst = "$dst.sha256"
    Invoke-WebRequest -Uri "$base/$tarball" -OutFile $dst -UseBasicParsing
    Invoke-WebRequest -Uri "$base/$tarball.sha256" -OutFile $shaDst -UseBasicParsing

    $expected = ((Get-Content $shaDst -Raw).Trim() -split '\s+')[0]
    $actual = (Get-FileHash -Path $dst -Algorithm SHA256).Hash
    if ($actual -ne $expected) {
        Write-Error ("Checksum mismatch for {0} — refusing to install.`n  expected {1}`n  actual   {2}" -f `
            $tarball, $expected.ToLower(), $actual.ToLower())
        exit 1
    }

    # tar.exe ships with Windows 10 1803+ / Windows 11 / Server 2019+.
    if (-not (Get-Command tar.exe -ErrorAction SilentlyContinue)) {
        Write-Error "tar.exe not found. Windows 10 1803+ or Windows 11 is required."
        exit 1
    }
    & tar.exe -xzf $dst -C $tmp.FullName
    if ($LASTEXITCODE -ne 0) { Write-Error "tar extraction failed"; exit 1 }

    $src = Join-Path $tmp.FullName "archi-$Version-$plat"
    $binDir = Join-Path $env:USERPROFILE '.local\bin'
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    Copy-Item -Force -Path (Join-Path $src 'archi.exe') -Destination (Join-Path $binDir 'archi.exe')

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($userPath -notlike "*$binDir*") {
        [Environment]::SetEnvironmentVariable('Path', "$binDir;$userPath", 'User')
        Write-Host ""
        Write-Host "Added $binDir to your user PATH — restart the terminal to pick it up."
    }

    Write-Host ""
    Write-Host "Archiplan $Version is installed."
    Write-Host ""
    Write-Host "Next: open your coding agent in a project and run /archi —"
    Write-Host "the agent drives everything from there. 'archi --help' lists the verbs."
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
