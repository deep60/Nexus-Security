# Contributing

## 1. Setup

```bash
git clone <your-fork-url>
cd Verdyx
scripts/development/setup.sh
```

## 2. Branching

Create a focused branch:

```bash
git checkout -b feature/short-description
```

## 3. Development Workflow

1. Make scoped changes.
2. Update tests and docs with your changes.
3. Run local checks:

```bash
scripts/testing/run-tests.sh
scripts/testing/security-scan.sh
```

## 4. Pull Request Checklist

- Code builds and tests pass.
- Docs updated where behavior changed.
- No secrets committed.
- PR description includes what changed and how it was validated.

## 5. Review Expectations

- Keep PRs reasonably small.
- Address review feedback with follow-up commits.
- Avoid force-push churn once review starts unless necessary.

## 6. Security Reporting

Do not open a public issue for vulnerabilities.

Use the channel in `SECURITY.md` for coordinated disclosure.
