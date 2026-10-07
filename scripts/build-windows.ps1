$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if (-not $IsWindows -and $PSVersionTable.PSEdition -eq 'Core') { throw 'Run this script on Windows.' }
foreach ($tool in @('node', 'npm', 'cargo', 'cmake')) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { throw "Install the development prerequisite: $tool" }
}
function Invoke-Checked([string]$File, [string[]]$Arguments) {
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$File failed with exit code $LASTEXITCODE" }
}
Invoke-Checked npm.cmd @('ci')
Invoke-Checked npm.cmd @('run', 'prepare:model')
Invoke-Checked npm.cmd @('run', 'build')
Invoke-Checked cargo @('fmt', '--manifest-path', 'src-tauri/Cargo.toml', '--check')
Invoke-Checked cargo @('test', '--manifest-path', 'src-tauri/Cargo.toml', '--no-default-features', '--locked')
Invoke-Checked cargo @('test', '--manifest-path', 'src-tauri/Cargo.toml', '--lib', '--locked')
Invoke-Checked cargo @('clippy', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--all-targets', '--', '-D', 'warnings')
Invoke-Checked npm.cmd @('run', 'tauri', '--', 'build', '--no-bundle', '--target', 'x86_64-pc-windows-msvc')
