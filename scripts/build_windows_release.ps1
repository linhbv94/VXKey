$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'Windows MSVC build host required.' }
if (-not $env:TAURI_SIGNING_PRIVATE_KEY) { throw 'Existing updater signing key is required; no unsigned updater fallback.' }
$repo_dir = Split-Path $PSScriptRoot -Parent
$config = Get-Content -Raw (Join-Path $repo_dir 'desktop/src-tauri/tauri.conf.json') | ConvertFrom-Json
$package = Get-Content -Raw (Join-Path $repo_dir 'desktop/package.json') | ConvertFrom-Json
if ($config.version -ne $package.version) { throw 'Version mismatch' }
if ($env:RELEASE_TAG -and $env:RELEASE_TAG -ne "v$($config.version)") { throw 'Tag/version mismatch' }
Push-Location (Join-Path $repo_dir 'desktop')
try {
    $env:RUSTUP_TOOLCHAIN = 'stable'
    Remove-Item Env:VITE_VXKEY_TEST_APP -ErrorAction SilentlyContinue
    & npm ci --ignore-scripts
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
    & npm exec -- tauri build --features windows-production --config src-tauri/tauri.windows.release.conf.json
    if ($LASTEXITCODE -ne 0) { throw 'Windows release build failed' }
} finally { Pop-Location }
