<div align="center">

```text
 ██████╗ ██████╗  █████╗       ██╗  ████████╗███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
██╔═══██╗██╔══██╗██╔══██╗      ██║  ╚══██╔══╝██╔════╝╚════██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
██║   ██║██████╔╝███████║█████╗██║     ██║   █████╗   █████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
██║   ██║██╔═══╝ ██╔══██║╚════╝██║     ██║   ██╔══╝  ██╔═══╝ ██╔═══╝ ██║   ██║ ██╔██╗   ╚██╔╝  
╚██████╔╝██║     ██║  ██║      ███████╗██║   ███████╗███████╗██║     ╚██████╔╝██╔╝ ██╗   ██║   
 ╚═════╝ ╚═╝     ╚═╝  ╚═╝      ╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   
```

# OPA-LTE2PROXY

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg?style=for-the-badge)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Platform: Linux | macOS | Windows](https://img.shields.io/badge/Platform-Linux%20|%20macOS%20|%20Windows-lightgrey.svg?style=for-the-badge)](https://github.com/naufalmng/opa-lte2proxy)
[![Protocol: SOCKS5](https://img.shields.io/badge/Protocol-SOCKS5-green.svg?style=for-the-badge)](https://en.wikipedia.org/wiki/SOCKS)
[![E2E Tests: Passed](https://img.shields.io/badge/E2E%20Tests-Passed-brightgreen.svg?style=for-the-badge)]()

**[English](#-english) | [Bahasa Indonesia](#-bahasa-indonesia)**

</div>

---

<a name="-english"></a>
# 🇬🇧 English

> **"Homemade Mobile Rotating Proxy 🐦‍🔥"**  
> *A high-performance Rust framework that turns standard 4G LTE USB modems into rotating mobile SOCKS5 proxies with hardware-free ADB airplane mode IP-cycling and failover protection.*

### 📝 Note
> This software is intended for research, network experimentation, and personal homelab environments. The author assumes no liability or responsibility for how this tool is used or any risks arising from its operation.

---

### 📦 Hardware & System Requirements

#### 1. Supported Modems
- **Tested & Verified:** **Qualcomm Snapdragon MSM8916 / 4G UFI USB Dongle** (Hardware ID: `05c6:9024`, Android 4.4/5.1 OS onboard with exposed ADB).
- **Compatible:** Qualcomm MDM9600/MDM9200 series, Huawei HiLink (E3372/E8372 with ADB/web API enabled), ZTE MF79U/MF833.
- **SIM Card:** Any cellular SIM with an active data package (XL Axiata, Telkomsel, Indosat, Tri, Smartfren, etc.).

#### 2. Software Requirements
- **ADB (Android Debug Bridge)**:
  - Linux: `sudo apt install adb`
  - macOS: `brew install android-platform-tools`
  - Windows: `winget install Google.PlatformTools` (or `scoop install adb`)
- **OS Supported**: Linux (Ubuntu 20.04+, Debian, Arch), Windows 10/11, macOS 12+.

---

### ⚡ One-Liner Quick Installation

#### 🐧 Linux & 🍏 macOS:
```bash
curl -fsSL https://raw.githubusercontent.com/naufalmng/opa-lte2proxy/main/install.sh | sudo bash
```

#### 🪟 Windows (PowerShell as Administrator):
```powershell
irm https://raw.githubusercontent.com/naufalmng/opa-lte2proxy/main/install.ps1 | iex
```

---

### 🔧 Interactive Proactive Setup Wizard (`olp setup`)

Need to reconfigure interfaces or scan connected modems? Just run:
```bash
olp setup
```
The wizard will:
1. **Auto-scan network interfaces** and highlight the modem interface (`192.168.200.x` / LTE).
2. **Auto-detect ADB devices** (detects Qualcomm serial numbers like `2285100c`).
3. **Prompt for ports** (SOCKS5 `:10800`, REST API `:10808`, auto-rotation timer).
4. **Run live hardware & signal verification** (operator detection and response test).
5. Save settings to `/etc/opa-lte2proxy.conf` and restart the service.

---

### 🏗️ Architecture & Topology

```mermaid
graph TD
    subgraph Clients
        Bot[Automation Bot / Browser]
        Curl[CLI / Script / olp]
    end

    subgraph Host Server
        Proxy[OPA-LTE2PROXY SOCKS5 :10800]
        API[Control REST API :10808]
        Guard[Failover Safety Guard]
        Table200[Isolated Routing Table 200]
        ADB[Android Debug Bridge Daemon]
    end

    subgraph Cellular Hardware
        Modem[Qualcomm 4G UFI USB Dongle]
        BTS[Cellular Base Station Tower]
        Internet((Public Internet))
    end

    Bot -->|SOCKS5 Request| Proxy
    Curl -->|POST /rotate| API
    API --> Guard
    Guard -->|Airplane Mode Cycle| ADB
    ADB -->|Toggle Radio| Modem
    Proxy -->|Bind Egress 192.168.200.174| Table200
    Table200 -->|Forward Outbound| Modem
    Modem -->|LTE Connection| BTS
    BTS -->|New CGNAT Public IP| Internet
```

---

### 💻 CLI Reference (`olp`)

| Command | Action |
| :--- | :--- |
| `olp status` | View modem status, cellular operator, signal, and active connections |
| `olp ip` | Print current active cellular public IP |
| `olp rotate` | Trigger Airplane Mode IP cycle (takes ~5–15 seconds) |
| `olp rotate --force` | Force IP rotation even if the server is in emergency LTE failover |
| `olp setup` | Launch the interactive proactive hardware & network setup wizard |

---

### 🔄 Sticky Sessions & Credentials

Pass standard proxy credentials to lock sessions or trigger rotations directly:
```text
# Sticky session per account:
socks5://user-session_123:mypassword@127.0.0.1:10800

# Trigger immediate IP rotation via credentials:
socks5://user-rotate:mypassword@127.0.0.1:10800
```

---
---

<a name="-bahasa-indonesia"></a>
# 🇮🇩 Bahasa Indonesia

> **"Mobile Rotating Proxy Rakitan Rumahan 🐦‍🔥"**  
> *Framework proxy seluler SOCKS5 berbasis Rust performa tinggi yang mengubah modem USB 4G LTE Qualcomm menjadi proxy berputar kelas enterprise dengan auto-cycling IP via ADB Airplane Mode dan proteksi failover server.*

### 📝 Catatan
> Software ini dibuat untuk riset, eksperimen jaringan, dan kebutuhan homelab pribadi. Segala bentuk penggunaan dan risiko operasional sepenuhnya berada di luar tanggung jawab pengembang.

---

### 📦 Kebutuhan Perangkat & Sistem

#### 1. Modem yang Didukung
- **Sudah Teruji & Terbukti:** **Qualcomm Snapdragon MSM8916 / 4G UFI USB Dongle** (Hardware ID: `05c6:9024`, OS Android 4.4/5.1 internal dengan ADB aktif).
- **Kompatibel:** Seri Qualcomm MDM9600/MDM9200, Huawei HiLink (E3372/E8372 mode ADB/API), ZTE MF79U/MF833.
- **Kartu SIM:** Kartu seluler apa saja dengan paket data aktif (XL, Telkomsel, Indosat, Tri, Smartfren).

#### 2. Kebutuhan Software
- **ADB (Android Debug Bridge)**:
  - Linux: `sudo apt install adb`
  - macOS: `brew install android-platform-tools`
  - Windows: `winget install Google.PlatformTools` (atau `scoop install adb`)
- **OS yang Didukung**: Linux (Ubuntu, Debian, dsb), Windows 10/11, macOS 12+.

---

### ⚡ Instalasi Instan (One-Liner)

#### 🐧 Linux & 🍏 macOS:
```bash
curl -fsSL https://raw.githubusercontent.com/naufalmng/opa-lte2proxy/main/install.sh | sudo bash
```

#### 🪟 Windows (Buka PowerShell as Administrator):
```powershell
irm https://raw.githubusercontent.com/naufalmng/opa-lte2proxy/main/install.ps1 | iex
```

---

### 🔧 Wizard Setup Interaktif Proaktif (`olp setup`)

Mau atur ulang interface atau baru colok modem baru? Cukup jalankan:
```bash
olp setup
```
Wizard akan secara proaktif:
1. **Memindai seluruh interface jaringan** dan otomatis mendeteksi interface modem LTE (`192.168.200.x`).
2. **Memindai perangkat ADB** (otomatis mendeteksi serial Qualcomm seperti `2285100c`).
3. **Menanyakan parameter konfigurasi** dengan nilai default yang tinggal ditekan ENTER.
4. **Menjalankan uji koneksi live** (cek operator dan respon sinyal modem).
5. Menyimpan ke `/etc/opa-lte2proxy.conf` dan me-restart service secara otomatis.

---

### 🧪 Pengujian End-to-End (E2E Test)

Framework dilengkapi suite pengujian otomatis:
```bash
/opt/app/opa-lte2proxy/tests/e2e_test.sh
```
*Menguji 8 titik kritis: validitas binary, status service, handshake SOCKS5 direct, autentikasi sticky session RFC 1929, REST API `/status` & `/ip`, trigger `/rotate`, serta isolasi Table 200.*

---

### 📜 Lisensi & Kontribusi

Didistribusikan di bawah lisensi terbuka **GNU General Public License v3.0 (GPL-3.0)**. Lihat file [LICENSE](LICENSE) untuk ketentuan lengkap.
