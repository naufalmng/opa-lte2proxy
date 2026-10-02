use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ModemController {
    adb_id: String,
    modem_iface: String,
    rotation_lock: Arc<Mutex<()>>,
    pub rotation_count: Arc<AtomicU64>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ModemStatus {
    pub connected: bool,
    pub internal_ip: String,
    pub operator: String,
    pub network_type: String,
    pub pdp_active: bool,
    pub rotation_count: u64,
    pub host_on_lte_failover: bool,
}

impl ModemController {
    pub fn new(adb_id: Option<String>, modem_iface: Option<String>) -> Self {
        let detected_adb = adb_id.unwrap_or_else(|| {
            if let Ok(out) = Command::new("adb").arg("devices").output() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[1] == "device" {
                        return parts[0].to_string();
                    }
                }
            }
            "2285100c".to_string()
        });

        Self {
            adb_id: detected_adb,
            modem_iface: modem_iface.unwrap_or_else(|| "enx0202025b3531".to_string()),
            rotation_lock: Arc::new(Mutex::new(())),
            rotation_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Cek apakah sistem server saat ini sedang failover ke LTE (ISP mati)
    pub fn is_host_on_lte_failover(&self) -> bool {
        if let Ok(out) = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.contains(&self.modem_iface) || s.contains("192.168.200.1");
        }
        false
    }

    pub fn get_adb_prop(&self, prop: &str) -> String {
        let out = Command::new("adb")
            .args(["-s", &self.adb_id, "shell", "getprop", prop])
            .output();

        match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "".to_string(),
        }
    }

    pub fn get_internal_ip(&self) -> String {
        let out = Command::new("adb")
            .args([
                "-s",
                &self.adb_id,
                "shell",
                "ip",
                "-o",
                "-4",
                "addr",
                "show",
                "rmnet0",
            ])
            .output();

        if let Ok(o) = out {
            let s = String::from_utf8_lossy(&o.stdout);
            for part in s.split_whitespace() {
                if part.contains('/') && part.starts_with("10.") {
                    return part.split('/').next().unwrap_or("").to_string();
                }
            }
        }
        "".to_string()
    }

    pub fn get_status(&self) -> ModemStatus {
        let pdp_active = self.get_adb_prop("gsm.defaultpdpcontext.active") == "true";
        let operator = self.get_adb_prop("gsm.operator.alpha");
        let network_type = self.get_adb_prop("gsm.network.type");
        let ip = self.get_internal_ip();
        let failover = self.is_host_on_lte_failover();

        ModemStatus {
            connected: !ip.is_empty(),
            internal_ip: ip,
            operator,
            network_type,
            pdp_active,
            rotation_count: self.rotation_count.load(Ordering::Relaxed),
            host_on_lte_failover: failover,
        }
    }

    pub async fn rotate_ip(&self, force: bool) -> Result<(String, String, u64), String> {
        // FAILOVER GUARD: Jangan rotate kalau server lagi numpang di modem (kecuali di-force)
        if !force && self.is_host_on_lte_failover() {
            return Err("FAILOVER_GUARD: Server saat ini sedang failover ke LTE (ISP mati). Rotasi IP dikunci agar koneksi server tidak putus. Kirim ?force=true untuk bypass.".to_string());
        }

        let _guard = self.rotation_lock.lock().await;
        let t0 = Instant::now();
        let old_ip = self.get_internal_ip();

        info!(
            "Starting IP rotation via ADB Airplane Mode cycle on device {}...",
            self.adb_id
        );

        // 1. Airplane mode ON
        let _ = tokio::process::Command::new("adb")
            .args([
                "-s",
                &self.adb_id,
                "shell",
                "settings",
                "put",
                "global",
                "airplane_mode_on",
                "1",
            ])
            .output()
            .await;

        let _ = tokio::process::Command::new("adb")
            .args([
                "-s",
                &self.adb_id,
                "shell",
                "am",
                "broadcast",
                "-a",
                "android.intent.action.AIRPLANE_MODE",
                "--ez",
                "state",
                "true",
            ])
            .output()
            .await;

        tokio::time::sleep(Duration::from_millis(2500)).await;

        // 2. Airplane mode OFF
        let _ = tokio::process::Command::new("adb")
            .args([
                "-s",
                &self.adb_id,
                "shell",
                "settings",
                "put",
                "global",
                "airplane_mode_on",
                "0",
            ])
            .output()
            .await;

        let _ = tokio::process::Command::new("adb")
            .args([
                "-s",
                &self.adb_id,
                "shell",
                "am",
                "broadcast",
                "-a",
                "android.intent.action.AIRPLANE_MODE",
                "--ez",
                "state",
                "false",
            ])
            .output()
            .await;

        // 3. Wait for PDP context re-attachment
        let mut new_ip = String::new();
        for _ in 0..15 {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let current = self.get_internal_ip();
            if !current.is_empty() && current != old_ip {
                new_ip = current;
                break;
            }
            if !current.is_empty() && old_ip.is_empty() {
                new_ip = current;
                break;
            }
        }

        if new_ip.is_empty() {
            new_ip = self.get_internal_ip();
        }

        let elapsed = t0.elapsed().as_millis() as u64;
        self.rotation_count.fetch_add(1, Ordering::Relaxed);

        if !new_ip.is_empty() {
            info!(
                "IP rotation successful: {} -> {} (took {}ms)",
                old_ip, new_ip, elapsed
            );
            Ok((old_ip, new_ip, elapsed))
        } else {
            warn!(
                "IP rotation completed but IP was not refreshed (took {}ms)",
                elapsed
            );
            Ok((old_ip, "waiting_carrier".to_string(), elapsed))
        }
    }
}
