use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info};

use crate::modem::ModemController;

pub struct Socks5Server {
    listen_addr: SocketAddr,
    egress_ip: IpAddr,
    modem: ModemController,
    pub active_conns: Arc<AtomicU64>,
    pub total_conns: Arc<AtomicU64>,
    pub rotate_every_reqs: u64,
}

impl Socks5Server {
    pub fn new(
        listen_addr: SocketAddr,
        egress_ip: IpAddr,
        modem: ModemController,
        rotate_every_reqs: u64,
    ) -> Self {
        Self {
            listen_addr,
            egress_ip,
            modem,
            active_conns: Arc::new(AtomicU64::new(0)),
            total_conns: Arc::new(AtomicU64::new(0)),
            rotate_every_reqs,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!(
            "SOCKS5 Server listening on {} -> Egress IP {} (Sticky/Rotating Ready)",
            self.listen_addr, self.egress_ip
        );

        loop {
            let (client_stream, client_addr) = listener.accept().await?;
            let egress_ip = self.egress_ip;
            let active = Arc::clone(&self.active_conns);
            let total = Arc::clone(&self.total_conns);
            let modem = self.modem.clone();
            let rotate_reqs = self.rotate_every_reqs;

            tokio::spawn(async move {
                active.fetch_add(1, Ordering::Relaxed);
                let conn_idx = total.fetch_add(1, Ordering::Relaxed) + 1;

                if let Err(e) = handle_connection(client_stream, egress_ip, &modem).await {
                    debug!("SOCKS5 connection error from {}: {}", client_addr, e);
                }

                active.fetch_sub(1, Ordering::Relaxed);

                // Auto-rotate every N requests if configured
                if rotate_reqs > 0 && conn_idx % rotate_reqs == 0 {
                    info!("[Sticky/Rotate] Request threshold reached ({} reqs). Triggering IP rotation...", conn_idx);
                    let m = modem.clone();
                    tokio::spawn(async move {
                        let _ = m.rotate_ip(false).await;
                    });
                }
            });
        }
    }
}

async fn handle_connection(
    mut client: TcpStream,
    egress_ip: IpAddr,
    modem: &ModemController,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Negotiation
    let mut buf = [0u8; 256];
    client.read_exact(&mut buf[..2]).await?;

    let version = buf[0];
    if version != 0x05 {
        return Err("Unsupported SOCKS version".into());
    }

    let nmethods = buf[1] as usize;
    client.read_exact(&mut buf[..nmethods]).await?;

    let methods = &buf[..nmethods];
    let supports_auth = methods.contains(&0x02); // USERNAME/PASSWORD auth (for session tracking)
    let supports_no_auth = methods.contains(&0x00);

    if supports_auth {
        // Offer Username/Password auth so client can pass sticky session or rotate trigger
        client.write_all(&[0x05, 0x02]).await?;

        // Read subnegotiation (RFC 1929)
        let sub_ver = client.read_u8().await?;
        if sub_ver != 0x01 {
            return Err("Unsupported auth subnegotiation".into());
        }

        let ulen = client.read_u8().await? as usize;
        let mut ubuf = vec![0u8; ulen];
        client.read_exact(&mut ubuf).await?;
        let username = String::from_utf8_lossy(&ubuf).to_string();

        let plen = client.read_u8().await? as usize;
        let mut pbuf = vec![0u8; plen];
        client.read_exact(&mut pbuf).await?;

        let session_id = username.clone();

        // If client sends username 'rotate' or 'user-rotate', trigger immediate IP rotation
        if username.to_lowercase().contains("rotate") {
            info!("[Sticky Engine] Client requested explicit rotation via credentials ('{}')", username);
            let m = modem.clone();
            tokio::spawn(async move {
                let _ = m.rotate_ip(false).await;
            });
        } else {
            debug!("[Sticky Engine] Sticky session identified: '{}'", session_id);
        }

        // Accept any credentials (auth success)
        client.write_all(&[0x01, 0x00]).await?;
    } else if supports_no_auth {
        // Accept NO_AUTH (0x00)
        client.write_all(&[0x05, 0x00]).await?;
    } else {
        client.write_all(&[0x05, 0xFF]).await?;
        return Err("No acceptable auth methods".into());
    }

    // 2. Request Details
    let mut req_header = [0u8; 4];
    client.read_exact(&mut req_header).await?;

    let cmd = req_header[1];
    let addr_type = req_header[3];

    if cmd != 0x01 {
        // Only CONNECT is supported
        client.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
        return Err("Command not supported".into());
    }

    let target_host = match addr_type {
        0x01 => {
            // IPv4
            let mut ip_buf = [0u8; 4];
            client.read_exact(&mut ip_buf).await?;
            IpAddr::V4(ip_buf.into()).to_string()
        }
        0x03 => {
            // Domain
            let len = client.read_u8().await? as usize;
            let mut domain_buf = vec![0u8; len];
            client.read_exact(&mut domain_buf).await?;
            String::from_utf8(domain_buf)?
        }
        0x04 => {
            // IPv6
            let mut ip_buf = [0u8; 16];
            client.read_exact(&mut ip_buf).await?;
            IpAddr::V6(ip_buf.into()).to_string()
        }
        _ => {
            client.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Err("Address type not supported".into());
        }
    };

    let target_port = client.read_u16().await?;
    let target_str = format!("{}:{}", target_host, target_port);

    // 3. Bind Outbound Socket to Modem Egress IP
    let mut target_stream = match connect_via_egress(&target_str, egress_ip).await {
        Ok(s) => s,
        Err(e) => {
            client.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Err(e);
        }
    };

    // Reply Success
    client.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;

    // 4. Bi-directional Stream Forwarding
    copy_bidirectional(&mut client, &mut target_stream).await?;

    Ok(())
}

async fn connect_via_egress(
    target: &str,
    egress_ip: IpAddr,
) -> Result<TcpStream, Box<dyn std::error::Error + Send + Sync>> {
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host(target).await?.collect();
    if addrs.is_empty() {
        return Err("Could not resolve target host".into());
    }

    let target_addr = addrs[0];

    let domain = if target_addr.is_ipv4() {
        socket2::Domain::IPV4
    } else {
        socket2::Domain::IPV6
    };

    let socket = socket2::Socket::new(domain, socket2::Type::STREAM, Some(socket2::Protocol::TCP))?;
    socket.set_reuse_address(true)?;

    let local_bind: SocketAddr = SocketAddr::new(egress_ip, 0);
    socket.bind(&local_bind.into())?;
    socket.set_nonblocking(true)?;

    let std_stream = match socket.connect_timeout(&target_addr.into(), Duration::from_secs(12)) {
        Ok(_) => std::net::TcpStream::from(socket),
        Err(e) => return Err(format!("Socket connect failed: {}", e).into()),
    };

    let tokio_stream = TcpStream::from_std(std_stream)?;
    Ok(tokio_stream)
}
