#!/usr/bin/env bash
# ==============================================================================
# OPA-LTE2PROXY Installer & Setup Script (Linux & macOS)
# One-liner: curl -fsSL https://raw.githubusercontent.com/naufalmng/opa-lte2proxy/main/install.sh | sudo bash
# ==============================================================================
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "/tmp/opa-lte2proxy")"
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
echo -e "${YELLOW}Tagline:${RESET} Homemade Mobile Rotating Proxy 🐦‍🔥"
echo -e "${CYAN}Catatan:${RESET} Software ini dibuat untuk riset dan kebutuhan pribadi. Segala bentuk penggunaan di luar tanggung jawab pengembang.\n"

OS_TYPE="$(uname -s)"

# 1. Root check
if [[ "$EUID" -ne 0 ]]; then
  echo -e "${RED}[✗] Installer ini butuh izin root (sudo).${RESET}"
  echo "Jalankan: curl -fsSL https://... | sudo bash"
  exit 1
fi

# 2. Check ADB
echo -e "${YELLOW}[1/4] Memeriksa dependensi ADB...${RESET}"
if command -v adb >/dev/null 2>&1; then
  echo -e "${GREEN}[✓] ADB terinstal:${RESET} $(adb version | head -1)"
else
  echo -e "${YELLOW}[!] ADB belum terpasang. Menginstal otomatis...${RESET}"
  if [[ "$OS_TYPE" == "Linux" ]]; then
    if command -v apt-get >/dev/null 2>&1; then
      apt-get update -qq && apt-get install -y -qq adb
    elif command -v dnf >/dev/null 2>&1; then
      dnf install -y adb
    fi
  elif [[ "$OS_TYPE" == "Darwin" ]]; then
    echo "Di macOS, jalankan: brew install android-platform-tools"
  fi
fi

# 3. Build or Install Binary
echo -e "\n${YELLOW}[2/4] Memasang binary opa-lte2proxy...${RESET}"
if [[ -f "$DIR/target/release/opa-lte2proxy" ]]; then
  cp "$DIR/target/release/opa-lte2proxy" "$BIN_DEST"
elif command -v cargo >/dev/null 2>&1; then
  echo "Membuild dari source..."
  export CC=/usr/bin/gcc
  cd "$DIR" && cargo build --release
  cp "$DIR/target/release/opa-lte2proxy" "$BIN_DEST"
else
  echo "Mengunduh pre-built binary..."
  TMP_BIN="/tmp/opa-lte2proxy"
  curl -fsSL -o "$TMP_BIN" "https://github.com/naufalmng/opa-lte2proxy/releases/latest/download/opa-lte2proxy-linux-amd64" || true
  if [[ -f "$TMP_BIN" ]]; then
    mv "$TMP_BIN" "$BIN_DEST"
  fi
fi

chmod +x "$BIN_DEST"
ln -sf "$BIN_DEST" "$ALIAS_DEST"
echo -e "${GREEN}[✓] Executable terpasang di:${RESET} $BIN_DEST (alias: ${BOLD}olp${RESET})"

# 4. Routing Table Setup (Linux Only)
if [[ "$OS_TYPE" == "Linux" ]]; then
  echo -e "\n${YELLOW}[3/4] Mengonfigurasi rule routing terisolasi (Table 200)...${RESET}"
  ip rule add from 192.168.200.174 table 200 2>/dev/null || true
  ip route add default via 192.168.200.1 dev enx0202025b3531 table 200 2>/dev/null || true
  echo -e "${GREEN}[✓] Routing Table 200 aktif.${RESET}"

  # 5. Systemd Service (Linux Only)
  echo -e "\n${YELLOW}[4/4] Memasang dan mengaktifkan systemd service...${RESET}"
  if [[ -f "$DIR/opa-lte2proxy.service" ]]; then
    cp "$DIR/opa-lte2proxy.service" "$SERVICE_DEST"
    systemctl daemon-reload
    systemctl enable "$SERVICE_NAME"
    systemctl restart "$SERVICE_NAME"
    echo -e "${GREEN}[✓] Service $SERVICE_NAME aktif & berjalan 24/7!${RESET}"
  fi
fi

echo -e "\n${BOLD}${GREEN}================================================================${RESET}"
echo -e "${BOLD}${GREEN} 🎉 INSTALASI OPA-LTE2PROXY SELESAI!${RESET}"
echo -e "${BOLD}${GREEN}================================================================${RESET}"
echo -e "${CYAN}SOCKS5 Proxy     :${RESET} 127.0.0.1:10800"
echo -e "${CYAN}REST Control API :${RESET} http://127.0.0.1:10808"
echo -e "${CYAN}Perintah Praktis :${RESET}"
echo -e "  • ${BOLD}olp setup${RESET}        - Wizard interaktif scan hardware & jaringan"
echo -e "  • ${BOLD}olp status${RESET}       - Cek operator, sinyal, dan IP aktif"
echo -e "  • ${BOLD}olp rotate${RESET}       - Ganti IP publik via Airplane Mode"
echo -e "  • ${BOLD}olp ip${RESET}           - Lihat IP seluler aktif saat ini"
echo -e "  • ${BOLD}olp rotate --force${RESET} - Paksa ganti IP meski server mode failover"
echo -e "${BOLD}${GREEN}================================================================${RESET}\n"
