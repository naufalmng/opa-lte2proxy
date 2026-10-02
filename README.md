# 🚀 OPA-LTE2PROXY

```text
 ██████╗ ██████╗  █████╗       ██╗  ████████╗███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
██╔═══██╗██╔══██╗██╔══██╗      ██║  ╚══██╔══╝██╔════╝╚════██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
██║   ██║██████╔╝███████║█████╗██║     ██║   █████╗   █████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
██║   ██║██╔═══╝ ██╔══██║╚════╝██║     ██║   ██╔══╝  ██╔═══╝ ██╔═══╝ ██║   ██║ ██╔██╗   ╚██╔╝  
╚██████╔╝██║     ██║  ██║      ███████╗██║   ███████╗███████╗██║     ╚██████╔╝██╔╝ ██╗   ██║   
 ╚═════╝ ╚═╝     ╚═╝  ╚═╝      ╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   
```

> **"Opa jagain proxy lo, gonta-ganti IP tanpa cabut colok."**  
> *Framework Mobile Proxy SOCKS5 Berperforma Tinggi (Rust) dengan Auto IP-Cycling via ADB Airplane Mode & Failover Safety Guard.*

---

## 📑 Daftar Isi / Table of Contents
- [Tentang OPA-LTE2PROXY](#-tentang-opa-lte2proxy)
- [Arsitektur & Failover Guard](#-arsitektur--failover-guard)
- [Fitur Utama](#-fitur-utama)
- [Cara Install & Deploy 24/7](#-cara-install--deploy-247)
- [Perintah CLI (`olp`)](#-perintah-cli-olp)
- [Cara Pakai Sticky Session](#-cara-pakai-sticky-session)
- [REST Control API (Port 10808)](#-rest-control-api-port-10808)
- [Integrasi Bot Farming (`ag29router`)](#-integrasi-bot-farming-ag29router)
- [Troubleshooting & Log](#-troubleshooting--log)

---

## 🌟 Tentang OPA-LTE2PROXY

**OPA-LTE2PROXY** adalah framework self-hosted yang mengubah modem USB 4G LTE fisik (Qualcomm UFI / Android MSM8916) menjadi **Rotating Mobile SOCKS5 Proxy** kelas enterprise. 

Tidak perlu hardware switcher USB mahal atau repot cabut-colok modem fisik secara manual. Engine ini memanfaatkan interface **ADB (Android Debug Bridge)** internal modem untuk mentrigger **Airplane Mode cycle**, memaksa Base Transceiver Station (BTS) operator seluler (XL, Telkomsel, Indosat, Tri) merilis IP lama dan mengalokasikan **IP publik baru (CGNAT)** dalam waktu 5–15 detik.

Ditulis dalam bahasa **Rust (Tokio async + Socket2)** dengan konsumsi memori kurang dari **1 MB RAM** dan performa zero-copy streaming.

---

## 🛡️ Arsitektur & Failover Guard

### 1. Egress Policy Routing Terisolasi (Table 200)
Proxy mengunci seluruh koneksi outbound ke IP interface modem (`192.168.200.174`). Linux kernel meneruskan paket via Table 200 ke modem gateway `192.168.200.1`:
- 100% traffic normal host (ISP rumah, Tailscale, SSH, Docker, background service) **tetap berjalan di Table main**.
- Proxy dan failover server tidak saling mengganggu.

### 2. Failover Safety Guard (Anti Server Down)
Jika ISP rumah mati dan script failover server (`opa-isp2lte`) mengalihkan koneksi host ke modem LTE:
- `OPA-LTE2PROXY` mendeteksi bahwa default route server sedang bertumpu pada modem.
- Endpoint `/rotate` **otomatis terkunci** dan menolak rotasi IP agar koneksi SSH / remote server tidak mendadak terputus.
- Jika pengguna tetap ingin merotasi, dapat mem-bypass dengan parameter `--force` atau `?force=true`.

---

## ⚡ Fitur Utama

- **Ultra-Lightweight Rust Engine**: Footprint RAM < 1 MB, CPU idle 0%, start dalam hitungan milidetik.
- **Hardware-Free IP Rotation**: Auto airplane mode toggle via ADB internal modem.
- **Sticky Session Ready**: Mendukung autentikasi kredensial SOCKS5 format `user-session_X:password` untuk mengunci sesi per akun.
- **Credential Rotation Trigger**: Mengirim username `user-rotate` di SOCKS5 otomatis memicu pergantian IP.
- **Request-Count Auto-Rotate (`--rotate-every-reqs N`)**: Otomatis ganti IP setelah N request selesai.
- **Time-Based Auto-Rotate (`--auto-rotate-mins N`)**: Otomatis ganti IP setiap N menit.
- **REST Control API (`:10808`)**: Endpoint `/rotate`, `/status`, `/ip` untuk integrasi bot/script apa pun.
- **Systemd Daemon 24/7**: Terintegrasi penuh dengan auto-start saat boot.

---

## 🛠️ Cara Install & Deploy 24/7

Di direktori project `/opt/app/opa-lte2proxy/`:
```bash
cd /opt/app/opa-lte2proxy
./install.sh
```

Installer otomatis:
1. Membuild binary release via `cargo build --release`.
2. Memasang executable ke `/usr/local/bin/opa-lte2proxy` dan membuat symlink alias **`olp`**.
3. Memasang config ke `/etc/opa-lte2proxy.conf`.
4. Mengonfigurasi rule routing table 200.
5. Memasang dan mengaktifkan service systemd `opa-lte2proxy.service` (berjalan 24/7).

---

## 💻 Perintah CLI (`olp`)

Framework menyediakan CLI shortcut **`olp`** yang bisa dipanggil dari mana saja:

| Perintah | Deskripsi |
| :--- | :--- |
| `olp status` | Menampilkan telemetri operator, status sinyal, IP aktif, dan jumlah koneksi |
| `olp ip` | Menampilkan IP seluler modem saat ini secara ringkas |
| `olp rotate` | Memicu pergantian IP publik via Airplane Mode cycle |
| `olp rotate --force` | Memaksa rotasi IP meskipun server sedang dalam mode failover darurat |
| `olp rotate --session acc2` | Memicu rotasi dengan menandai session ID tertentu di log telemetri |
| `sudo systemctl restart opa-lte2proxy` | Restart service proxy background |

---

## 🔄 Cara Pakai Sticky Session

Untuk mengelola farming ratusan akun tanpa terkena checkpoint Google/Instagram/TikTok:

### 1. Sticky Sesi per Akun (On-Demand via REST API)
Pola paling stabil untuk bot/worker paralel:
1. Bot memproses akun #1 lewat `socks5://127.0.0.1:10800` (IP tetap sticky sepanjang login akun #1).
2. Setelah akun #1 sukses/selesai, bot memanggil REST API:
   ```bash
   curl -X POST http://127.0.0.1:10808/rotate
   ```
3. Modem melakukan Airplane mode ~10 detik dan mendapatkan IP baru.
4. Bot lanjut memproses akun #2 dengan IP seluler baru yang fresh!

### 2. Sticky Sesi via Kredensial SOCKS5
Dapat langsung disuplai ke tool atau browser antidetect yang mendukung format autentikasi:
```text
socks5://user-session_123:anypassword@127.0.0.1:10800
```
- Request dengan session ID yang sama dianggap satu sesi identitas.
- Jika ingin memicu rotasi langsung dari browser/bot tanpa panggil API HTTP:
  ```text
  socks5://user-rotate:anypassword@127.0.0.1:10800
  ```

---

## 🌐 REST Control API (Port 10808)

### `GET /status`
Mengembalikan status lengkap sistem:
```bash
curl -s http://127.0.0.1:10808/status | jq .
```
```json
{
  "status": "online",
  "modem": {
    "connected": true,
    "internal_ip": "10.3.13.96",
    "operator": "XL",
    "network_type": "LTE",
    "pdp_active": true,
    "rotation_count": 2,
    "host_on_lte_failover": false
  },
  "proxy": {
    "active_connections": 0,
    "total_connections": 18,
    "rotate_every_reqs": 0
  }
}
```

### `POST /rotate`
Trigger pergantian IP langsung:
```bash
curl -s -X POST http://127.0.0.1:10808/rotate | jq .
```
```json
{
  "status": "success",
  "old_ip": "10.5.84.210",
  "new_ip": "10.3.13.96",
  "duration_ms": 12942,
  "rotation_count": 1,
  "session": "default"
}
```

---

## 🤖 Integrasi Bot Farming (`ag29router`)

Di file `/opt/garapan/ag29router/proxies.txt`, masukkan endpoint SOCKS5 lokal:
```text
socks5://127.0.0.1:10800
```

Worker Camoufox akan otomatis merutekan Google OAuth headless lewat IP seluler kartu SIM modem.

---

## 🔍 Troubleshooting & Log

Cek log service live:
```bash
sudo journalctl -u opa-lte2proxy -f
```

Periksa apakah modem terdeteksi di ADB:
```bash
adb devices
# Output harus muncul: 2285100c device
```

Periksa rule routing table 200:
```bash
ip rule show | grep 200
ip route show table 200
```

---

<p align="center">
  <b>OPA-LTE2PROXY</b> — Dibuat untuk ekosistem server Bojongkulur.
</p>
