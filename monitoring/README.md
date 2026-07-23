# Observability & Backups

Both stacks are **opt-in via compose profiles** so they don't affect CI or a plain
`docker compose up`. On the VM they're enabled with `COMPOSE_PROFILES=monitoring,backup`.

## Observability (profile: `monitoring`)

- **Prometheus** scrapes every backend service's `/metrics` (`verdyx_service_up`,
  `verdyx_http_requests_total`, `verdyx_http_errors_total`, uptime, build info) every
  15s and evaluates the alert rules in `prometheus/alerts.yml`, firing them at Alertmanager.
- **Alertmanager** groups/dedups/routes alerts and delivers them (Slack) — see below.
- **Grafana** serves the provisioned *Verdyx — Service Overview* dashboard.

Both bind to **127.0.0.1 only** — no new public attack surface. View them over an SSH tunnel:

```sh
ssh -L 3001:localhost:3001 -L 9090:localhost:9090 -i ~/.ssh/verdyx-prod.pem ubuntu@<VM_IP>
# Grafana:    http://localhost:3001  (admin / $GRAFANA_ADMIN_PASSWORD)
# Prometheus: http://localhost:9090  (/alerts shows firing rules)
```

### Alert delivery (Alertmanager)
`alertmanager` receives fired alerts and routes them by severity (`critical` →
`#verdyx-critical`, everything else → `#verdyx-alerts`) with grouping, dedup, and
inhibition — see `alertmanager/alertmanager.yml`. It binds to `127.0.0.1:9093`.

The Slack webhook is a secret and is **not** committed; Alertmanager reads it from a
gitignored file at notify time (it still starts if the file is absent, just logs a
delivery error until you add it):

```sh
echo -n 'https://hooks.slack.com/services/XXX/YYY/ZZZ' \
  > monitoring/alertmanager/secrets/slack_api_url
docker compose --profile monitoring restart alertmanager   # or up -d
```

Prefer email? Replace the `slack_configs` in a receiver with `email_configs`
(`smtp_smarthost`, `smtp_from`, `smtp_auth_password_file: /etc/alertmanager/secrets/smtp_password`).
Validate any change with `amtool check-config alertmanager/alertmanager.yml`.

## Backups (profile: `backup`)

`db-backup` runs `pg_dumpall` of the whole cluster nightly (gzipped) into the `pg_backups`
volume with `BACKUP_RETENTION_DAYS` (default 7) retention. Protects against bad migrations,
accidental drops, and logical corruption.

**Every dump is verified before it's kept** (`scripts/backup/pg-backup.sh`): gzip integrity
plus a content check that the `PostgreSQL database cluster dump` header and `CREATE DATABASE
verdyx*` statements are present. A truncated or empty dump is deleted and the cycle fails
loudly rather than silently retaining a useless file — *an untested backup is not a backup*.

Restore:
```sh
gunzip -c /path/to/verdyx-YYYYMMDD-HHMMSS.sql.gz | \
  docker exec -i verdyx-postgres psql -U verdyx_user -d postgres
```

### Off-site (protects against VM/disk loss)
Local backups don't survive a VM/disk failure. Set `BACKUP_S3_BUCKET` (and reuse the stack's
`AWS_*` creds / `AWS_ENDPOINT_URL` for MinIO) and each **verified** dump is auto-pushed to
S3/MinIO by the backup container — no separate host cron needed:
```sh
BACKUP_S3_BUCKET=verdyx-backups        # enables off-site upload
BACKUP_S3_PREFIX=pg                     # key prefix (default: pg)
BACKUP_S3_ENDPOINT=                     # blank = real AWS S3; set for MinIO
```
Keep at least one copy on a **different host/account** than the VM. Gold standard beyond
integrity checks: periodically restore the latest dump into a throwaway Postgres and run a
smoke query — schedule this as a monthly DR drill.
