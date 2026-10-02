# ==============================================================================
# OPA-LTE2PROXY Windows Installer & Setup Script (PowerShell)
# One-liner: irm https://raw.githubusercontent.com/naufalmng/opa-lte2proxy/main/install.ps1 | iex
# ==============================================================================

Write-Host @"
 ██████╗ ██████╗  █████╗       ██╗  ████████╗███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
██╔═══██╗██╔══██╗██╔══██╗      ██║  ╚══██╔══╝██╔════╝╚════██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
██║   ██║██████╔╝███████║█████╗██║     ██║   █████╗   █████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
██║   ██║██╔═══╝ ██╔══██║╚════╝██║     ██║   ██╔══╝  ██╔═══╝ ██╔═══╝ ██║   ██║ ██╔██╗   ╚██╔╝  
╚██████╔╝██║     ██║  ██║      ███████╗██║   ███████╗███████╗██║     ╚██████╔╝██╔╝ ██╗   ██║   
 ╚═════╝ ╚═╝     ╚═╝  ╚═╝      ╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   
"@ -ForegroundColor Cyan

Write-Host "🚀 OPA-LTE2PROXY Installer for Windows" -ForegroundColor Green
Write-Host "Tagline: Homemade Mobile Rotating Proxy 🐦‍🔥" -ForegroundColor Yellow
Write-Host "Catatan: Software ini dibuat untuk riset dan kebutuhan pribadi. Segala bentuk penggunaan di luar tanggung jawab pengembang.`n" -ForegroundColor DarkGray

# 1. Administrator Check
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warning "Installer ini membutuhkan hak akses Administrator. Harap jalankan PowerShell as Administrator!"
    Exit 1
}

$InstallDir = "$env:ProgramData\opa-lte2proxy"
$BinDest = "$InstallDir\opa-lte2proxy.exe"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# 2. Check ADB
Write-Host "[1/3] Memeriksa Android Debug Bridge (ADB)..." -ForegroundColor Cyan
if (Get-Command "adb" -ErrorAction SilentlyContinue) {
    Write-Host "  [✓] ADB terdeteksi." -ForegroundColor Green
} else {
    Write-Warning "  [!] ADB belum terpasang di PATH Windows."
    Write-Host "      Download Android SDK Platform-Tools atau pasang via winget/scoop:" -ForegroundColor Gray
    Write-Host "      winget install Google.PlatformTools  (atau scoop install adb)" -ForegroundColor Gray
}

# 3. Check or Build Binary
Write-Host "[2/3] Memasang binary opa-lte2proxy..." -ForegroundColor Cyan
if (Test-Path "target\release\opa-lte2proxy.exe") {
    Copy-Item "target\release\opa-lte2proxy.exe" -Destination $BinDest -Force
    Write-Host "  [✓] Binary lokal disalin ke $BinDest" -ForegroundColor Green
} elseif (Get-Command "cargo" -ErrorAction SilentlyContinue) {
    Write-Host "  [*] Mem-build binary dari source..." -ForegroundColor Yellow
    cargo build --release
    Copy-Item "target\release\opa-lte2proxy.exe" -Destination $BinDest -Force
} else {
    Write-Host "  [*] Mengunduh pre-built binary release..." -ForegroundColor Yellow
    $DownloadUrl = "https://github.com/naufalmng/opa-lte2proxy/releases/latest/download/opa-lte2proxy-windows-amd64.exe"
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $BinDest -UseBasicParsing
        Write-Host "  [✓] Berhasil diunduh ke $BinDest" -ForegroundColor Green
    } catch {
        Write-Warning "  [!] Gagal mengunduh binary otomatis. Silakan build manual via 'cargo build --release'."
    }
}

# Add to System PATH if not present
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "Machine")
    $env:Path += ";$InstallDir"
    Write-Host "  [✓] Ditambahkan ke PATH sistem." -ForegroundColor Green
}

# 4. Interactive Configuration Wizard
Write-Host "`n[3/3] Menjalankan Setup Interaktif..." -ForegroundColor Cyan
if (Test-Path $BinDest) {
    & $BinDest setup
}

Write-Host @"
================================================================
 🎉 INSTALASI WINDOWS SELESAI!
================================================================
SOCKS5 Proxy     : 127.0.0.1:10800
REST Control API : http://127.0.0.1:10808
Perintah CLI:
  • opa-lte2proxy status
  • opa-lte2proxy rotate
  • opa-lte2proxy ip
================================================================
"@ -ForegroundColor Green
