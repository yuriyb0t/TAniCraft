#requires -Version 5.1
<#
.SYNOPSIS
    Снимает скриншот окна TAniCraft в PNG.

.DESCRIPTION
    Ищет запущенный процесс tanicraft. Если его нет — запускает exe (по умолчанию
    target\debug\tanicraft.exe относительно корня репозитория), ждёт появления окна,
    выводит его на передний план и копирует область окна в PNG.

    Скрипт нужен для визуальной проверки UI: юнит-тестов на отрисовку нет,
    единственный способ проверить вёрстку — посмотреть на скриншот.

.PARAMETER Exe
    Путь к tanicraft.exe. По умолчанию <repo>\target\debug\tanicraft.exe.

.PARAMETER Out
    Куда сохранить PNG. По умолчанию %TEMP%\opencode\tanicraft.png.

.PARAMETER DelayMs
    Пауза после вывода окна на передний план, мс. По умолчанию 700.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\screenshot.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\screenshot.ps1 -Out shots\ui.png
#>
[CmdletBinding()]
param(
    [string]$Exe = "",
    [string]$Out = "",
    [int]$DelayMs = 700
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

# --- Win32 ---------------------------------------------------------------
if (-not ('TAniCraft.Shot' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

namespace TAniCraft {
    public static class Shot {
        [StructLayout(LayoutKind.Sequential)]
        public struct RECT { public int Left, Top, Right, Bottom; }

        [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
        [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
        [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
        [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT r);
        [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
        [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(
            IntPtr hwnd, int attr, out RECT value, int size);

        public const int SW_RESTORE = 9;
        public const int DWMWA_EXTENDED_FRAME_BOUNDS = 9;

        // Реальные границы окна без невидимой тени-рамки Windows 10/11.
        public static RECT Bounds(IntPtr hwnd) {
            RECT r;
            if (DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, out r,
                                      Marshal.SizeOf(typeof(RECT))) == 0) {
                return r;
            }
            GetWindowRect(hwnd, out r);
            return r;
        }
    }
}
'@
}

Add-Type -AssemblyName System.Drawing
[void][TAniCraft.Shot]::SetProcessDPIAware()

# --- Пути ----------------------------------------------------------------
$repoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($Exe)) {
    $Exe = Join-Path $repoRoot 'target\debug\tanicraft.exe'
}
if ([string]::IsNullOrWhiteSpace($Out)) {
    $outDir = Join-Path $env:TEMP 'opencode'
    $Out = Join-Path $outDir 'tanicraft.png'
}
elseif (-not [IO.Path]::IsPathRooted($Out)) {
    $Out = Join-Path $repoRoot $Out
}
$outDir = Split-Path -Parent $Out
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
}

# --- Процесс -------------------------------------------------------------
function Get-AppWindow {
    # Окно считается найденным, только когда у него уже есть реальный размер:
    # сразу после старта MainWindowHandle может указывать на 16x16 заглушку.
    function Find-Valid {
        Get-Process -Name tanicraft -ErrorAction SilentlyContinue |
            Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
            Where-Object {
                $r = [TAniCraft.Shot]::Bounds($_.MainWindowHandle)
                ($r.Right - $r.Left) -ge 320 -and ($r.Bottom - $r.Top) -ge 240
            } |
            Select-Object -First 1
    }

    $p = Find-Valid
    if ($p) { return $p }

    if (-not (Test-Path $Exe)) {
        throw "tanicraft is not running and the exe was not found: $Exe`nBuild it first: cargo build"
    }
    Write-Host "Launching $Exe ..."
    Start-Process -FilePath $Exe | Out-Null

    $deadline = (Get-Date).AddSeconds(20)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 200
        $p = Find-Valid
        if ($p) { return $p }
    }
    throw "tanicraft window did not appear within 20 seconds."
}

$proc = Get-AppWindow
$hwnd = $proc.MainWindowHandle

if ([TAniCraft.Shot]::IsIconic($hwnd)) {
    [void][TAniCraft.Shot]::ShowWindow($hwnd, [TAniCraft.Shot]::SW_RESTORE)
    Start-Sleep -Milliseconds 250
}
[void][TAniCraft.Shot]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds $DelayMs
$hwnd = (Get-Process -Id $proc.Id).MainWindowHandle

# --- Захват --------------------------------------------------------------
$r = [TAniCraft.Shot]::Bounds($hwnd)
$w = $r.Right - $r.Left
$h = $r.Bottom - $r.Top
if ($w -le 0 -or $h -le 0) {
    throw "Invalid window size: ${w}x${h}"
}

$bmp = New-Object System.Drawing.Bitmap $w, $h
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
try {
    $gfx.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
}
finally {
    $gfx.Dispose()
}

try {
    $bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
}
finally {
    $bmp.Dispose()
}

Write-Output $Out
Write-Host ("{0}x{1}  {2}" -f $w, $h, $Out)
