#Requires -Version 5
$ErrorActionPreference = 'Stop'

# Archiplan installer, served at https://archiplan.ai/install.ps1 (downloads ride api.archiplan.ai). Downloads
# the archi tarball for Windows x64 and installs archi.exe to %USERPROFILE%\.local\bin.
# Pin a version with $env:ARCHI_VERSION; point at another host with $env:ARCHI_BASE_URL.

$Base = if ($env:ARCHI_BASE_URL) { $env:ARCHI_BASE_URL } else { 'https://api.archiplan.ai' }
$Version = if ($env:ARCHI_VERSION) { $env:ARCHI_VERSION } else { '__INJECT_AT_DEPLOY__' }

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

$tarball = "archi-$Version-$plat.tar.gz"
$tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ("archi-install-" + [Guid]::NewGuid().ToString('N')))
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    Write-Host "Downloading $tarball..."
    $dst = Join-Path $tmp.FullName $tarball
    Invoke-WebRequest -Uri "$Base/download/$tarball" -OutFile $dst -UseBasicParsing

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
