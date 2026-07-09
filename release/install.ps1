#Requires -Version 5
$ErrorActionPreference = 'Stop'

# Archiplan installer. Downloads the archi tarball for this platform from
# GitHub Releases and installs it to %USERPROFILE%\.local\bin. Pin a version
# with $env:ARCHI_VERSION; point at another asset host with $env:ARCHI_BASE_URL.

$Repo = 'archiplan-ai/Archiplan'
$Base = if ($env:ARCHI_BASE_URL) { $env:ARCHI_BASE_URL } else { "https://github.com/$Repo/releases/download" }

$arch = $env:PROCESSOR_ARCHITECTURE
if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
switch ($arch) {
    'AMD64' { $plat = 'windows-x64' }
    'x86_64' { $plat = 'windows-x64' }
    default {
        Write-Error "Unsupported platform: Windows-$arch. Supported: Windows x86_64."
        exit 1
    }
}

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Version = $env:ARCHI_VERSION
if (-not $Version) {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
    $Version = $release.tag_name -replace '^v', ''
    if (-not $Version) {
        Write-Error "Could not resolve the latest release. Set `$env:ARCHI_VERSION and retry."
        exit 1
    }
}

$tarball = "archi-$Version-$plat.tar.gz"
$tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ("archi-install-" + [Guid]::NewGuid().ToString('N')))
try {
    Write-Host "Downloading $tarball..."
    $dst = Join-Path $tmp.FullName $tarball
    Invoke-WebRequest -Uri "$Base/v$Version/$tarball" -OutFile $dst -UseBasicParsing
    Invoke-WebRequest -Uri "$Base/v$Version/$tarball.sha256" -OutFile "$dst.sha256" -UseBasicParsing

    $expected = ((Get-Content "$dst.sha256" -TotalCount 1) -split '\s+')[0]
    $actual = (Get-FileHash -Algorithm SHA256 -Path $dst).Hash
    if ($expected -ne $actual) {
        Write-Error "Checksum mismatch for $tarball — aborting."
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

    # Add $binDir to User PATH (persistent) and current process PATH (so
    # `archi` works in the shell we were just launched from). Case-insensitive
    # contains check on a semicolon-padded form to avoid partial-prefix matches.
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not ($userPath)) { $userPath = '' }
    if (-not ";$userPath;".ToLower().Contains(";$($binDir.ToLower());")) {
        $newPath = if ($userPath) { "$userPath;$binDir" } else { $binDir }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Host "Added $binDir to your User PATH."
    }
    if (-not ";$env:Path;".ToLower().Contains(";$($binDir.ToLower());")) {
        $env:Path = "$env:Path;$binDir"
    }

    Write-Host ""
    Write-Host "Archiplan $Version is installed."
    Write-Host ""
    Write-Host "Get started: run 'archi init' in a project directory."
}
finally {
    Remove-Item -Recurse -Force $tmp.FullName -ErrorAction SilentlyContinue
}
