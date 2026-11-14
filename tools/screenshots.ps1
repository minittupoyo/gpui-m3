# README 用のスクリーンショットをギャラリーから撮り直す（Windows / PowerShell 7）。
#
#   cargo build --example gallery
#   pwsh -File tools/screenshots.ps1
#
# ギャラリーを自動で起動・操作して、ウィンドウだけを docs/screenshots/ に保存する。
# 撮影中はマウスとキーボードを触らず、ギャラリーのウィンドウを他のウィンドウで隠さないこと。
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Shoot {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int size);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(int f, int x, int y, int d, int e);
}
"@
$root = Split-Path $PSScriptRoot -Parent
$exe = Join-Path $root "target/debug/examples/gallery.exe"
$out = Join-Path $root "docs/screenshots"
New-Item -ItemType Directory -Force $out | Out-Null

function Start-Gallery($page) {
  Get-Process gallery -ErrorAction SilentlyContinue | Stop-Process -Force
  $env:M3_PAGE = "$page"
  $p = Start-Process -FilePath $exe -PassThru
  Start-Sleep 4
  $p.Refresh()
  [Shoot]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
  Start-Sleep -Milliseconds 600
  return $p
}
function Rect($p) {
  # 影を除いた、見えているウィンドウの範囲（DWMWA_EXTENDED_FRAME_BOUNDS = 9）
  $r = New-Object Shoot+RECT
  [Shoot]::DwmGetWindowAttribute($p.MainWindowHandle, 9, [ref]$r, 16) | Out-Null
  return $r
}
function Snap($p, $name) {
  # ホバー表示やツールチップが写らないよう、いったんウィンドウ内の空白へ動かしてから外へ出す
  $r = Rect $p
  [Shoot]::SetCursorPos($r.L + 900, $r.T + 80) | Out-Null
  Start-Sleep -Milliseconds 700
  [Shoot]::SetCursorPos(5, 5) | Out-Null
  Start-Sleep -Milliseconds 900
  $w = $r.R - $r.L; $h = $r.B - $r.T
  $bmp = New-Object System.Drawing.Bitmap $w, $h
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
  $bmp.Save("$out\$name.png", [System.Drawing.Imaging.ImageFormat]::Png)
  "$name ${w}x${h}"
}
function Wheel($p, $notches) {
  $r = Rect $p
  [Shoot]::SetCursorPos($r.L + 700, $r.T + 500) | Out-Null
  $d = if ($notches -lt 0) { -120 } else { 120 }
  1..[math]::Abs($notches) | ForEach-Object { [Shoot]::mouse_event(0x800, 0, 0, $d, 0); Start-Sleep -Milliseconds 15 }
  Start-Sleep -Milliseconds 700
}
function Click($p, $x, $y) {
  $r = Rect $p
  [Shoot]::SetCursorPos($r.L + $x, $r.T + $y) | Out-Null
  Start-Sleep -Milliseconds 150
  [Shoot]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 60; [Shoot]::mouse_event(4, 0, 0, 0, 0)
  Start-Sleep -Milliseconds 900
}

# 0 Buttons / 1 Selection / 2 Chips & Cards / 6 Text fields / 7 Menus / 9 Color
foreach ($pair in @(@(0, "buttons"), @(1, "selection"), @(2, "chips-cards"), @(6, "text-fields"), @(9, "color"))) {
  $p = Start-Gallery $pair[0]
  Snap $p $pair[1]
}

# More: carousel
$p = Start-Gallery 10
Wheel $p -60; Wheel $p 19
Snap $p "carousel"
# More: pickers
$p = Start-Gallery 10
Wheel $p -60; Wheel $p 6
Snap $p "pickers"

# Light theme (sun icon at the bottom of the rail)
$p = Start-Gallery 0
$r = Rect $p
Click $p 47 ($r.B - $r.T - 36)
Snap $p "buttons-light"

# Menu with a submenu
$p = Start-Gallery 7
Click $p 196 270
Click $p 230 465
Snap $p "menu-submenu"

Get-Process gallery -ErrorAction SilentlyContinue | Stop-Process -Force
