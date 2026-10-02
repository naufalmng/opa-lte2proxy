#!/usr/bin/env bash
# ==============================================================================
# OPA-LTE2PROXY Installer & Setup Script
# "Opa jagain proxy lo, gonta-ganti IP tanpa cabut colok."
# ==============================================================================
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_SRC="$DIR/target/release/opa-lte2proxy"
BIN_DEST="/usr/local/bin/opa-lte2proxy"
ALIAS_DEST="/usr/local/bin/olp"
SERVICE_NAME="opa-lte2proxy.service"
SERVICE_DEST="/etc/systemd/system/$SERVICE_NAME"
CONF_DEST="/etc/opa-lte2proxy.conf"

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

echo -e "${CYAN}"
cat << 'EOF'
 ██████╗ ██████╗  █████╗       ██╗  ████████╗███████╗██████╗ ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
██╔═══██╗██╔══██╗██╔══██╗      ██║  ╚══██╔══╝██╔════╝╚════██╗██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
██║   ██║██████╔╝███████║█████╗██║     ██║   █████╗   █████╔╝██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
██║   ██║██╔═══╝ ██╔══██║╚════╝██║     ██║   ██╔══╝  ██╔═══╝ ██╔═══╝ ██║   ██║ ██╔██╗   ╚██╔╝  
╚██████╔╝██║     ██║  ██║      ███████╗██║   ███████╗███████╗██║     ╚██████╔╝██╔╝ ██╗   ██║   
 ╚═════╝ ╚═╝     ╚═╝  ╚═╝      ╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   
EOF
echo -e "${RESET}"
echo -e "${BOLD}🚀 OPA-LTE2PROXY Installer${RESET}"
echo -e "${CYAN}Tagline:${RESET} Opa jagain proxy lo, gonta-ganti IP tanpa cabut colok.\n"

# 1. Build Binary if needed
if [[ ! -f "$BIN_SRC" ]]; then
  echo -e "${YELLOW}[*] Building Rust release binary...${RESET}"
  if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
  fi
  export CC=/usr/bin/gcc
  cd "$DIR" && cargo build --release
fi

echo -e "${GREEN}[✓] Binary terverifikasi:${RESET} $BIN_SRC"

# 2. Install Binary & CLI Alias
echo -e "${YELLOW}[*] Memasang binary ke /usr/local/bin/...${RESET}"
sudo cp "$BIN_SRC" "$BIN_DEST"
sudo chmod +x "$BIN_DEST"
sudo ln -sf "$BIN_DEST" "$ALIAS_DEST"
echo -e "${GREEN}[✓] Executable terpasang:${RESET} $BIN_DEST (alias: ${BOLD}olp${RESET})"

# 3. Setup Config
if [[ ! -f "$CONF_DEST" ]]; then
  echo -e "${YELLOW}[*] Memasang config ke $CONF_DEST...${RESET}"
  sudo cp "$DIR/opa-lte2proxy.conf" "$CONF_DEST"
else
  echo -e "${CYAN}[i] Config $CONF_DEST sudah ada, mempertahankan file lama.${RESET}"
fi

# 4. Setup Isolated Routing Rule (Table 200)
echo -e "${YELLOW}[*] Mengonfigurasi rule routing terisolasi (Table 200)...${RESET}"
sudo ip rule add from 192.168.200.174 table 200 2>/dev/null || true
sudo ip route add default via 192.168.200.1 dev enx0202025b3531 table 200 2>/dev/null || true
echo -e "${GREEN}[✓] Routing Table 200 aktif (Egress -> 192.168.200.174).${RESET}"

# 5. Install Systemd Service
echo -e "${YELLOW}[*] Memasang systemd service...${RESET}"
sudo cp "$DIR/$SERVICE_NAME" "$SERVICE_DEST"
sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"
sudo systemctl restart "$SERVICE_NAME"
echo -e "${GREEN}[✓] Service $SERVICE_NAME aktif & berjalan!${RESET}"

echo -e "\n${BOLD}${GREEN}================================================================${RESET}"
echo -e "${BOLD}${GREEN} 🎉 INSTALASI OPA-LTE2PROXY SELESAI & SUDAH BERJALAN 24/7!${RESET}"
echo -e "${BOLD}${GREEN}================================================================${RESET}"
echo -e "${CYAN}SOCKS5 Proxy     :${RESET} 127.0.0.1:10800"
echo -e "${CYAN}REST Control API :${RESET} http://127.0.0.1:10808"
echo -e "${CYAN}CLI Shortcuts    :${RESET}"
echo -e "  • ${BOLD}olp status${RESET}         - Cek telemetri modem & status koneksi"
echo -e "  • ${BOLD}olp rotate${RESET}         - Ganti IP publik via Airplane Mode"
echo -e "  • ${BOLD}olp ip${RESET}             - Lihat IP seluler saat ini"
echo -e "  • ${BOLD}olp rotate --force${RESET} - Paksa ganti IP meski server mode failover"
echo -e "  • ${BOLD}sudo systemctl status opa-lte2proxy${RESET}"
echo -e "${BOLD}${GREEN}================================================================${RESET}\n"
