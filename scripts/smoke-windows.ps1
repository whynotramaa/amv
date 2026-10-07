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

Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class Win {
    public delegate bool EnumProc(IntPtr hwnd, IntPtr param);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc proc, IntPtr param);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr hwnd, int index);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool GetWindowDisplayAffinity(IntPtr hwnd, out uint affinity);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int max);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int max);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    public static IntPtr FindTauri(uint target) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((hwnd, _) => {
            uint pid; GetWindowThreadProcessId(hwnd, out pid);
            var cls = new StringBuilder(256); GetClassName(hwnd, cls, 256);
            if (pid == target && cls.ToString() == "Tauri Window") { found = hwnd; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
    public static List<string> Describe(uint target) {
        var rows = new List<string>();
        EnumWindows((hwnd, _) => {
            uint pid; GetWindowThreadProcessId(hwnd, out pid);
            if (pid != target) return true;
            var title = new StringBuilder(256); GetWindowText(hwnd, title, 256);
            var cls = new StringBuilder(256); GetClassName(hwnd, cls, 256);
            RECT r; GetWindowRect(hwnd, out r);
            uint affinity; GetWindowDisplayAffinity(hwnd, out affinity);
            rows.Add(String.Format("hwnd={0} class={1} title='{2}' visible={3} minimized={4} rect=({5},{6})-({7},{8}) style=0x{9:X8} exstyle=0x{10:X8} affinity={11}",
                hwnd, cls, title, IsWindowVisible(hwnd), IsIconic(hwnd), r.Left, r.Top, r.Right, r.Bottom, GetWindowLong(hwnd, -16), GetWindowLong(hwnd, -20), affinity));
            return true;
        }, IntPtr.Zero);
        return rows;
    }
}
'@

function Save-Windows([string]$Name) {
    $rows = foreach ($process in @(Get-Process harness -ErrorAction SilentlyContinue)) { [Win]::Describe([uint32]$process.Id) }
    "--- windows $Name" | Tee-Object -Append (Join-Path $Out 'result.txt')
    $rows | Where-Object { $_ -match 'Tao|Harness' } | Tee-Object -Append (Join-Path $Out 'result.txt')
}

# UI Automation reads the rendered page even though the window is hidden from capture.
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
function Save-Accessibility([string]$Name) {
    "--- accessibility $Name" | Tee-Object -Append (Join-Path $Out 'result.txt')
    foreach ($process in @(Get-Process harness -ErrorAction SilentlyContinue)) {
        $hwnd = [Win]::FindTauri([uint32]$process.Id)
        if ($hwnd -eq [IntPtr]::Zero) { continue }
        $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
        $queue = New-Object System.Collections.Queue
        $queue.Enqueue(@([System.Windows.Automation.AutomationElement]::FromHandle($hwnd), 0))
        $count = 0
        while ($queue.Count -gt 0 -and $count -lt 120) {
            $element, $depth = $queue.Dequeue()
            $info = $element.Current
            $rect = $info.BoundingRectangle
            $line = ('  ' * [Math]::Min($depth, 12)) + "$($info.ControlType.ProgrammaticName) '$($info.Name)' offscreen=$($info.IsOffscreen) rect=($([int]$rect.X),$([int]$rect.Y),$([int]$rect.Width)x$([int]$rect.Height))"
            $line | Tee-Object -Append (Join-Path $Out 'result.txt')
            $count++
            $child = $walker.GetFirstChild($element)
            while ($child) { $queue.Enqueue(@($child, ($depth + 1))); $child = $walker.GetNextSibling($child) }
        }
    }
}

# Reads the page through WebView2's DevTools port, which content protection does not hide.
function Save-Page([string]$Name) {
    try {
        $targets = Invoke-RestMethod 'http://127.0.0.1:9222/json' -TimeoutSec 5
    } catch {
        "devtools unavailable: $_" | Tee-Object -Append (Join-Path $Out 'result.txt'); return
    }
    foreach ($target in @($targets | Where-Object type -eq 'page')) {
        "--- page $Name $($target.url)" | Tee-Object -Append (Join-Path $Out 'result.txt')
        $socket = New-Object System.Net.WebSockets.ClientWebSocket
        $socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, [Threading.CancellationToken]::None).Wait()
        $script:id = 0
        $call = {
            param($method, $params)
            $script:id++
            $body = @{ id = $script:id; method = $method; params = $params } | ConvertTo-Json -Depth 5 -Compress
            $bytes = [Text.Encoding]::UTF8.GetBytes($body)
            $socket.SendAsync([ArraySegment[byte]]$bytes, 'Text', $true, [Threading.CancellationToken]::None).Wait()
            while ($true) {
                $stream = New-Object IO.MemoryStream
                do {
                    $chunk = New-Object byte[] 65536
                    $result = $socket.ReceiveAsync([ArraySegment[byte]]$chunk, [Threading.CancellationToken]::None).Result
                    $stream.Write($chunk, 0, $result.Count)
                } until ($result.EndOfMessage)
                $reply = [Text.Encoding]::UTF8.GetString($stream.ToArray()) | ConvertFrom-Json
                if ($reply.id -eq $script:id) { return $reply }
            }
        }
        $probe = 'JSON.stringify({ href: location.href, ready: document.readyState, root: (document.getElementById("root") || {}).childElementCount, text: document.body.innerText.slice(0, 300), size: [innerWidth, innerHeight], visibility: document.visibilityState })'
        $value = (& $call 'Runtime.evaluate' @{ expression = $probe; returnByValue = $true }).result.result.value
        "dom: $value" | Tee-Object -Append (Join-Path $Out 'result.txt')
        $shot = (& $call 'Page.captureScreenshot' @{ format = 'png' }).result.data
        if ($shot) { [IO.File]::WriteAllBytes((Join-Path $Out "page-$Name.png"), [Convert]::FromBase64String($shot)) }
        $socket.Dispose()
    }
}

$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
# Tauri passes its own browser arguments, so also set the per-app policy that WebView2 honors.
$policy = 'HKCU:\Software\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments'
New-Item -Force $policy | Out-Null
New-ItemProperty -Force $policy -Name 'harness.exe' -Value '--remote-debugging-port=9222' | Out-Null
$app = Start-Process $exe.FullName -PassThru
Start-Sleep -Seconds 8
Save-Screen 'launch-8s'
Save-Windows 'launch-8s'
Start-Sleep -Seconds 12
Save-Screen 'launch-20s'
Save-Windows 'launch-20s'
Save-Accessibility 'launch-20s'
Save-Page 'launch-20s'
$alive = -not $app.HasExited
"alive after 20s: $alive" | Tee-Object -Append (Join-Path $Out 'result.txt')
if (-not $alive) { "exit code: $($app.ExitCode)" | Tee-Object -Append (Join-Path $Out 'result.txt') }

# Launching again must surface the running instance instead of starting a second one.
if ($alive) {
    Start-Process $exe.FullName | Out-Null
    Start-Sleep -Seconds 5
    Save-Screen 'second-launch'
    Save-Windows 'second-launch'
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

# Echo the evidence into the job log so it can be read without downloading the artifact.
Get-ChildItem $Out -File -Include *.txt, *.log -Recurse | ForEach-Object { "===== $($_.Name)"; Get-Content $_.FullName -Tail 60 }
$codec = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() | Where-Object MimeType -eq 'image/jpeg'
$quality = New-Object System.Drawing.Imaging.EncoderParameters 1
$quality.Param[0] = New-Object System.Drawing.Imaging.EncoderParameter ([System.Drawing.Imaging.Encoder]::Quality), 55L
Get-ChildItem $Out -Filter *.png | ForEach-Object {
    $image = [System.Drawing.Image]::FromFile($_.FullName)
    $scale = [Math]::Min(1.0, 1024 / $image.Width)
    $small = New-Object System.Drawing.Bitmap $image, ([int]($image.Width * $scale)), ([int]($image.Height * $scale))
    $stream = New-Object System.IO.MemoryStream
    $small.Save($stream, $codec, $quality)
    $text = [Convert]::ToBase64String($stream.ToArray())
    "===== BEGIN SCREENSHOT $($_.BaseName)"
    for ($i = 0; $i -lt $text.Length; $i += 4000) { $text.Substring($i, [Math]::Min(4000, $text.Length - $i)) }
    "===== END SCREENSHOT $($_.BaseName)"
    $small.Dispose(); $image.Dispose(); $stream.Dispose()
}

Get-Process harness -ErrorAction SilentlyContinue | Stop-Process -Force
Remove-Item -Force -Recurse 'HKCU:\Software\Policies\Microsoft\Edge\WebView2' -ErrorAction SilentlyContinue
if (-not $alive) { throw 'Harness exited during startup. See the smoke artifact.' }
