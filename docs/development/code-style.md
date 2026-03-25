# Code Style Guide

## General Principles

- Keep functions small and explicit.
- Prefer clear naming over abbreviations.
- Add tests for behavior changes.
- Keep security-sensitive logic easy to audit.

## Rust (Backend)

### Formatting and Linting

```bash
cd backend
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Naming

- Types: `PascalCase`
- Functions/modules/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

### Error Handling

- Prefer typed errors (`thiserror`) and context (`anyhow::Context`).
- Return early for invalid input.
- Avoid `unwrap()` in production paths.

## TypeScript/React (Frontend)

### Formatting and Linting

```bash
cd frontend
npm run lint
npm run test:run
```

### Conventions

- Components in `PascalCase`.
- Hooks prefixed with `use`.
- Keep API calls isolated from UI components when possible.

## Solidity/Hardhat (Blockchain)

### Checks

```bash
cd blockchain
npm run lint
npm test
```

### Conventions

- Keep access control explicit.
- Emit events for state transitions.
- Keep modifiers and error messages consistent.

## Commit Style

Use conventional commits:

- `feat(scope): ...`
- `fix(scope): ...`
- `docs(scope): ...`
- `refactor(scope): ...`
- `test(scope): ...`

Example:

```text
feat(api-gateway): add bounty stats endpoint
```
