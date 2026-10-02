use std::collections::HashMap;
use std::io::{self, Write};
use std::process::Command;

pub fn run_interactive_setup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                 🛠️  ALL-IN-ONE INTERACTIVE SETUP                   ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    println!("💡 Panduan Singkat:");
    println!("  • Tool ini bekerja dengan semua modem 4G LTE USB (Qualcomm, Huawei, ZTE, dsb).");
    println!("  • Mendukung setup otomatis maupun konfigurasi manual tanpa locking interface.\n");

    println!("Pilih Mode Setup:");
    println!("  [1] Auto-Detect (Rekomendasi — Deteksi cerdas interface & modem ADB)");
    println!(
        "  [2] Custom / Manual (Pilih interface manual, tentukan IP gateway & subnet sendiri)\n"
    );

    let mode_choice = prompt_input("Pilihan mode (1/2)", "1");

    if mode_choice == "2" {
        run_manual_setup()?;
    } else {
        run_auto_setup()?;
    }

    Ok(())
}

fn run_auto_setup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📡 [1/3] Memindai Interface Jaringan & Perangkat ADB...");
    let ifaces = scan_interfaces();
    let adb_devices = scan_adb_devices();

    let candidate_iface = ifaces
        .iter()
        .find(|x| x.note.contains("MODEM CANDIDATE (LTE)"))
        .or_else(|| ifaces.iter().find(|x| x.note.contains("MODEM CANDIDATE")))
        .or_else(|| ifaces.first());

    let (sel_iface, sel_egress, sel_gw) = match candidate_iface {
        Some(c) => (c.name.clone(), c.ip.clone(), c.gw.clone()),
        None => (
            "enx0202025b3531".to_string(),
            "192.168.200.174".to_string(),
            "192.168.200.1".to_string(),
        ),
    };

    let sel_adb = adb_devices
        .first()
        .map(|d| d.serial.clone())
        .unwrap_or_else(|| "2285100c".to_string());

    println!("\n🔍 Hasil Deteksi Otomatis:");
    println!("  • Interface Modem : {}", sel_iface);
    println!("  • IP Egress Host  : {}", sel_egress);
    println!("  • Gateway Modem   : {}", sel_gw);
    println!("  • Device Serial   : {}", sel_adb);
    println!("  • SOCKS5 Port     : 127.0.0.1:10800");
    println!("  • REST API Port   : 127.0.0.1:10808");

    let confirm = prompt_input("\nGunakan konfigurasi otomatis ini? [Y/n]", "y");
    if confirm.to_lowercase().starts_with('n') {
        println!("\nBeralih ke mode konfigurasi manual...");
        return run_manual_setup();
    }

    finalize_setup(
        &sel_iface,
        &sel_egress,
        &sel_gw,
        "200",
        &sel_adb,
        "127.0.0.1:10800",
        "127.0.0.1:10808",
        "0",
        "0",
    )
}

fn run_manual_setup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📡 [1/3] Daftar Seluruh Interface Jaringan yang Ditemukan:");
    let ifaces = scan_interfaces();

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

    let iface_idx_str = prompt_input(
        "\nPilih nomor interface modem (atau ketik nama interface)",
        "1",
    );
    let (sel_iface, default_ip, default_gw) = if let Ok(idx) = iface_idx_str.parse::<usize>() {
        if idx >= 1 && idx <= ifaces.len() {
            let item = &ifaces[idx - 1];
            (item.name.clone(), item.ip.clone(), item.gw.clone())
        } else {
            (
                iface_idx_str.clone(),
                "192.168.200.174".to_string(),
                "192.168.200.1".to_string(),
            )
        }
    } else {
        (
            iface_idx_str.clone(),
            "192.168.200.174".to_string(),
            "192.168.200.1".to_string(),
        )
    };

    println!("\n📱 [2/3] Daftar Perangkat ADB:");
    let adb_devices = scan_adb_devices();
    if adb_devices.is_empty() {
        println!("  (Tidak ada device ADB terdeteksi otomatis)");
    } else {
        for (i, d) in adb_devices.iter().enumerate() {
            println!("  [{}] Serial: {} ({})", i + 1, d.serial, d.info);
        }
    }

    let default_adb = adb_devices
        .first()
        .map(|d| d.serial.clone())
        .unwrap_or_else(|| "2285100c".to_string());

    println!(
        "\n⚙️  [3/3] Masukkan Parameter Kustom (Tekan ENTER untuk menggunakan nilai dalam kurung):"
    );
    let final_iface = prompt_input("Nama Interface Modem", &sel_iface);
    let final_egress = prompt_input("IP Egress Modem", &default_ip);
    let final_gw = prompt_input("Gateway IP Modem", &default_gw);
    let final_table = prompt_input("Linux Routing Table ID", "200");
    let final_adb = prompt_input("ADB Device Serial ID", &default_adb);
    let final_listen = prompt_input("SOCKS5 Listen Address", "127.0.0.1:10800");
    let final_api = prompt_input("REST Control API Address", "127.0.0.1:10808");
    let final_timer = prompt_input("Auto-Rotate Timer Menit (0 = on-demand)", "0");
    let final_reqs = prompt_input("Auto-Rotate Setiap N Request (0 = disabled)", "0");

    finalize_setup(
        &final_iface,
        &final_egress,
        &final_gw,
        &final_table,
        &final_adb,
        &final_listen,
        &final_api,
        &final_timer,
        &final_reqs,
    )
}

