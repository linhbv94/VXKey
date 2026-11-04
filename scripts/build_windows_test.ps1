$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'This build requires Windows x64 with Rust stable MSVC and Windows SDK.' }
$repo_dir = Split-Path $PSScriptRoot -Parent
Push-Location (Join-Path $repo_dir 'desktop')
try {
    $env:RUSTUP_TOOLCHAIN = 'stable'
    & npm ci --ignore-scripts
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
    & npm exec -- tauri build
    if ($LASTEXITCODE -ne 0) { throw 'Windows Test build failed' }
    $candidate_dir = Join-Path $repo_dir 'build/windows_candidate'
    New-Item -ItemType Directory -Force $candidate_dir | Out-Null
    $installers = @(Get-ChildItem 'src-tauri/target/release/bundle/nsis' -Filter '*-setup.exe' -File)
    if ($installers.Count -ne 1) { throw 'Expected exactly one candidate installer; inspect stale outputs.' }
    $version = (Get-Content -Raw package.json | ConvertFrom-Json).version.Replace('.', '_')
    $destination = Join-Path $candidate_dir "vxkey_test_${version}_windows_x64_setup.exe"
    Copy-Item -LiteralPath $installers[0].FullName -Destination $destination
    Get-FileHash -LiteralPath $destination -Algorithm SHA256
} finally { Pop-Location }
