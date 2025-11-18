# Incident Response Plan

This document outlines procedures for handling security incidents at Nexus Security.

## Incident Classification

### Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| P1 - Critical | Active breach, funds at risk | 15 minutes | Smart contract exploit, data exfiltration |
| P2 - High | Potential breach, service down | 1 hour | Auth bypass, DoS attack |
| P3 - Medium | Limited impact, contained | 4 hours | Single account compromise |
| P4 - Low | Minor issue, no data loss | 24 hours | Failed attack attempts |

## Response Team

### Roles

- **Incident Commander**: Coordinates response, makes decisions
- **Technical Lead**: Leads technical investigation
- **Communications Lead**: Handles internal/external comms
- **Legal/Compliance**: Advises on obligations

### On-Call Rotation

- Primary: Check PagerDuty schedule
- Secondary: Backup responder
- Escalation: CTO ’ CEO

## Response Phases

### 1. Detection

**Sources**:

- Monitoring alerts (Prometheus/Grafana)
- User reports
- Security scanning
- External notification

**Initial Actions**:

```
1. Acknowledge alert
2. Assess severity level
3. Page appropriate responders
4. Create incident channel (#incident-YYYY-MM-DD)
```

### 2. Containment

**Immediate Actions by Severity**:

#### P1 - Critical

```bash
# Pause smart contracts (if applicable)
cast send $CONTRACT "pause()" --private-key $KEY

# Block suspicious IPs
kubectl exec -it nginx -- nginx -s reload

# Revoke compromised credentials
./scripts/revoke-all-tokens.sh

# Take affected services offline
kubectl scale deployment api-gateway --replicas=0
```

#### P2 - High

```bash
# Enable enhanced logging
kubectl set env deployment/api-gateway LOG_LEVEL=debug

# Increase rate limits
kubectl apply -f emergency-rate-limits.yaml

# Block specific users/IPs
redis-cli SADD blocked_ips "x.x.x.x"
```

#### P3/P4

- Document indicators
- Monitor closely
- Prepare containment if escalates

### 3. Eradication

**Objectives**:

- Remove attacker access
- Patch vulnerabilities
- Clean compromised systems

**Actions**:

```bash
# Rotate all secrets
./scripts/rotate-secrets.sh

# Deploy security patches
kubectl set image deployment/api-gateway api-gateway=nexus/api-gateway:patched

# Reset compromised accounts
./scripts/force-password-reset.sh --users affected_users.txt
```

### 4. Recovery

**Steps**:

1. Verify systems are clean
2. Restore from known-good backups
3. Gradually restore services
4. Monitor for re-compromise

```bash
# Restore database from backup
aws rds restore-db-instance-from-db-snapshot \
  --db-instance-identifier nexus-postgres-recovered \
  --db-snapshot-identifier pre-incident-snapshot

# Gradual traffic restoration
kubectl scale deployment api-gateway --replicas=1
# Monitor...
kubectl scale deployment api-gateway --replicas=3
```

### 5. Post-Incident

**Timeline**: Within 48 hours

#### Incident Report Template

```markdown
## Incident Report: [Title]

**Date**: YYYY-MM-DD
**Severity**: P1/P2/P3/P4
**Duration**: X hours
**Commander**: Name

### Summary
Brief description of what happened.

### Timeline
- HH:MM - Event detected
- HH:MM - Team assembled
- HH:MM - Containment achieved
- HH:MM - Eradication complete
- HH:MM - Recovery complete

### Impact
- Users affected: X
- Data exposed: Description
- Financial impact: $X

### Root Cause
Technical explanation of vulnerability.

### Resolution
How the issue was fixed.

### Lessons Learned
What we'll do better.

### Action Items
- [ ] Task 1 - Owner - Due date
- [ ] Task 2 - Owner - Due date
```

## Communication

### Internal Communication

**Channels**:

- Slack: #incident-response (real-time)
- Email: security-team@nexus-security.com

**Updates**:

- P1: Every 30 minutes
- P2: Every hour
- P3/P4: At resolution

### External Communication

#### User Notification (if required)

```
Subject: Security Notice - Action Required

Dear [User],

We detected unauthorized access to your Nexus Security account on [date].
We have secured your account and recommend you:

1. Reset your password immediately
2. Review recent account activity
3. Enable two-factor authentication

No funds were affected. We apologize for any inconvenience.

Questions? Contact security@nexus-security.com

Nexus Security Team
```

#### Public Disclosure

**Criteria for public disclosure**:

- User data was exposed
- Regulatory requirement
- Wide-scale impact

**Template**:

```
SECURITY INCIDENT NOTICE

Date: [Date]

Nexus Security identified and resolved a security incident on [date].
[Brief description of what occurred and impact].

What We Did:
- [Actions taken]

What You Should Do:
- [User actions]

We are committed to transparency and security. For questions,
contact security@nexus-security.com.
```

## Playbooks

### Compromised User Account

1. Disable account
2. Revoke all sessions
3. Check for unauthorized actions
4. Notify user via verified contact
5. Require password reset + 2FA

### Smart Contract Vulnerability

1. Pause contract if possible
2. Assess exploitability
3. Prepare patch
4. Deploy to testnet
5. Emergency audit
6. Deploy to mainnet
7. Resume contract

### DDoS Attack

1. Enable CDN DDoS protection
2. Implement aggressive rate limiting
3. Block attacking IPs/ASNs
4. Scale infrastructure
5. Contact ISP if needed

### Data Breach

1. Contain the breach
2. Assess data exposed
3. Notify legal/compliance
4. Preserve evidence
5. Notify affected users
6. Report to authorities (if required)

## Tools

### Investigation

- Log analysis: Grafana Loki
- Network: Wireshark, tcpdump
- Forensics: Velociraptor

### Communication

- Incident management: PagerDuty
- Chat: Slack
- Video: Zoom

### Recovery

- Backups: AWS Backup
- Config management: Terraform
- Secrets: AWS Secrets Manager

## Training

- Tabletop exercises: Quarterly
- Technical drills: Monthly
- New hire training: Onboarding

## Review

This plan is reviewed and updated:

- After every P1/P2 incident
- Quarterly at minimum
- When infrastructure changes significantly

## Contacts

**Internal**:

- Security Team: security@nexus-security.com
- On-call: See PagerDuty

**External**:

- Legal Counsel: [Contact]
- Cyber Insurance: [Contact]
- Law Enforcement: [Contact]
