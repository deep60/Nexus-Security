# Verdyx — Go-Live Runbook (compose-on-VM + managed RDS/ElastiCache)

This is the human/AWS side of the deploy automation. Architecture context is in
[`infrastructure/DEPLOYMENT.md`](../../infrastructure/DEPLOYMENT.md). Do the steps
in order. Everything is idempotent unless noted.

---

## Pre-flight fixes (do these first — they'll bite you otherwise)

These are existing inconsistencies in the repo. Fix before starting:

1. **RDS master username must match `POSTGRES_USER`.**
   `infrastructure/terraform/main.tf` creates the RDS instance with
   `username = "verdyx"`, but the rendered `.env` (Ansible `env.j2`) sets
   `POSTGRES_USER=verdyx_prod`. `ensure-databases.sh` connects as `POSTGRES_USER`,
   so they must be identical. **Pick one** and make both agree (simplest: set
   `env.j2` → `POSTGRES_USER=verdyx`, or set `main.tf` → `username = "verdyx_prod"`).

2. **Domain is `verdyx.io`, not `verdyx.com`.** Update:
   - `infrastructure/ansible/inventories/production.ini` → `verdyx_domain=verdyx.io`
   - `infrastructure/terraform/terraform.tfvars` → `domain_name = "verdyx.io"`

3. **Fix the repo URL + host in the Ansible inventory.**
   `production.ini` has placeholder `ansible_host=REPLACE_ME` and
   `verdyx_repo_url=git@github.com:verdyx/deep60.git`. Set the real VM IP/DNS and
   the correct repo URL (and add a deploy key on the VM if the repo is private).

---

## Prerequisites

