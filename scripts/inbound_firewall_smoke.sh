#!/usr/bin/env bash
# QUEUE #252 — listening socket + iptables INPUT DROP ≠ non-listening placeholder.
# Requires Docker with --cap-add=NET_ADMIN. Prints markers consumed by Rust tests.
set -euo pipefail

IMAGE="${AIRA_INBOUND_FIREWALL_IMAGE:-alpine:3.20}"
PORT="${AIRA_INBOUND_FIREWALL_PORT:-49157}"

docker run --rm --cap-add=NET_ADMIN "$IMAGE" sh -c "
set -e
apk add --no-cache python3 iptables iproute2 >/dev/null
python3 - <<'PY'
import socket, subprocess, time

PORT = ${PORT}
KIND = 'firewall-input-drop'

s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(('0.0.0.0', PORT))
s.listen(1)
print('LISTEN_OK', flush=True)

# Control: prove the socket is listening (≠ non-listening placeholder).
c = socket.create_connection(('127.0.0.1', PORT), timeout=2)
c.close()
conn, _ = s.accept()
conn.close()
print('CONTROL_CONNECT_OK', flush=True)

subprocess.check_call(['iptables', '-A', 'INPUT', '-p', 'tcp', '--dport', str(PORT), '-j', 'DROP'])

blocked = False
try:
    socket.create_connection(('127.0.0.1', PORT), timeout=1)
except OSError:
    blocked = True
if not blocked:
    raise SystemExit('expected firewall block after INPUT DROP')
print('FIREWALL_BLOCK_OK', flush=True)

ss = subprocess.check_output(['ss', '-ltn'], text=True)
if str(PORT) not in ss:
    raise SystemExit('listener disappeared; not a firewall proof')
print('STILL_LISTENING_OK', flush=True)
print(KIND, flush=True)
# Keep process alive briefly so operators can inspect; exit success.
time.sleep(0.05)
s.close()
PY
"
