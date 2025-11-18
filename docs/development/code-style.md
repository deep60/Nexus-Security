# Code Style Guide

This document outlines the coding standards and conventions for the Nexus Security project.

## Rust Code Style

### Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `BountyManager` |
| Functions | snake_case | `get_analysis_result` |
| Variables | snake_case | `file_hash` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_FILE_SIZE` |
| Modules | snake_case | `bounty_manager` |

### Error Handling

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Analysis timeout after {0} seconds")]
    Timeout(u64),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Async Code

- Use `tokio` for async runtime
- Prefer `async/await` over manual futures
- Use `?` operator for error propagation

```rust
pub async fn analyze_file(file_id: Uuid) -> Result<AnalysisResult, AnalysisError> {
    let file = get_file(file_id).await?;
    let result = perform_analysis(&file).await?;
    store_result(&result).await?;
    Ok(result)
}
```

### Documentation

Document all public items:

```rust
/// Analyzes a file for potential threats.
///
/// # Arguments
///
/// * `file_path` - Path to the file to analyze
/// * `options` - Analysis configuration options
///
/// # Returns
///
/// Returns `AnalysisResult` on success, or `AnalysisError` on failure.
///
/// # Examples
///
/// ```
/// let result = analyze_file("/tmp/sample.exe", AnalysisOptions::default()).await?;
/// println!("Verdict: {:?}", result.verdict);
/// ```
pub async fn analyze_file(
    file_path: &str,
    options: AnalysisOptions,
) -> Result<AnalysisResult, AnalysisError> {
    // Implementation
}
```

### Linting

Run clippy and fix all warnings:

```bash
cargo clippy -- -D warnings
```

## TypeScript/React Code Style

### Formatting

Use Prettier with project config:

```bash
npm run format
```

### ESLint

```bash
npm run lint
```

### Component Structure

```tsx
// components/BountyCard.tsx
import React from 'react';
import { Bounty } from '@/types';

interface BountyCardProps {
  bounty: Bounty;
  onClaim?: (id: string) => void;
}

export const BountyCard: React.FC<BountyCardProps> = ({ bounty, onClaim }) => {
  return (
    <div className="rounded-lg border p-4">
      <h3 className="text-lg font-semibold">{bounty.title}</h3>
      <p className="text-gray-600">{bounty.description}</p>
      {onClaim && (
        <button onClick={() => onClaim(bounty.id)}>
          Claim Bounty
        </button>
      )}
    </div>
  );
};
```

### Hooks

Custom hooks should start with `use`:

```tsx
export function useBounties(filters: BountyFilters) {
  const [bounties, setBounties] = useState<Bounty[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchBounties(filters).then(setBounties).finally(() => setLoading(false));
  }, [filters]);

  return { bounties, loading };
}
```

## SQL Style

### Naming

- Tables: plural, snake_case (`bounties`, `analysis_results`)
- Columns: snake_case (`created_at`, `file_hash`)
- Indexes: `idx_table_column`
- Foreign keys: `fk_table_reference`

### Formatting

```sql
SELECT
    b.id,
    b.title,
    b.reward_amount,
    u.username AS creator_name
FROM bounties b
INNER JOIN users u ON b.creator_id = u.id
WHERE b.status = 'open'
    AND b.expires_at > NOW()
ORDER BY b.created_at DESC
LIMIT 20;
```

## Git Conventions

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `refactor/description` - Code refactoring
- `docs/description` - Documentation updates

### Commit Messages

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

```
feat(analysis): add YARA rule hot-reloading

fix(bounty): correct reward calculation for multiple analysts

docs(api): update authentication endpoint examples
```

## Code Review Checklist

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] New code has tests
- [ ] Documentation updated
- [ ] No security vulnerabilities
- [ ] No hardcoded secrets
- [ ] Error handling is appropriate
- [ ] Logging is sufficient
