param([switch]$SkipChecks)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if (-not $IsWindows -and $PSVersionTable.PSEdition -eq 'Core') { throw 'Run this script on Windows.' }
if (-not $SkipChecks) { & "$PSScriptRoot/build-windows.ps1" }
& npm.cmd run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis
if ($LASTEXITCODE -ne 0) { throw 'Windows packaging failed.' }
$installer = Get-ChildItem 'src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*-setup.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $installer) { throw 'NSIS installer was not produced.' }
New-Item -ItemType Directory -Force 'dist' | Out-Null
Copy-Item $installer.FullName 'dist/Harness-Setup-x64.exe' -Force
$hash = (Get-FileHash 'dist/Harness-Setup-x64.exe' -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  Harness-Setup-x64.exe" | Set-Content 'dist/Harness-Setup-x64.exe.sha256' -Encoding ascii
Write-Host 'Created dist/Harness-Setup-x64.exe'
