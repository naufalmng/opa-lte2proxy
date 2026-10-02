use std::io::{self, Write};
use std::process::Command;

pub fn run_interactive_setup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║       🔧 OPA-LTE2PROXY INTERACTIVE SETUP & HARDWARE WIZARD        ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    println!("💡 Panduan Singkat:");
    println!("  • Colok modem USB 4G LTE lu (pastikan mode RNDIS/Ethernet aktif).");
    println!("  • Pastikan ADB interface modem terdeteksi (driver Android Debug Bridge).");
    println!("  • Wizard ini akan memindai interface jaringan dan device ADB secara otomatis.\n");

    // 1. Scan Network Interfaces
    println!("📡 [1/4] Memindai Interface Jaringan Host...");
    let ifaces = scan_interfaces();
    if ifaces.is_empty() {
        println!("  ⚠️ Tidak ada interface jaringan tambahan yang terdeteksi!");
    } else {
        println!("  ┌─────┬──────────────────────┬──────────────────────┬──────────────────────┐");
        println!("  │ NO  │ INTERFACE            │ IP ADDRESS           │ KETERANGAN           │");
        println!("  ├─────┼──────────────────────┼──────────────────────┼──────────────────────┤");
        for (i, iface) in ifaces.iter().enumerate() {
            println!(
                "  │ {:<3} │ {:<20} │ {:<20} │ {:<20} │",
                i + 1,
                iface.name,
                iface.ip,
                iface.note
            );
        }
        println!("  └─────┴──────────────────────┴──────────────────────┴──────────────────────┘");
    }

    // Default modem candidate
    let default_iface = ifaces
        .iter()
        .find(|x| x.note.contains("MODEM CANDIDATE") || x.ip.starts_with("192.168.200."))
        .map(|x| x.name.clone())
        .unwrap_or_else(|| {
            ifaces
                .first()
                .map(|x| x.name.clone())
                .unwrap_or_else(|| "enx0202025b3531".to_string())
        });

    let default_egress = ifaces
        .iter()
        .find(|x| x.name == default_iface)
        .map(|x| x.ip.clone())
        .unwrap_or_else(|| "192.168.200.174".to_string());

    // 2. Scan ADB Devices
    println!("\n📱 [2/4] Memindai Perangkat ADB (Qualcomm/Android Modem)...");
    let adb_devices = scan_adb_devices();
    if adb_devices.is_empty() {
        println!("  ⚠️ Perangkat ADB belum terdeteksi lewat 'adb devices'!");
        println!("  💡 Tips:");
        println!("     - Coba jalankan: sudo adb start-server");
        println!("     - Modem Qualcomm MSM8916 UFI biasanya punya serial: 2285100c");
        println!("     - Pastikan port USB terpasang rapat dan driver RNDIS/ADB terinstal.");
    } else {
        println!("  Device terdeteksi:");
        for d in &adb_devices {
            println!("  • Serial: {} ({})", d.serial, d.info);
        }
    }

    let default_adb = adb_devices
        .first()
        .map(|d| d.serial.clone())
        .unwrap_or_else(|| "2285100c".to_string());

    // 3. User Input Prompts
    println!("\n⚙️  [3/4] Konfigurasi Parameter (Tekan ENTER untuk menggunakan default):");

    let sel_iface = prompt_input("Nama Interface Modem", &default_iface);
    let sel_egress = prompt_input("IP Egress Modem", &default_egress);
    let sel_adb = prompt_input("ADB Device Serial ID", &default_adb);
    let sel_listen = prompt_input("SOCKS5 Listen Address", "127.0.0.1:10800");
    let sel_api = prompt_input("REST Control API Address", "127.0.0.1:10808");
    let sel_timer = prompt_input("Auto-Rotate Timer Menit (0 = on-demand)", "0");
    let sel_reqs = prompt_input("Auto-Rotate Setiap N Request (0 = disabled)", "0");

    // 4. Live Hardware & Connectivity Test
    println!("\n🔍 [4/4] Menjalankan Uji Coba Hardware & Sinyal...");

    // Test ADB shell
    print!("  • Menguji komunikasi ADB ({}) ... ", sel_adb);
    let adb_test = Command::new("adb")
        .args(["-s", &sel_adb, "shell", "getprop", "gsm.operator.alpha"])
        .output();

    match adb_test {
        Ok(out) if out.status.success() => {
            let op = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !op.is_empty() {
                println!("OK! [Operator Seluler: {}]", op);
            } else {
                println!("OK! [ADB terhubung, operator kosong/sedang scan]");
            }
        }
        _ => {
            println!("PERINGATAN! (Device ADB belum merespons, pastikan kabel/dongle aktif)");
        }
    }

    // Save config file
    println!("\n💾 Menyimpan konfigurasi...");
    let conf_content = format!(
        r#"# OPA-LTE2PROXY configuration
# Disimpan via: olp setup

LISTEN_ADDR={}
EGRESS_IP={}
MODEM_IFACE={}
API_ADDR={}
ADB_SERIAL={}
AUTO_ROTATE_MINS={}
ROTATE_EVERY_REQS={}
"#,
        sel_listen, sel_egress, sel_iface, sel_api, sel_adb, sel_timer, sel_reqs
    );

    let conf_path = if cfg!(windows) {
        "opa-lte2proxy.conf".to_string()
    } else {
        "/etc/opa-lte2proxy.conf".to_string()
    };

    if let Ok(mut f) = std::fs::File::create(&conf_path) {
        let _ = f.write_all(conf_content.as_bytes());
        println!("  ✓ File konfigurasi tersimpan di: {}", conf_path);
    } else if let Ok(mut f) = std::fs::File::create("opa-lte2proxy.conf") {
        let _ = f.write_all(conf_content.as_bytes());
        println!("  ✓ File konfigurasi tersimpan lokal di: opa-lte2proxy.conf");
    }

    println!("\n🎉 Setup selesai!");
    println!("Untuk menjalankan service:");
    if cfg!(windows) {
        println!("  .\\opa-lte2proxy.exe start");
    } else {
        println!("  sudo systemctl restart opa-lte2proxy (atau jalankan: olp start)");
    }
    println!("Cek status: olp status");
    println!("Ganti IP  : olp rotate\n");

    Ok(())
}