fn finalize_setup(
    iface: &str,
    egress: &str,
    gw: &str,
    table: &str,
    adb_id: &str,
    listen: &str,
    api: &str,
    timer: &str,
    reqs: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🔍 Menjalankan Diagnostik Koneksi & Hardware...");

    // 1. Test Gateway ping
    print!("  • Menguji konektivitas Gateway ({}) ... ", gw);
    let ping_test = Command::new("ping")
        .args(["-c", "1", "-W", "2", gw])
        .output();
    if let Ok(out) = ping_test {
        if out.status.success() {
            println!("TERHUBUNG!");
        } else {
            println!("TIDAK MERESPONS (Pastikan subnet/gateway benar)");
        }
    } else {
        println!("SKIPPED");
    }

    // 2. Test ADB
    print!("  • Menguji komunikasi ADB ({}) ... ", adb_id);
    let adb_test = Command::new("adb")
        .args(["-s", adb_id, "shell", "getprop", "gsm.operator.alpha"])
        .output();

    if let Ok(out) = adb_test {
        if out.status.success() {
            let op = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !op.is_empty() {
                println!("OK! [Operator: {}]", op);
            } else {
                println!("OK! [ADB Aktif]");
            }
        } else {
            println!("DEVICE TIDAK MERESPONS (Cek kabel/port ADB)");
        }
    } else {
        println!("ADB COMMAND TIDAK DITEMUKAN");
    }

    // 3. Setup Linux Routing Table if on Linux
    if cfg!(target_os = "linux") && !egress.is_empty() && !gw.is_empty() {
        print!(
            "  • Mengonfigurasi rule routing isolasi (Table {}) ... ",
            table
        );
        let _ = Command::new("sudo")
            .args(["ip", "rule", "add", "from", egress, "table", table])
            .output();
        let _ = Command::new("sudo")
            .args([
                "ip", "route", "add", "default", "via", gw, "dev", iface, "table", table,
            ])
            .output();
        println!("AKTIF!");
    }

    // 4. Save Config File
    println!("\n💾 Menyimpan file konfigurasi...");
    let conf_content = format!(
        r#"# OPA-LTE2PROXY configuration
# Disimpan via: olp setup

LISTEN_ADDR={}
EGRESS_IP={}
MODEM_IFACE={}
MODEM_GW={}
ROUTING_TABLE={}
API_ADDR={}
ADB_SERIAL={}
AUTO_ROTATE_MINS={}
ROTATE_EVERY_REQS={}
"#,
        listen, egress, iface, gw, table, api, adb_id, timer, reqs
    );

    let conf_path = if cfg!(windows) {
        "opa-lte2proxy.conf".to_string()
    } else {
        "/etc/opa-lte2proxy.conf".to_string()
    };

    let mut saved = false;
    if let Ok(mut f) = std::fs::File::create(&conf_path) {
        if f.write_all(conf_content.as_bytes()).is_ok() {
            println!("  ✓ Tersimpan di: {}", conf_path);
            saved = true;
        }
    }
    if !saved {
        if let Ok(mut f) = std::fs::File::create("opa-lte2proxy.conf") {
            let _ = f.write_all(conf_content.as_bytes());
            println!("  ✓ Tersimpan lokal di: opa-lte2proxy.conf");
        }
    }

    // 5. Offer restart service
    let restart_prompt = prompt_input("\nRestart service opa-lte2proxy sekarang? [Y/n]", "y");
    if restart_prompt.to_lowercase().starts_with('y') {
        if cfg!(target_os = "linux") {
            let _ = Command::new("sudo")
                .args(["systemctl", "restart", "opa-lte2proxy"])
                .output();
            println!("  ✓ Service opa-lte2proxy berhasil di-restart!");
        } else if cfg!(windows) {
            println!("  (Di Windows, jalankan: opa-lte2proxy.exe start)");
        }
    }

    println!("\n🎉 Setup selesai dan siap digunakan!");
    println!("Cek status : olp status");
    println!("Cek IP     : olp ip");
    println!("Ganti IP   : olp rotate\n");

    Ok(())
}

pub fn load_config_file() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let paths = ["/etc/opa-lte2proxy.conf", "opa-lte2proxy.conf"];
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
            break;
        }
    }
    map
}

#[derive(Clone)]
struct IfaceInfo {
    name: String,
    ip: String,
    gw: String,
    note: String,
}

fn scan_interfaces() -> Vec<IfaceInfo> {
    let mut list = Vec::new();

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

                let mut gw = "192.168.200.1".to_string();
                let mut note = "General Network".to_string();

                if ip.starts_with("192.168.200.") {
                    note = "★ MODEM CANDIDATE (LTE)".to_string();
                    gw = "192.168.200.1".to_string();
                } else if ip.starts_with("192.168.8.") {
                    note = "★ MODEM CANDIDATE (Huawei/ZTE)".to_string();
                    gw = "192.168.8.1".to_string();
                } else if ip.starts_with("192.168.0.") {
                    note = "★ MODEM/LAN Subnet .0".to_string();
                    gw = "192.168.0.1".to_string();
                } else if ip.starts_with("192.168.100.") {
                    note = "ISP / Primary LAN".to_string();
                    gw = "192.168.100.1".to_string();
                } else if name.contains("tailscale") {
                    note = "VPN Mesh".to_string();
                    gw = "-".to_string();
                } else if name.contains("enx") || name.contains("usb") || name.contains("wwan") {
                    note = "★ USB/Cellular Network Device".to_string();
                }

                list.push(IfaceInfo { name, ip, gw, note });
            }
        }
    } else if cfg!(windows) {
        list.push(IfaceInfo {
            name: "Ethernet".to_string(),
            ip: "192.168.200.174".to_string(),
            gw: "192.168.200.1".to_string(),
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