- **Local tools:** `terraform >= 1.0`, `aws` CLI, `ansible` + `ansible-vault`, `gh` (optional).
- **AWS:** an account + credentials with rights to create S3, DynamoDB, VPC, RDS,
  ElastiCache (and EKS if you don't skip it). `aws configure` or export
  `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_REGION=us-east-1`.
- **A VM** (EC2 or any Ubuntu host) reachable by SSH, in/peered to the same VPC as
  RDS/ElastiCache (RDS lives in private subnets; the VM must be able to reach it).
- **Docker Hub** account (images are pushed there).

---

## Step 1 — Terraform remote state

The S3 state bucket + DynamoDB lock table must exist **before** `terraform init`.

```bash
cd infrastructure/terraform

# One-time, idempotent. Creates verdyx-terraform-state (S3) + verdyx-terraform-locks (DynamoDB).
./scripts/bootstrap-tf-state.sh
# (override defaults if needed: AWS_REGION=... BUCKET=... LOCK_TABLE=... ./scripts/...)

# Initialise against the S3 backend. If you had local state, answer "yes" to migrate.
terraform init
```

✅ Done when `terraform init` prints "Successfully configured the backend s3".

---

## Step 2 — Provision the data tier (RDS + ElastiCache)

```bash
cd infrastructure/terraform
cp terraform.tfvars.example terraform.tfvars      # then edit (region, domain_name=verdyx.io, sizes)

terraform plan     # review

# LAUNCH scope: skip EKS, provision only VPC + RDS + ElastiCache + S3.
terraform apply \
  -target=module.vpc \
  -target=module.rds \
  -target=module.redis \
  -target=aws_security_group.rds \
  -target=aws_security_group.redis \
  -target=aws_s3_bucket.uploads \
  -target=aws_s3_bucket_versioning.uploads \
  -target=aws_s3_bucket_server_side_encryption_configuration.uploads \
  -target=aws_s3_bucket_public_access_block.uploads
```

> To provision **everything including EKS** (Phase 2), just run `terraform apply`
> with no `-target`.

Grab the endpoints you'll need for Ansible:

```bash
terraform output rds_endpoint        # -> vault_rds_endpoint  (strip any :5432 suffix; host only)
terraform output redis_endpoint      # -> vault_elasticache_endpoint
terraform output uploads_bucket_name
```

> **RDS master password:** Terraform's `rds` module generates one unless you pass
> `manage_master_user_password`/`password`. Retrieve it (Secrets Manager or the
> module output) and use it as `vault_postgres_password`. Set the ElastiCache
> auth token as `vault_redis_password`.

✅ Done when `terraform output rds_endpoint` and `redis_endpoint` return hostnames.

---

## Step 3 — Secrets → render `/opt/verdyx/.env` via Ansible

```bash
cd infrastructure/ansible

# 1. Create the vault file from the example.
cp group_vars/all/vault.yml.example group_vars/all/vault.yml

# 2. Fill it in. Generate strong secrets:
#      openssl rand -hex 32
#    Paste the Terraform endpoints into:
#      vault_rds_endpoint:          <rds_endpoint host>
#      vault_elasticache_endpoint:  <redis_endpoint host>
#    Set vault_postgres_password / vault_redis_password to the RDS/ElastiCache
#    creds from Step 2. Paste deployed contract addresses + chain id.
#    NOTE: vault_treasury_private_key controls real funds — ideally reference a
#    KMS/HSM, do not paste a hot key if you can avoid it.
$EDITOR group_vars/all/vault.yml

# 3. Encrypt it (keep the vault password safe — you'll need it to run the playbook).
ansible-vault encrypt group_vars/all/vault.yml

# 4. Run the full host bootstrap. This installs Docker, clones the repo, renders
#    /opt/verdyx/.env (0600) from env.j2 + vault, sets up nginx + monitoring, and
#    brings the stack up via scripts/deployment/remote-deploy.sh (ensures the
#    per-service RDS databases, starts app services with --no-deps, health-checks).
ansible-playbook -i inventories/production.ini playbooks/site.yml --ask-vault-pass
```

✅ Done when the playbook finishes green and `/opt/verdyx/.env` exists on the VM
with the RDS/ElastiCache hosts (not `postgres`/`redis`).

---

## Step 4 — GitHub Actions secrets

Set these in **Repo → Settings → Secrets and variables → Actions**. The workflows
reference them by these exact names:

| Secret | Used by | Notes |
|--------|---------|-------|
| `SSH_PRIVATE_KEY` | deploy.yml | Private key whose public half is in the prod VM's `authorized_keys` |
| `SSH_USER` | deploy.yml | e.g. `verdyx` or `ubuntu` |
| `SSH_HOST` | deploy.yml | Prod VM IP/DNS |
| `STAGING_SSH_PRIVATE_KEY` | staging.yml | Staging VM key |
| `STAGING_SSH_USER` | staging.yml | |
| `STAGING_SSH_HOST` | staging.yml | Staging VM IP/DNS |
| `DOCKER_USERNAME` | deploy.yml, staging.yml | Docker Hub user (also the image namespace) |
| `DOCKER_PASSWORD` | deploy.yml, staging.yml | Docker Hub token/password |
| `SLACK_WEBHOOK` | deploy.yml | Deploy notifications |
| `SNYK_TOKEN` | frontend-ci.yml | Optional; scan is non-blocking without it |

CLI alternative:

```bash
gh secret set SSH_PRIVATE_KEY   < ~/.ssh/verdyx_deploy
gh secret set SSH_USER          --body "verdyx"
gh secret set SSH_HOST          --body "1.2.3.4"
gh secret set DOCKER_USERNAME   --body "yourdockeruser"
gh secret set DOCKER_PASSWORD   --body "dckr_pat_..."
gh secret set SLACK_WEBHOOK     --body "https://hooks.slack.com/..."
# ...and the STAGING_* trio
```

Also create a GitHub **Environment** named `production` (and `staging`) if you want
approval gates — the workflows reference `environment: production`/`staging`.

✅ Done when `gh secret list` shows all required names.

---

## Step 5 — VM prerequisites

The Ansible `docker` role installs Docker, so if you ran Step 3 that's covered.
Confirm on the VM:

```bash
docker --version && docker compose version    # Docker + compose v2
psql --version || echo "no psql — ensure-databases.sh will use a dockerized client (fine)"
```

- `postgresql-client` is **optional**: `ensure-databases.sh` falls back to a
  throwaway `postgres:16-alpine` container for `psql` if none is on the host.
- The VM's outbound security group must allow it to reach RDS (5432) and
  ElastiCache (6379). The Terraform SGs allow the EKS SG by default — **add the
  VM's SG/CIDR** to `aws_security_group.rds`/`redis` ingress, or the app can't
  connect. (This is the one Terraform edit the compose-on-VM path needs.)

✅ Done when, from the VM: `nc -zv <rds_endpoint> 5432` and `nc -zv <redis_endpoint> 6379` succeed.

---

## Step 6 — First deploy & verify (staging first!)

```bash
# Trigger staging by pushing to develop (staging.yml runs on: develop).
git push origin develop
```

Watch the Actions run. On the staging VM, the deploy calls `remote-deploy.sh`,
which prints its progress:

```
==> [deploy] ensuring databases ...        # ensure-databases.sh creates verdyx_* DBs on RDS
==> [deploy] compose up --no-deps ...       # app services only; each self-migrates at startup
==> [deploy] healthy.                        # or: rolls back to the previous SHA on failure
```

Verify:

```bash
curl -fsS https://staging.verdyx.io/api/v1/health      # 200
# on the VM:
cd /opt/verdyx && bash scripts/maintenance/check-migrations.sh   # (adapt for RDS host if needed)
```

Only after staging is green, promote to production:

```bash
git checkout main && git merge --ff-only develop && git push origin main
```

`deploy.yml` runs the same flow against the prod VM, then `curl https://verdyx.io/api/v1/health`.

### If a deploy fails
`remote-deploy.sh` **auto-rolls-back** to the previous good revision and restarts,
then exits non-zero (Actions shows red). Investigate, fix, redeploy. For a manual
rollback to any tag: `scripts/deployment/rollback.sh --to <git-ref>`.

---

## Quick reference — what each new script does

| Script | Runs where | Purpose |
|--------|-----------|---------|
| `infrastructure/terraform/scripts/bootstrap-tf-state.sh` | your laptop | Create S3 state bucket + DynamoDB lock (once) |
| `scripts/deployment/ensure-databases.sh` | the VM | Create per-service DBs + extensions on RDS |
| `scripts/deployment/remote-deploy.sh` | the VM | Pull → ensure DBs → compose up → health check → rollback on fail |
| `scripts/deployment/rollback.sh` | the VM | Manual rollback to a git ref |
| `scripts/maintenance/generate-prod-secrets.sh` | your laptop | Generate strong secret values for the vault |
