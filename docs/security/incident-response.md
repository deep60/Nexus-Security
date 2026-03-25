# Incident Response Playbook

## Severity Levels

- P1: Active compromise or financial risk.
- P2: Major degradation or high-confidence breach indicator.
- P3: Contained issue with limited blast radius.
- P4: Low-risk or informational finding.

## 1. Detection and Triage

1. Capture alert source and timestamp.
2. Identify impacted service(s) and data scope.
3. Assign severity and incident commander.
4. Open incident channel and record timeline.

## 2. Immediate Containment

Examples:

- Disable affected route or service deployment.
- Block abusive origins/IPs at ingress.
- Rotate impacted secrets.
- Pause unsafe rollout and freeze deploy pipeline.

## 3. Investigation

- Collect logs from API gateway and affected services.
- Validate auth events and suspicious token patterns.
- Inspect DB/Redis activity for abuse signatures.
- Confirm whether blockchain credentials were exposed.

## 4. Recovery

- Deploy fixed build.
- Restore data from known-good backups if needed.
- Re-run `scripts/deployment/health-check.sh`.
- Monitor for recurrence before closing incident.

## 5. Post-Incident Actions

- Produce incident report with timeline and root cause.
- Add preventive controls/tests.
- Update docs and runbooks.
- Share action items with owners and due dates.

## Useful Commands

```bash
# Health check after remediation
scripts/deployment/health-check.sh

# Backup before risky hotfix work
scripts/maintenance/backup.sh

# Controlled rollback
scripts/deployment/rollback.sh --to <git-ref>
```
