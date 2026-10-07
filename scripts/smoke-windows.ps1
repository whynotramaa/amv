param([string]$Installer = 'dist/Harness-Setup-x64.exe', [string]$Out = 'smoke')
# Installs the packaged app like a user would, launches it and records what happened.
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

function Save-Screen([string]$Name) {
    $bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Left, $bounds.Top, 0, 0, $bitmap.Size)
    $bitmap.Save((Join-Path $Out "$Name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose(); $bitmap.Dispose()
}

$started = Get-Date
$install = Start-Process $Installer -ArgumentList '/S' -Wait -PassThru
"installer exit code: $($install.ExitCode)" | Tee-Object (Join-Path $Out 'result.txt')
$exe = Get-ChildItem "$env:LOCALAPPDATA\Harness", "$env:LOCALAPPDATA\Programs\Harness" -Filter 'harness.exe' -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $exe) { throw 'harness.exe was not installed.' }
"installed: $($exe.FullName)" | Tee-Object -Append (Join-Path $Out 'result.txt')

$app = Start-Process $exe.FullName -PassThru
Start-Sleep -Seconds 8
Save-Screen 'launch-8s'
Start-Sleep -Seconds 12
Save-Screen 'launch-20s'
$alive = -not $app.HasExited
"alive after 20s: $alive" | Tee-Object -Append (Join-Path $Out 'result.txt')
if (-not $alive) { "exit code: $($app.ExitCode)" | Tee-Object -Append (Join-Path $Out 'result.txt') }

# Launching again must surface the running instance instead of starting a second one.
if ($alive) {
    Start-Process $exe.FullName | Out-Null
    Start-Sleep -Seconds 5
    Save-Screen 'second-launch'
    "processes after second launch: $(@(Get-Process harness -ErrorAction SilentlyContinue).Count)" | Tee-Object -Append (Join-Path $Out 'result.txt')
}

$data = Join-Path $env:LOCALAPPDATA 'local.harness.desktop'
if (Test-Path $data) {
    Get-ChildItem $data -Recurse -Include *.log, startup-error.txt | Copy-Item -Destination $Out -ErrorAction SilentlyContinue
    Get-ChildItem $data -Recurse | Select-Object FullName, Length | Out-File (Join-Path $Out 'data-files.txt')
}
Get-WinEvent -FilterHashtable @{ LogName = 'Application'; StartTime = $started } -ErrorAction SilentlyContinue |
    Where-Object { $_.Message -match 'harness' } |
    Format-List TimeCreated, ProviderName, Id, Message | Out-File (Join-Path $Out 'events.txt')

Get-Process harness -ErrorAction SilentlyContinue | Stop-Process -Force
if (-not $alive) { throw 'Harness exited during startup. See the smoke artifact.' }
