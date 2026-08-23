[CmdletBinding()]
param(
    [switch]$SkipTests,
    [switch]$InstallDependencies
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$manifestPath = Join-Path $projectRoot 'src-tauri\Cargo.toml'
$tauriConfigPath = Join-Path $projectRoot 'src-tauri\tauri.conf.json'
$packageJsonPath = Join-Path $projectRoot 'package.json'
$versionPath = Join-Path $projectRoot 'VERSION'
$packageLockPath = Join-Path $projectRoot 'package-lock.json'
$cargoLockPath = Join-Path $projectRoot 'src-tauri\Cargo.lock'

function Write-Step {
    param([Parameter(Mandatory)][string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Require-Command {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$InstallHint
    )

    $command = Get-Command $Name -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $command) {
        throw "Required command '$Name' was not found. $InstallHint"
    }
    return $command.Source
}

function Read-JsonFile {
    param([Parameter(Mandatory)][string]$Path)

    try {
        return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    }
    catch {
        throw "Unable to parse JSON file '$Path': $($_.Exception.Message)"
    }
}

function Assert-VersionConsistency {
    Write-Step 'Checking release version consistency'

    $releaseVersion = (Get-Content -LiteralPath $versionPath -Raw).Trim()
    if ($releaseVersion -notmatch '^\d+\.\d+\.\d+$') {
        throw "VERSION must contain a semantic version such as 1.0.0; found '$releaseVersion'."
    }

    $package = Read-JsonFile -Path $packageJsonPath
    $packageLockText = Get-Content -LiteralPath $packageLockPath -Raw
    $tauriConfig = Read-JsonFile -Path $tauriConfigPath
    $cargoManifestVersion = (Select-String -LiteralPath $manifestPath -Pattern '^version\s*=\s*"([^"]+)"' |
        Select-Object -First 1).Matches.Groups[1].Value
    $cargoLockVersion = (Select-String -LiteralPath $cargoLockPath -Pattern '^name\s*=\s*"my-codex"\s*$' -Context 0,1 |
        Select-Object -First 1).Context.PostContext |
        Select-String -Pattern '^version\s*=\s*"([^"]+)"' |
        Select-Object -First 1
    if ($cargoLockVersion) {
        $cargoLockVersion = $cargoLockVersion.Matches.Groups[1].Value
    }

    $packageLockVersion = [regex]::Match(
        $packageLockText,
        '(?s)\A\{\s*"name"\s*:\s*"my-codex"\s*,\s*"version"\s*:\s*"([^"]+)"'
    ).Groups[1].Value
    $packageLockRootVersion = [regex]::Match(
        $packageLockText,
        '(?s)"packages"\s*:\s*\{\s*""\s*:\s*\{\s*"name"\s*:\s*"my-codex"\s*,\s*"version"\s*:\s*"([^"]+)"'
    ).Groups[1].Value
    if (-not $packageLockVersion -or -not $packageLockRootVersion) {
        throw "Unable to read the root package versions from '$packageLockPath'."
    }

    $declaredVersions = [ordered]@{
        'VERSION' = $releaseVersion
        'package.json' = [string]$package.version
        'package-lock.json' = $packageLockVersion
        'package-lock root package' = $packageLockRootVersion
        'src-tauri/tauri.conf.json' = [string]$tauriConfig.version
        'src-tauri/Cargo.toml' = $cargoManifestVersion
        'src-tauri/Cargo.lock' = $cargoLockVersion
    }

    foreach ($entry in $declaredVersions.GetEnumerator()) {
        if ($entry.Value -ne $releaseVersion) {
            throw "Version mismatch: $($entry.Key) declares '$($entry.Value)', expected '$releaseVersion'."
        }
    }

    Write-Host "Release version: $releaseVersion" -ForegroundColor Green
    return $releaseVersion
}

function Import-VsDeveloperEnvironment {
    if ((Get-Command cl.exe -ErrorAction SilentlyContinue) -and
        (Get-Command link.exe -ErrorAction SilentlyContinue)) {
        return
    }

    $candidateVsWhere = @()
    if (${env:ProgramFiles(x86)}) {
        $candidateVsWhere += Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    }
    if ($env:ProgramFiles) {
        $candidateVsWhere += Join-Path $env:ProgramFiles 'Microsoft Visual Studio\Installer\vswhere.exe'
    }
    $candidateVsWhere = @($candidateVsWhere | Where-Object { Test-Path -LiteralPath $_ })

    if (-not $candidateVsWhere) {
        throw 'Visual Studio Installer/vswhere.exe was not found. Install Visual Studio 2022 Build Tools with Desktop development with C++ and a Windows 10/11 SDK.'
    }

    $vsWhere = $candidateVsWhere[0]
    # Quote the wildcard so PowerShell does not expand it into workspace paths
    # before invoking vswhere.
    $installationPath = & $vsWhere -latest -products '*' `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath

    if ($installationPath) {
        $installationPath = ($installationPath | Select-Object -First 1).ToString().Trim()
    }

    if (-not $installationPath) {
        throw 'Visual C++ x64 build tools were not found. Install the Desktop development with C++ workload.'
    }

    $vsDevCmd = Join-Path $installationPath 'Common7\Tools\VsDevCmd.bat'
    if (-not (Test-Path -LiteralPath $vsDevCmd)) {
        throw "Visual Studio developer environment script was not found: $vsDevCmd"
    }

    Write-Step 'Loading the Visual Studio x64 developer environment'
    $environmentLines = & $env:ComSpec /d /s /c "`"$vsDevCmd`" -no_logo -arch=amd64 -host_arch=amd64 && set"
    if ($LASTEXITCODE -ne 0) {
        throw "VsDevCmd.bat failed with exit code $LASTEXITCODE."
    }

    foreach ($line in $environmentLines) {
        $separator = $line.IndexOf('=')
        if ($separator -le 0) { continue }
        $name = $line.Substring(0, $separator)
        $value = $line.Substring($separator + 1)
        Set-Item -Path "Env:$name" -Value $value
    }

    Require-Command 'cl.exe' 'Confirm that the MSVC x64 tools are installed.' | Out-Null
    Require-Command 'link.exe' 'Confirm that the MSVC x64 linker is installed.' | Out-Null
}

if ([System.Environment]::OSVersion.Platform -ne [System.PlatformID]::Win32NT) {
    throw 'This script only supports Windows. Use the platform-specific Tauri build flow on macOS or Linux.'
}

if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "Rust manifest was not found: $manifestPath"
}
if (-not (Test-Path -LiteralPath $packageJsonPath)) {
    throw "package.json was not found: $packageJsonPath"
}
if (-not (Test-Path -LiteralPath $tauriConfigPath)) {
    throw "Tauri configuration was not found: $tauriConfigPath"
}
if (-not (Test-Path -LiteralPath $versionPath)) {
    throw "VERSION was not found: $versionPath"
}
if (-not (Test-Path -LiteralPath $packageLockPath)) {
    throw "package-lock.json was not found: $packageLockPath"
}
if (-not (Test-Path -LiteralPath $cargoLockPath)) {
    throw "src-tauri/Cargo.lock was not found: $cargoLockPath"
}

$releaseVersion = Assert-VersionConsistency

Write-Step 'Checking Node.js, npm, and Rust prerequisites'
Require-Command 'node.exe' 'Install Node.js 20 or later.' | Out-Null
Require-Command 'npm.cmd' 'Install Node.js with npm.' | Out-Null
Require-Command 'rustc.exe' 'Install stable-x86_64-pc-windows-msvc from https://rustup.rs.' | Out-Null
Require-Command 'cargo.exe' 'Install Rust and Cargo from https://rustup.rs.' | Out-Null
Import-VsDeveloperEnvironment

Push-Location $projectRoot
try {
    if ($InstallDependencies -or -not (Test-Path -LiteralPath (Join-Path $projectRoot 'node_modules'))) {
        Write-Step 'Installing frontend dependencies from package-lock.json'
        & npm.cmd ci
        if ($LASTEXITCODE -ne 0) { throw "npm ci failed with exit code $LASTEXITCODE." }
    }

    Write-Step 'Running TypeScript checks'
    & npm.cmd run typecheck
    if ($LASTEXITCODE -ne 0) { throw "Typecheck failed with exit code $LASTEXITCODE." }

    if (-not $SkipTests) {
        Write-Step 'Running frontend tests'
        & npm.cmd run test
        if ($LASTEXITCODE -ne 0) { throw "Frontend tests failed with exit code $LASTEXITCODE." }
    }

    Write-Step 'Running Rust compile checks'
    & cargo.exe check --locked --manifest-path $manifestPath
    if ($LASTEXITCODE -ne 0) { throw "cargo check failed with exit code $LASTEXITCODE." }

    if (-not $SkipTests) {
        Write-Step 'Running Rust tests'
        & cargo.exe test --locked --manifest-path $manifestPath
        if ($LASTEXITCODE -ne 0) { throw "cargo test failed with exit code $LASTEXITCODE." }
    }

    Write-Step "Building My Codex $releaseVersion and Windows installers"
    & npm.cmd run tauri:build
    if ($LASTEXITCODE -ne 0) { throw "Tauri build failed with exit code $LASTEXITCODE." }

    $releaseRoot = Join-Path $projectRoot 'src-tauri\target\release'
    $expectedExecutable = Join-Path $releaseRoot 'my-codex.exe'
    $artifacts = @()

    if (Test-Path -LiteralPath $expectedExecutable) {
        $artifacts += Get-Item -LiteralPath $expectedExecutable
    }

    $bundleRoot = Join-Path $releaseRoot 'bundle'
    if (Test-Path -LiteralPath $bundleRoot) {
        $artifacts += Get-ChildItem -LiteralPath $bundleRoot -Recurse -File |
            Where-Object { $_.Extension -in @('.exe', '.msi') }
    }

    $versionToken = [regex]::Escape($releaseVersion)
    $installerArtifacts = @($artifacts | Where-Object {
        $_.Extension -eq '.exe' -and
        $_.FullName -match '\\bundle\\nsis\\' -and
        $_.Name -match "(^|_)$versionToken(_|-)"
    })

    if (-not (Test-Path -LiteralPath $expectedExecutable)) {
        throw "The Tauri build finished, but the application EXE was not found: $expectedExecutable"
    }
    if (-not $installerArtifacts) {
        throw "The Tauri build finished, but no NSIS installer EXE was found under $bundleRoot."
    }

    Write-Step "Build complete: deliverable EXE artifacts for $releaseVersion"
    @($expectedExecutable) + $installerArtifacts |
        ForEach-Object {
            if ($_ -is [string]) { Get-Item -LiteralPath $_ } else { $_ }
        } |
        Sort-Object FullName -Unique |
        ForEach-Object {
            $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash
            Write-Host ("{0}`n  Size: {1:N0} bytes`n  SHA256: {2}" -f $_.FullName, $_.Length, $hash) -ForegroundColor Green
        }

    Write-Host "`nNote: a successful local build does not prove Authenticode signing, SmartScreen reputation, or clean-machine installation acceptance." -ForegroundColor Yellow
}
finally {
    Pop-Location
}
