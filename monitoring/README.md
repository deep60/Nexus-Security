# Observability & Backups

Both stacks are **opt-in via compose profiles** so they don't affect CI or a plain
`docker compose up`. On the VM they're enabled with `COMPOSE_PROFILES=monitoring,backup`.

## Observability (profile: `monitoring`)

- **Prometheus** scrapes every backend service's `/metrics` (`verdyx_service_up`,
  `verdyx_http_requests_total`, `verdyx_http_errors_total`, uptime, build info) every
  15s and evaluates the alert rules in `prometheus/alerts.yml`.
- **Grafana** serves the provisioned *Verdyx — Service Overview* dashboard.

Both bind to **127.0.0.1 only** — no new public attack surface. View them over an SSH tunnel:

```sh
ssh -L 3001:localhost:3001 -L 9090:localhost:9090 -i ~/.ssh/verdyx-prod.pem ubuntu@<VM_IP>
# Grafana:    http://localhost:3001  (admin / $GRAFANA_ADMIN_PASSWORD)
# Prometheus: http://localhost:9090  (/alerts shows firing rules)
```

### Alert delivery (last mile — needs your channel)
Prometheus evaluates the rules and shows them at `/alerts`, but delivery needs a contact
point. Easiest: **Grafana → Alerting → Contact points** → add a Slack/Discord/email/Telegram
webhook, then a notification policy. (Or add Alertmanager wired to the same webhook.)

## Backups (profile: `backup`)

`db-backup` runs `pg_dumpall` of the whole cluster nightly (gzipped) into the `pg_backups`
volume with `BACKUP_RETENTION_DAYS` (default 7) retention. Protects against bad migrations,
accidental drops, and logical corruption.

Restore:
```sh
gunzip -c /path/to/verdyx-YYYYMMDD-HHMMSS.sql.gz | \
  docker exec -i verdyx-postgres psql -U verdyx_user -d postgres
```

### Off-site (protects against VM/disk loss)
Local backups don't survive a VM/disk failure. Sync the volume off-box — e.g. a host cron:
```sh
# nightly, after the container's dump window
0 4 * * *  docker run --rm -v verdyx_pg_backups:/b:ro amazon/aws-cli \
             s3 sync /b s3://<your-bucket>/verdyx-db/ --storage-class STANDARD_IA
```
(Or `mc mirror` to the on-VM MinIO for a quick copy — but keep at least one copy off the VM.)
