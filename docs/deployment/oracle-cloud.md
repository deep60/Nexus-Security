# Deploying Verdyx on Oracle Cloud "Always Free" (ARM)

This runs the full stack **for $0** on an Oracle Cloud Ampere A1 (ARM) VM. The
Always-Free shape gives up to **4 OCPU / 24 GB RAM**, which is plenty for the
whole compose stack (10 Rust services + Postgres/Redis/MinIO/ClamAV + frontend).

TLS is handled in-repo by the Caddy `edge` profile (automatic Let's Encrypt) —
no host nginx needed.

---

## 1. Create the VM

1. Sign up at <https://cloud.oracle.com> (needs a card for verification; Always-Free
   resources are never charged).
2. **Compute → Instances → Create instance**:
   - **Image:** Canonical Ubuntu 24.04 (**aarch64/ARM**).
   - **Shape:** `VM.Standard.A1.Flex` → set **4 OCPUs / 24 GB** (all within Always-Free).
   - **Boot volume:** 100–200 GB (Always-Free allows up to 200 GB total).
   - Add your **SSH public key**.
3. If you get **"Out of capacity"** for A1 (common in busy regions), retry in a
   different Availability Domain or region, or lower to 1 OCPU / 6 GB and retry.
4. Note the instance's **public IPv4**.

> Tip: to keep the IP stable, reserve it — **Networking → Reserved public IPs** →
> assign it to the instance. Otherwise the IP can change on stop/start (and you'd
> re-point DuckDNS each time).

## 2. Open the ports (TWO layers — this is the #1 gotcha)

Oracle blocks traffic at both the cloud firewall **and** inside the Ubuntu image.

**a) VCN Security List (cloud):** Networking → your VCN → Security Lists →
default → **Add Ingress Rules** (Source `0.0.0.0/0`, TCP):
- port **80**, port **443**  (22 is already open)

**b) Ubuntu host firewall:** the Oracle Ubuntu image ships restrictive iptables
that drop 80/443. SSH in and allow them:

```sh
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 80  -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 443 -j ACCEPT
sudo netfilter-persistent save    # persist across reboots
```

## 3. Install Docker

```sh
sudo apt-get update && sudo apt-get install -y ca-certificates curl git
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker "$USER" && newgrp docker   # run docker without sudo
docker compose version                             # sanity check
```

## 4. Point DuckDNS at the new IP

The old server is gone, so update the record (and add an auto-updater so a
changed IP self-heals):

```sh
# one-off update (replace TOKEN + IP)
curl "https://www.duckdns.org/update?domains=verdyx&token=YOUR_DUCKDNS_TOKEN&ip=THE_NEW_PUBLIC_IP"

# keep it current every 5 min
( crontab -l 2>/dev/null; \
  echo "*/5 * * * * curl -s 'https://www.duckdns.org/update?domains=verdyx&token=YOUR_DUCKDNS_TOKEN&ip=' >/dev/null" ) | crontab -
```

Confirm it resolves to the new IP before continuing: `dig +short verdyx.duckdns.org`.

## 5. Get the arm64 images

The images must be **arm64**. Two ways:

- **Build in CI (recommended):** GitHub → Actions → **Build images (multi-arch)**
  → *Run workflow* (defaults to `linux/amd64,linux/arm64`). Pushes multi-arch
  `:latest` to GHCR. Then on the VM just `docker compose pull`.
- **Build on the VM:** `docker compose build` (native arm64, no emulation) — slower
  first time but needs no registry.

If the GHCR packages are private, log in on the VM first:
`echo <GH_PAT> | docker login ghcr.io -u <github-user> --password-stdin`.

## 6. Configure and launch

```sh
git clone https://github.com/deep60/Verdyx.git && cd Verdyx
cp .env.example .env
```

Edit `.env` — set real values (restore from your backup, NOT the examples):
`POSTGRES_PASSWORD`, `REDIS_PASSWORD`, `JWT_SECRET` (≥32 chars), the `MINIO_*`
creds, `DOMAIN=verdyx.duckdns.org`, `ACME_EMAIL=you@example.com`. Leave the
blockchain vars/treasury key as-is until you deploy contracts to a funded chain
(the app degrades gracefully without them).

Bring it up — app + observability + backups + Caddy TLS:

```sh
export COMPOSE_PROFILES=monitoring,backup,edge
docker compose pull          # skip if you built locally
docker compose up -d
```

Caddy will fetch a Let's Encrypt cert for `verdyx.duckdns.org` automatically
(ports 80/443 must be reachable — steps 2 & 4).

## 7. Verify

```sh
docker compose ps                       # all healthy?
curl -I https://verdyx.duckdns.org       # 200/301 from Caddy
curl https://verdyx.duckdns.org/api/v1/health
```

Open <https://verdyx.duckdns.org> in a browser.

---

## Notes / troubleshooting

- **ClamAV** is memory-hungry (loads a ~1 GB signature DB) and slow to become
  healthy on first boot (`start_period: 300s`) — give it a few minutes.
- **Alerts:** to get Slack delivery, drop your webhook in
  `monitoring/alertmanager/secrets/slack_api_url` (see `monitoring/README.md`).
- **Off-site backups:** set `BACKUP_S3_BUCKET` in `.env` to push verified dumps
  off the VM (you can point it at MinIO or any S3).
- **Grafana/Prometheus** bind to `127.0.0.1` only — reach them via SSH tunnel,
  never expose them publicly.
- **Auto-deploy:** `watchtower` in the compose file will pull new `:latest` images
  from GHCR every 5 min, so a CI multi-arch build refreshes the VM automatically.
