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

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Platform: Linux](https://img.shields.io/badge/Platform-Linux-blue.svg?style=for-the-badge&logo=linux)](https://ubuntu.com/)
[![Protocol: SOCKS5](https://img.shields.io/badge/Protocol-SOCKS5-green.svg?style=for-the-badge)](https://en.wikipedia.org/wiki/SOCKS)
[![Status: Production](https://img.shields.io/badge/Status-Production-brightgreen.svg?style=for-the-badge)]()

**[English](#-english) | [Bahasa Indonesia](#-bahasa-indonesia)**

</div>

---

<a name="-english"></a>
# 🇬🇧 English

> **"Homemade Mobile Rotating Proxy 🐦‍🔥"**  
> *A high-performance, self-hosted Rust framework that transforms standard Qualcomm 4G LTE USB dongles into enterprise-grade rotating mobile SOCKS5 proxies with hardware-free ADB airplane-mode IP-cycling and failover protection.*

---

### ⚠️ Educational & Research Disclaimer

> **IMPORTANT NOTICE FOR SECURITY AUDITORS, LAW ENFORCEMENT, AND COMPLIANCE OFFICERS:**  
> This software is strictly developed and released for **educational purposes, network engineering research, protocol analysis, and benign homelab testing**. It is designed to evaluate dynamic Carrier-Grade NAT (CGNAT) address allocations, multi-WAN homelab routing isolation, and cellular telemetry behavior in controlled private environments.  
> 
> The author does **not** condone, promote, or encourage any unlawful activities, unauthorized access, denial-of-service, automated credential abuse, spam, fraud, or violation of any telecommunication policies and national laws. The user assumes 100% full legal responsibility and liability for their operation and compliance with local regulations.

---

### 🌟 Key Features

- **Blazing Fast Rust Engine**: Built with `tokio` async runtime and `socket2`, consuming less than **1 MB RAM** with zero-copy bidirectional streaming.
- **Hardware-Free IP Rotation**: Toggles cellular radio via ADB Airplane Mode cycle on Qualcomm MSM8916 dongles. Cellular carriers (XL/Telkomsel/Indosat) assign a **new public CGNAT IP** in 5–15 seconds without physical disconnects.
- **Isolated Egress Policy Routing (Table 200)**: Outbound sockets are hard-bound to the modem interface IP (`192.168.200.174`). Host network traffic (ISP, Tailscale, SSH, Docker) remains untouched on the primary routing table.
- **Failover Safety Guard (Anti-Downtime)**: Automatically monitors host default routes. If primary ISP fails and the server relies on the modem (`opa-isp2lte`), IP rotation is **locked** (`HTTP 423`) to prevent accidental host disconnections (bypassable with `--force`).
- **Sticky Session Ready**: Supports standard commercial proxy session credentials (`socks5://user-session1:pass@127.0.0.1:10800`) and on-demand credential rotation (`user-rotate`).
- **REST Control API (Port 10808)**: Trigger instant rotations via `POST /rotate`, inspect carrier telemetry via `GET /status`, or retrieve current cellular IP via `GET /ip`.
- **Systemd Production Daemon**: Automatically managed 24/7 with zero-downtime restarts.

---

### 🏗️ Architecture & Topology

```mermaid
graph TD
    subgraph Client / Applications
        Bot[Bot / Scraper / Browser]
        Curl[CLI / Scripts]
    end

    subgraph Host Server Linux
        Proxy[OPA-LTE2PROXY SOCKS5 :10800]
        API[Control REST API :10808]
        Guard[Failover Safety Guard]
        Table200[Linux Routing Table 200]
        ADB[Android Debug Bridge Daemon]
    end

    subgraph Hardware & Cellular
        Modem[Qualcomm 4G UFI USB Dongle]
        BTS[Cellular Base Station Tower]
        Internet((Public Internet))
    end

    Bot -->|SOCKS5 Request| Proxy
    Curl -->|POST /rotate| API
    API --> Guard
    Guard -->|Execute Airplane Mode| ADB
    ADB -->|Toggle Radio| Modem
    Proxy -->|Bind Egress 192.168.200.174| Table200
    Table200 -->|Forward Traffic| Modem
    Modem -->|LTE Data| BTS
    BTS -->|Dynamic CGNAT IP| Internet
```

---

### 🚀 Quick Start (English)

#### 1. Installation
Clone and run the automated installer:
```bash
git clone https://github.com/naufalmng/opa-lte2proxy.git /opt/app/opa-lte2proxy
cd /opt/app/opa-lte2proxy
chmod +x install.sh && ./install.sh
```
*The installer automatically compiles the release binary, creates the systemd service, configures Table 200 routing, and links the `olp` CLI shortcut.*

#### 2. CLI Usage (`olp`)
```bash
# Check modem telemetry and proxy connections
olp status

# View current cellular public IP
olp ip

# Trigger immediate IP rotation via Airplane Mode cycle
olp rotate

# Force IP rotation during emergency server failover
olp rotate --force

# Inspect background systemd service status
sudo systemctl status opa-lte2proxy
```

#### 3. Client Configuration
Point any application, antidetect browser, or bot to the local SOCKS5 endpoint:
```text
Host     : 127.0.0.1
Port     : 10800
Protocol : SOCKS5
```

---

### 🌐 REST Control API Reference

| Endpoint | Method | Description | Example Response |
| :--- | :--- | :--- | :--- |
| `/status` | `GET` | Telemetry, carrier info, IP, connections | `{"status":"online","modem":{"operator":"XL","internal_ip":"10.3.13.96"}}` |
| `/ip` | `GET` | Current active cellular IP | `10.3.13.96` |
| `/rotate` | `POST` / `GET` | Trigger Airplane Mode IP cycle | `{"status":"success","old_ip":"10.5.84.210","new_ip":"10.3.13.96","duration_ms":12942}` |

---
---

<a name="-bahasa-indonesia"></a>
# 🇮🇩 Bahasa Indonesia

> **"Mobile Rotating Proxy Rakitan Rumahan 🐦‍🔥"**  
> *Framework mobile proxy SOCKS5 berbasis Rust performa tinggi yang mengubah modem USB 4G LTE Qualcomm menjadi proxy berputar kelas enterprise dengan auto-cycling IP via ADB Airplane Mode dan proteksi failover server.*

---

### ⚠️ Catatan Edukasi & Penelitian Hukum

> **PERHATIAN PENTING BAGI AUDITOR KEAMANAN & APARAT PENEGAK HUKUM:**  
> Software ini dibuat dan didistribusikan murni untuk **tujuan edukasi, riset rekayasa jaringan, analisis protokol komunikasi, serta pengujian infrastruktur homelab pribadi**. Framework ini ditujukan untuk mempelajari alokasi Carrier-Grade NAT (CGNAT) pada jaringan seluler, isolasi policy routing multi-WAN pada kernel Linux, dan telemetri perangkat IoT.  
> 
> Penulis **tidak mendukung, tidak memfasilitasi, dan melarang keras** segala bentuk penyalahgunaan alat ini untuk tindakan melawan hukum, akses tanpa izin, penipuan online, carding, brute force, spamming, atau pelanggaran UU ITE dan regulasi telekomunikasi nasional. Seluruh risiko penggunaan sepenuhnya menjadi tanggung jawab moral dan hukum pengguna.

---

### 🌟 Fitur Unggulan

- **Engine Rust Super Ringan**: Dibangun dengan runtime asinkron `tokio` dan `socket2`, hanya menggunakan memori **< 1 MB RAM** dengan streaming dua arah zero-copy.
- **Ganti IP Tanpa Cabut-Colok Fisik**: Memanfaatkan ADB internal modem Qualcomm MSM8916 untuk memicu cycle Airplane Mode. Operator seluler (XL/Axis/Telkomsel/Indosat/Tri) otomatis merilis IP lama dan memberikan **IP publik baru (CGNAT)** dalam waktu 5–15 detik.
- **Policy Routing Terisolasi (Table 200)**: Seluruh socket proxy di-bind ke interface modem (`192.168.200.174`). Koneksi internet utama server (ISP rumah, Tailscale, SSH, Docker) **tetap 100% aman dan tidak terganggu** di tabel routing utama.
- **Failover Safety Guard (Anti Server Down)**: Mendeteksi rute utama server secara real-time. Jika ISP rumah mati dan server sedang berjalan darurat via modem (`opa-isp2lte`), perintah rotasi IP **otomatis dikunci** (`HTTP 423`) agar koneksi SSH remote tidak mendadak terputus (dapat di-bypass dengan opsi `--force`).
- **Mendukung Sticky Session**: Mendukung kredensial sesi ala proxy komersial (`socks5://user-session1:pass@127.0.0.1:10800`) dan perintah rotasi langsung via kredensial (`user-rotate`).
- **REST Control API (Port 10808)**: Trigger rotasi instan via `POST /rotate`, cek sinyal & operator via `GET /status`, dan intip IP aktif via `GET /ip`.
- **Systemd Daemon 24/7**: Berjalan otomatis di background dan aktif kembali saat server reboot.

---

### 🚀 Panduan Cepat (Bahasa Indonesia)

#### 1. Instalasi Otomatis
Jalankan script installer:
```bash
git clone https://github.com/naufalmng/opa-lte2proxy.git /opt/app/opa-lte2proxy
cd /opt/app/opa-lte2proxy
chmod +x install.sh && ./install.sh
```

#### 2. Perintah CLI (`olp`)
Shortcut CLI **`olp`** tersedia langsung di seluruh terminal server:
```bash
# Cek kondisi modem, operator, sinyal, dan status koneksi proxy
olp status

# Lihat IP seluler yang sedang aktif
olp ip

# Ganti IP seluler baru via Airplane Mode
olp rotate

# Paksa ganti IP meski server sedang dalam mode failover darurat
olp rotate --force

# Cek log service live di background
sudo journalctl -u opa-lte2proxy -f
```

---

### 📂 Struktur Direktori Project

```text
/opt/app/opa-lte2proxy/
├── src/
│   ├── main.rs          # CLI parser, banner ANSI Shadow & subcommands
│   ├── socks5.rs        # SOCKS5 engine (socket2 egress bind + sticky auth)
│   ├── modem.rs         # ADB Airplane mode cycler & Failover Guard
│   └── api.rs           # Axum REST Control API
├── Cargo.toml           # Dependensi Rust (Tokio, Axum, Socket2, Clap)
├── install.sh           # Installer one-liner (Build, symlink, service, routing)
├── run.sh               # Launcher script wrapper
├── opa-lte2proxy.conf   # File konfigurasi utama
├── opa-lte2proxy.service# Systemd service unit
├── LICENSE              # Lisensi Open Source (MIT)
└── README.md            # Dokumentasi lengkap (Bilingual EN/ID)
```

---

### 📜 Lisensi & Kontribusi

Didistribusikan di bawah lisensi terbuka **MIT License**. Lihat file [LICENSE](LICENSE) untuk detail lengkap. Dibuat dan dirancang khusus untuk ekosistem server Bojongkulur.
