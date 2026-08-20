#!/usr/bin/env bash
#
# Prepares a fresh Google Cloud e2-micro (always-free) VM to run Verdyx.
# Idempotent — safe to re-run after a reboot or a failed attempt.
#
#   curl -fsSL https://raw.githubusercontent.com/deep60/Verdyx/main/scripts/deployment/gcp-bootstrap.sh | sudo bash
#
# or, if the repo is already cloned:  sudo bash scripts/deployment/gcp-bootstrap.sh
#
# Does NOT start the stack — it only makes the box capable of running it. You
# still need to write /opt/verdyx/.env before the first `up`. See
# docs/deployment/gcp-free-tier.md.

set -euo pipefail

APP_DIR=${APP_DIR:-/opt/verdyx}
REPO_URL=${REPO_URL:-https://github.com/deep60/Verdyx.git}
SWAP_SIZE=${SWAP_SIZE:-3G}
SWAP_FILE=/swapfile

log() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
warn() { printf '\033[1;33m[warn] %s\033[0m\n' "$*" >&2; }

if [[ $EUID -ne 0 ]]; then
	echo "Run as root (sudo)." >&2
	exit 1
fi

# ── Swap ──────────────────────────────────────────────────────────────────
# This is the single most important step. The stack idles around 700 MB on a
# 1 GB box, so a routine traffic spike would otherwise trigger the kernel OOM
# killer and take out a container. Swap converts that hard failure into a slow
# request, which on a portfolio deployment is a good trade.
log "Configuring ${SWAP_SIZE} swap"
if swapon --show | grep -q "$SWAP_FILE"; then
	echo "swap already active, skipping"
else
	if [[ ! -f $SWAP_FILE ]]; then
		# fallocate can produce a sparse file that swapon rejects on some
		# filesystems; dd is slower but always yields a usable swapfile.
		fallocate -l "$SWAP_SIZE" "$SWAP_FILE" 2>/dev/null ||
			dd if=/dev/zero of="$SWAP_FILE" bs=1M count=$((${SWAP_SIZE%G} * 1024)) status=progress
	fi
	chmod 600 "$SWAP_FILE"
	mkswap "$SWAP_FILE"
	swapon "$SWAP_FILE"
	grep -q "$SWAP_FILE" /etc/fstab || echo "$SWAP_FILE none swap sw 0 0" >>/etc/fstab
fi

# Default swappiness of 60 thrashes a small box. 10 keeps hot pages resident
# and uses swap only as the overflow it is meant to be here.
log "Tuning kernel for a memory-constrained host"
cat >/etc/sysctl.d/99-verdyx.conf <<'EOF'
vm.swappiness=10
vm.vfs_cache_pressure=50
# 9 services each holding connections; the default ephemeral range and
# somaxconn are tight once Caddy and the gateway are both proxying.
net.core.somaxconn=1024
net.ipv4.tcp_max_syn_backlog=2048
EOF
sysctl --quiet --load /etc/sysctl.d/99-verdyx.conf

# ── Packages ──────────────────────────────────────────────────────────────
log "Installing base packages"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq ca-certificates curl git jq postgresql-client >/dev/null

# ── Docker ────────────────────────────────────────────────────────────────
if command -v docker >/dev/null 2>&1; then
	log "Docker already installed ($(docker --version))"
else
	log "Installing Docker"
	curl -fsSL https://get.docker.com | sh
fi

# Cap the journal and container logs. A 30 GB disk fills faster than you would
# think with 9 services logging, and a full disk wedges Docker itself.
log "Capping log growth"
mkdir -p /etc/docker
if [[ -f /etc/docker/daemon.json ]] && ! jq -e '."log-opts"' /etc/docker/daemon.json >/dev/null 2>&1; then
	warn "/etc/docker/daemon.json exists without log-opts; merging"
	tmp=$(mktemp)
	jq '. + {"log-driver":"json-file","log-opts":{"max-size":"10m","max-file":"3"}}' \
		/etc/docker/daemon.json >"$tmp" && mv "$tmp" /etc/docker/daemon.json
elif [[ ! -f /etc/docker/daemon.json ]]; then
	cat >/etc/docker/daemon.json <<'EOF'
{
  "log-driver": "json-file",
  "log-opts": { "max-size": "10m", "max-file": "3" }
}
EOF
fi
systemctl restart docker
systemctl enable docker >/dev/null 2>&1 || true

mkdir -p /etc/systemd/journald.conf.d
cat >/etc/systemd/journald.conf.d/99-verdyx.conf <<'EOF'
[Journal]
SystemMaxUse=200M
EOF
systemctl restart systemd-journald || true

# Let the login user run docker without sudo.
for u in ${SUDO_USER:-} ubuntu debian; do
	if [[ -n $u ]] && id "$u" >/dev/null 2>&1; then
		usermod -aG docker "$u" && log "Added '$u' to the docker group (re-login to take effect)"
		break
	fi
done

# ── Application directory ────────────────────────────────────────────────
log "Preparing ${APP_DIR}"
if [[ -d $APP_DIR/.git ]]; then
	git -C "$APP_DIR" pull --ff-only || warn "git pull failed; leaving working tree as-is"
else
	mkdir -p "$(dirname "$APP_DIR")"
	git clone --depth 1 "$REPO_URL" "$APP_DIR"
fi

# ── Firewall ─────────────────────────────────────────────────────────────
# Unlike Oracle's images, GCE Debian/Ubuntu images do not ship restrictive host
# iptables — inbound filtering happens in the VPC firewall instead, which is
# configured with `gcloud` from your laptop (see the runbook). Only interfere
# if ufw is present AND active, which is not the default.
if command -v ufw >/dev/null 2>&1 && ufw status 2>/dev/null | grep -q '^Status: active'; then
	log "ufw is active — allowing 80/443"
	ufw allow 80/tcp >/dev/null
	ufw allow 443/tcp >/dev/null
fi

log "Bootstrap complete"
cat <<EOF

Memory now available:
$(free -h)

Next steps:
  1. cd ${APP_DIR}
  2. sudo cp .env.gcp.example .env && sudo chmod 600 .env
  3. sudo nano .env          # fill every <<FILL>> value
  4. sudo docker compose -f docker-compose.gcp.yml up -d

Then watch it come up:
     sudo docker compose -f docker-compose.gcp.yml ps
     sudo docker stats --no-stream
EOF
