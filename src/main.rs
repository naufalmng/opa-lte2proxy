mod api;
mod modem;
mod setup;
mod socks5;

use clap::{Parser, Subcommand};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const BANNER: &str = r#"
 ██████╗ ██████╗  █████╗       ██╗  ████████╗███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
██╔═══██╗██╔══██╗██╔══██╗      ██║  ╚══██╔══╝██╔════╝╚════██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
██║   ██║██████╔╝███████║█████╗██║     ██║   █████╗   █████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
██║   ██║██╔═══╝ ██╔══██║╚════╝██║     ██║   ██╔══╝  ██╔═══╝ ██╔═══╝ ██║   ██║ ██╔██╗   ╚██╔╝  
╚██████╔╝██║     ██║  ██║      ███████╗██║   ███████╗███████╗██║     ╚██████╔╝██╔╝ ██╗   ██║   
 ╚═════╝ ╚═╝     ╚═╝  ╚═╝      ╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   
"#;

#[derive(Parser, Debug)]
#[command(
    name = "opa-lte2proxy",
    version = "1.0.0",
    about = "OPA-LTE2PROXY — Framework Rotating Mobile SOCKS5 Proxy dengan Auto IP-Cycling & Failover Guard"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Address & port for SOCKS5 proxy server [default: 127.0.0.1:10800]
    #[arg(short = 'l', long, default_value = "127.0.0.1:10800", global = true)]
    listen: SocketAddr,

    /// Outbound IP to bind egress traffic to (Modem SIM) [default: 192.168.200.174]
    #[arg(short = 'b', long, default_value = "192.168.200.174", global = true)]
    bind: IpAddr,

    /// Port for REST API control server [default: 127.0.0.1:10808]
    #[arg(long, default_value = "127.0.0.1:10808", global = true)]
    api: SocketAddr,

    /// Explicit ADB device serial ID (Auto-detect jika tidak diisi)
    #[arg(long, global = true)]
    adb: Option<String>,

    /// Auto-rotate IP setiap N menit (0 = disabled / on-demand)
    #[arg(long, default_value_t = 0, global = true)]
    auto_rotate_mins: u64,

    /// Auto-rotate IP setiap N request SOCKS5 selesai (0 = disabled)
    #[arg(long, default_value_t = 0, global = true)]
    rotate_every_reqs: u64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Jalankan service proxy daemon (default)
    Start,
    /// Setup interaktif / wizard hardware pemindaian modem & interface
    Setup,
    /// Setup interaktif (alias)
    Config,
    /// Trigger rotasi IP secara manual via REST API
    Rotate {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        session: Option<String>,
    },
    /// Cek telemetri modem & status proxy
    Status,
    /// Cek IP seluler aktif saat ini
    Ip,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Setup) | Some(Commands::Config) => {
            setup::run_interactive_setup()?;
            return Ok(());
        }
        Some(Commands::Rotate { force, session }) => {
            let client = reqwest::Client::new();
            let mut url = format!("http://{}/rotate", cli.api);
            let mut params: Vec<String> = Vec::new();
            if *force {
                params.push("force=true".to_string());
            }
            if let Some(s) = session {
                params.push(format!("session={}", s));
            }
            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }

            println!("Triggering IP rotation via {}...", url);
            let res = client.post(&url).send().await?;
            let body = res.text().await?;
            println!("{}", body);
            return Ok(());
        }
        Some(Commands::Status) => {
            let res = reqwest::get(format!("http://{}/status", cli.api)).await?;
            let body = res.text().await?;
            println!("{}", body);
            return Ok(());
        }
        Some(Commands::Ip) => {
            let res = reqwest::get(format!("http://{}/ip", cli.api)).await?;
            let body = res.text().await?;
            println!("{}", body);
            return Ok(());
        }
        Some(Commands::Start) | None => {
            // Jalankan daemon proxy
        }
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "opa_lte2proxy=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = setup::load_config_file();
    let listen_addr: SocketAddr = cfg
        .get("LISTEN_ADDR")
        .and_then(|s| s.parse().ok())
        .unwrap_or(cli.listen);
    let egress_ip: IpAddr = cfg
        .get("EGRESS_IP")
        .and_then(|s| s.parse().ok())
        .unwrap_or(cli.bind);
    let api_addr: SocketAddr = cfg
        .get("API_ADDR")
        .and_then(|s| s.parse().ok())
        .unwrap_or(cli.api);
    let adb_id = cli.adb.or_else(|| cfg.get("ADB_SERIAL").cloned());
    let auto_rotate_mins = cfg
        .get("AUTO_ROTATE_MINS")
        .and_then(|s| s.parse().ok())
        .unwrap_or(cli.auto_rotate_mins);
    let rotate_every_reqs = cfg
        .get("ROTATE_EVERY_REQS")
        .and_then(|s| s.parse().ok())
        .unwrap_or(cli.rotate_every_reqs);
    let modem_iface = cfg.get("MODEM_IFACE").cloned();

    println!("{}", BANNER);
    println!(" 🚀 OPA-LTE2PROXY — Opa jagain proxy lo, gonta-ganti IP tanpa cabut colok.");
    println!(" -----------------------------------------------------------------------");
    println!(" SOCKS5 Proxy  : {}", listen_addr);
    println!(" Bound Egress  : {}", egress_ip);
    println!(" Control API   : http://{}", api_addr);
    if auto_rotate_mins > 0 {
        println!(" Auto-Rotate   : Tiap {} menit", auto_rotate_mins);
    } else {
        println!(" Auto-Rotate   : On-Demand (via /rotate atau credentials)");
    }
    if rotate_every_reqs > 0 {
        println!(" Req-Rotate    : Tiap {} request selesai", rotate_every_reqs);
    }
    println!(" Failover Guard: Aktif (Kunci rotasi otomatis jika ISP rumah mati)");
    println!(" -----------------------------------------------------------------------\n");

    let modem = modem::ModemController::new(adb_id, modem_iface);
    let socks_server = Arc::new(socks5::Socks5Server::new(
        listen_addr,
        egress_ip,
        modem.clone(),
        rotate_every_reqs,
    ));

    let app_state = api::AppState {
        modem: modem.clone(),
        socks: Arc::clone(&socks_server),
    };

    // 1. Spawn SOCKS5 Proxy Server
    let socks_handle = {
        let server = Arc::clone(&socks_server);
        tokio::spawn(async move {
            if let Err(e) = server.run().await {
                error!("SOCKS5 Server error: {}", e);
            }
        })
    };

    // 2. Spawn REST API Control Server
    let api_handle = {
        let addr = api_addr;
        tokio::spawn(async move {
            if let Err(e) = api::run_api_server(addr, app_state).await {
                error!("REST API Server error: {}", e);
            }
        })
    };

    // 3. Optional Auto-Rotate Timer Loop
    if auto_rotate_mins > 0 {
        let modem_ctrl = modem.clone();
        let interval_mins = auto_rotate_mins;
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_mins * 60));
            interval.tick().await; // skip immediate first tick
            loop {
                interval.tick().await;
                info!(
                    "[Timer] Auto-rotating IP every {} minutes...",
                    interval_mins
                );
                if let Err(e) = modem_ctrl.rotate_ip(false).await {
                    error!("[Timer] Auto-rotation skipped/failed: {}", e);
                }
            }
        });
    }

    tokio::select! {
        _ = socks_handle => info!("SOCKS5 task ended"),
        _ = api_handle => info!("API task ended"),
        _ = tokio::signal::ctrl_c() => {
            println!("\n[*] Received SIGINT, shutting down cleanly...");
        }
    }

    Ok(())
}
