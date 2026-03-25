# Threat Model

## Scope

Nexus Security handles:

- User auth and session workflows
- File or URL submission data
- Bounty/reward decision logic
- Blockchain-connected value transfer paths

## High-Value Assets

- JWT and API secrets
- Database records (users, bounties, submissions, payouts)
- Contract private keys and RPC credentials
- Submission artifacts and derived analysis output

## Primary Threat Actors

- External attackers targeting auth, APIs, and infra.
- Malicious insiders or compromised operator accounts.
- Abusive platform users gaming payout/reputation logic.

## Key Attack Surfaces

### 1. Authentication and Authorization

Risks:
- Credential stuffing and brute-force attempts.
- Token replay or weak secret handling.

Controls:
- Strong JWT secret management and rotation.
- Rate limiting and lockout policies.
- Route-level auth enforcement for protected endpoints.

### 2. API Input Handling

Risks:
- Injection and malformed payload abuse.
- Oversized payloads and parser pressure.

Controls:
- Input validation at handler boundaries.
- Payload limits and request timeout constraints.
- Structured error handling without leaking internals.

### 3. Submission and Analysis Pipeline

Risks:
- Malicious artifacts targeting scanning infra.
- Sandbox breakout or unsafe execution paths.

Controls:
- Isolation boundaries for analysis execution.
- Strict scanner timeouts and concurrency controls.
- Controlled storage and sanitized file handling.

### 4. Data Layer

Risks:
- Unauthorized reads/writes.
- Data corruption or loss during operational changes.

Controls:
- Least-privilege DB credentials.
- Backup and restore runbooks (`scripts/maintenance/*`).
- Migration discipline and rollback planning.

### 5. Blockchain Integration

Risks:
- Contract misconfiguration or wrong address usage.
- Private key exposure.

Controls:
- Environment-specific config templates under `config/`.
- Secret manager usage for keys (never in repo).
- Pre-deploy contract/address validation in staging first.

## Recommended Ongoing Controls

- Run `scripts/testing/security-scan.sh` in CI.
- Use dependency and image scanning before release.
- Apply patch and secret rotation cadences.
- Keep incident response procedures current.
