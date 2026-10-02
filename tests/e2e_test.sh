#!/usr/bin/env bash
# ==============================================================================
# OPA-LTE2PROXY End-to-End (E2E) Test Suite
# ==============================================================================
set -euo pipefail

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
RED="\033[31m"
RESET="\033[0m"

pass() { echo -e "  ${GREEN}[PASS]${RESET} $1"; }
fail() { echo -e "  ${RED}[FAIL]${RESET} $1"; exit 1; }

echo -e "\n${BOLD}${CYAN}================================================================${RESET}"
echo -e "${BOLD}${CYAN} 🧪 RUNNING OPA-LTE2PROXY END-TO-END (E2E) TESTS${RESET}"
echo -e "${BOLD}${CYAN}================================================================${RESET}\n"

# Test 1: Binary Existence & Version
echo -e "${BOLD}1. Memeriksa Binary & Versi:${RESET}"
olp --version > /dev/null && pass "Binary /usr/local/bin/opa-lte2proxy (olp) aktif dan merespons --version" || fail "Binary olp tidak ditemukan atau gagal dijalankan"

# Test 2: Daemon Process & Port Listening
echo -e "\n${BOLD}2. Memeriksa Status Service & Port Listening:${RESET}"
nc -z -w2 127.0.0.1 10800 && pass "SOCKS5 Proxy aktif listening di 127.0.0.1:10800" || fail "Port 10800 SOCKS5 tidak terbuka"
nc -z -w2 127.0.0.1 10808 && pass "REST Control API aktif listening di 127.0.0.1:10808" || fail "Port 10808 API tidak terbuka"

# Test 3: REST API /status Endpoint
echo -e "\n${BOLD}3. Menguji REST API /status Endpoint:${RESET}"
STATUS_JSON=$(curl -s --max-time 5 http://127.0.0.1:10808/status)
if echo "$STATUS_JSON" | grep -q '"status":"online"'; then
  OPERATOR=$(echo "$STATUS_JSON" | grep -o '"operator":"[^"]*"' | cut -d: -f2 | tr -d '"')
  pass "REST API /status valid (Status: online, Operator: $OPERATOR)"
else
  fail "REST API /status mengembalikan payload tidak valid: $STATUS_JSON"
fi

# Test 4: REST API /ip Endpoint
echo -e "\n${BOLD}4. Menguji REST API /ip Endpoint:${RESET}"
CURRENT_IP=$(curl -s --max-time 5 http://127.0.0.1:10808/ip)
if [[ "$CURRENT_IP" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  pass "IP seluler modem aktif: $CURRENT_IP"
else
  fail "Gagal mendapatkan IP dari /ip: $CURRENT_IP"
fi

# Test 5: SOCKS5 Handshake & Connect (Tanpa Auth)
echo -e "\n${BOLD}5. Menguji SOCKS5 Handshake (Direct/No Auth):${RESET}"
HTTP_RESP=$(curl -s --socks5 127.0.0.1:10800 --max-time 8 http://192.168.200.1/ || true)
if echo "$HTTP_RESP" | grep -qi "4G UFI"; then
  pass "Koneksi via SOCKS5 (Direct) berhasil menembus portal modem"
else
  fail "Koneksi SOCKS5 direct gagal merespons"
fi

# Test 6: SOCKS5 Sticky Session Handshake (Dengan Kredensial RFC 1929)
echo -e "\n${BOLD}6. Menguji SOCKS5 Sticky Session Auth (RFC 1929):${RESET}"
AUTH_RESP=$(curl -s --socks5 127.0.0.1:10800 --proxy-user "user-session_test99:secret" --max-time 8 http://192.168.200.1/ || true)
if echo "$AUTH_RESP" | grep -qi "4G UFI"; then
  pass "SOCKS5 Sticky Session kredensial berhasil dinegosiasikan"
else
  fail "SOCKS5 Sticky Session kredensial gagal"
fi

# Test 7: CLI Subcommands (olp status & olp ip)
echo -e "\n${BOLD}7. Menguji CLI Subcommands:${RESET}"
CLI_STATUS=$(olp status)
if echo "$CLI_STATUS" | grep -q '"modem"'; then
  pass "CLI 'olp status' berhasil memformat telemetri"
else
  fail "CLI 'olp status' gagal"
fi

# Test 8: Linux Routing Table 200 Isolation
echo -e "\n${BOLD}8. Memeriksa Policy Routing Table 200:${RESET}"
if ip rule show | grep -q "from 192.168.200.174 lookup 200"; then
  pass "Rule 'ip rule from 192.168.200.174 lookup 200' aktif terdaftar di kernel"
else
  fail "Rule table 200 tidak ditemukan di kernel"
fi

echo -e "\n${BOLD}${GREEN}================================================================${RESET}"
echo -e "${BOLD}${GREEN} ✅ SELURUH TEST E2E BERHASIL LULUS (ALL TESTS PASSED)!${RESET}"
echo -e "${BOLD}${GREEN}================================================================${RESET}\n"