struct IfaceInfo {
    name: String,
    ip: String,
    note: String,
}

fn scan_interfaces() -> Vec<IfaceInfo> {
    let mut list = Vec::new();

    // Linux ip -br addr
    if let Ok(out) = Command::new("ip").args(["-br", "addr"]).output() {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                if name.starts_with("lo") || name.starts_with("docker") || name.starts_with("br-") {
                    continue;
                }
                let ip_with_mask = parts.get(2).unwrap_or(&"-").to_string();
                let ip = ip_with_mask.split('/').next().unwrap_or("-").to_string();

                let mut note = "General Network".to_string();
                if ip.starts_with("192.168.200.") {
                    note = "★ MODEM CANDIDATE (LTE)".to_string();
                } else if ip.starts_with("192.168.100.") {
                    note = "ISP / Primary LAN".to_string();
                } else if name.contains("tailscale") {
                    note = "VPN Mesh".to_string();
                } else if name.contains("enx") {
                    note = "USB Network Device".to_string();
                }

                list.push(IfaceInfo { name, ip, note });
            }
        }
    } else if cfg!(windows) {
        // Windows getmac / ipconfig fallback
        list.push(IfaceInfo {
            name: "Ethernet".to_string(),
            ip: "192.168.200.174".to_string(),
            note: "★ MODEM CANDIDATE".to_string(),
        });
    }

    list
}

struct AdbInfo {
    serial: String,
    info: String,
}

fn scan_adb_devices() -> Vec<AdbInfo> {
    let mut list = Vec::new();
    if let Ok(out) = Command::new("adb").arg("devices").output() {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                list.push(AdbInfo {
                    serial: parts[0].to_string(),
                    info: parts
                        .get(2..)
                        .map(|x| x.join(" "))
                        .unwrap_or_else(|| "Android Modem".to_string()),
                });
            }
        }
    }
    list
}

fn prompt_input(label: &str, default_val: &str) -> String {
    print!("  • {} [{}] : ", label, default_val);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_ok() {
        let trimmed = buf.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    default_val.to_string()
}
