# Production Deployment Runbook

Use this runbook for controlled releases.

## 1. Pre-Deploy Checklist

- All tests passing: `scripts/testing/run-tests.sh`
- Security scan reviewed: `scripts/testing/security-scan.sh`
- Config templates updated for production (`config/production/*`)
- Secrets rotated and present in secret manager
- Rollback target identified (git tag/commit)

## 2. Deploy

```bash
scripts/deployment/deploy.sh \
  --compose-file docker-compose.yml \
  --env-file .env
```

Deploy selected services only:

```bash
scripts/deployment/deploy.sh --services "api-gateway user-service"
```

## 3. Post-Deploy Validation

```bash
scripts/deployment/health-check.sh --compose-file docker-compose.yml
```

Review logs:

```bash
docker compose logs --tail=200 api-gateway user-service analysis-engine
```

## 4. Backup and Restore

Create backup:

```bash
scripts/maintenance/backup.sh
```

Restore from backup folder:

```bash
scripts/maintenance/restore.sh --backup backups/<timestamp>
```

## 5. Rollback

Rollback to a known-good git ref:

```bash
scripts/deployment/rollback.sh --to <git-ref>
```

Optional scoped rollback:

```bash
scripts/deployment/rollback.sh --to <git-ref> --services "api-gateway user-service"
```

## 6. Operational Notes

- Keep deploys small and observable.
- Prefer staged rollouts where possible.
- Run health checks before and after every change window.
