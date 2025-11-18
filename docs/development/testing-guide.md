# Testing Guide

This guide covers testing practices for the Nexus Security project.

## Test Types

| Type | Purpose | Location |
|------|---------|----------|
| Unit | Test individual functions | `src/*/tests.rs` |
| Integration | Test service interactions | `tests/` |
| E2E | Test full user flows | `e2e/` |

## Backend Testing (Rust)

### Running Tests

```bash
cd backend

# Run all tests
cargo test

# Run specific test
cargo test test_analyze_file

# Run tests with output
cargo test -- --nocapture

# Run tests for specific crate
cargo test -p analysis-engine
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_entropy() {
        let data = b"AAAAAAAAAA";
        let entropy = calculate_entropy(data);
        assert!(entropy < 1.0, "Low entropy expected for repeated data");
    }

    #[tokio::test]
    async fn test_hash_file() {
        let result = hash_file("/tmp/test.txt").await.unwrap();
        assert_eq!(result.md5.len(), 32);
        assert_eq!(result.sha256.len(), 64);
    }
}
```

### Integration Tests

```rust
// tests/bounty_integration.rs
use nexus_security::test_utils::*;

#[tokio::test]
async fn test_bounty_creation_flow() {
    let app = spawn_test_app().await;

    // Create user
    let user = app.create_test_user().await;

    // Upload file
    let file_id = app.upload_file(&user.token, "test.exe").await;

    // Create bounty
    let bounty = app.create_bounty(&user.token, file_id, 100).await;

    assert_eq!(bounty.status, "open");
    assert_eq!(bounty.reward_amount, 100);
}
```

### Mocking

Use `mockall` for mocking:

```rust
use mockall::automock;

#[automock]
trait DatabaseClient {
    async fn get_bounty(&self, id: Uuid) -> Result<Bounty, Error>;
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockDatabaseClient::new();
    mock.expect_get_bounty()
        .returning(|_| Ok(Bounty::default()));

    let service = BountyService::new(mock);
    let result = service.get_bounty(Uuid::new_v4()).await;
    assert!(result.is_ok());
}
```

### Test Fixtures

```rust
// tests/fixtures/mod.rs
pub fn sample_bounty() -> Bounty {
    Bounty {
        id: Uuid::new_v4(),
        title: "Test Bounty".to_string(),
        reward_amount: 100,
        status: BountyStatus::Open,
        ..Default::default()
    }
}

pub fn sample_analysis_result() -> AnalysisResult {
    AnalysisResult {
        verdict: Verdict::Malicious,
        confidence: 0.95,
        ..Default::default()
    }
}
```

## Frontend Testing (React)

### Running Tests

```bash
cd frontend

# Run all tests
npm test

# Run with coverage
npm test -- --coverage

# Run specific file
npm test -- BountyCard.test.tsx
```

### Component Tests

```tsx
// components/__tests__/BountyCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { BountyCard } from '../BountyCard';

describe('BountyCard', () => {
  const mockBounty = {
    id: '1',
    title: 'Test Bounty',
    description: 'Test description',
    reward_amount: 100,
  };

  it('renders bounty information', () => {
    render(<BountyCard bounty={mockBounty} />);

    expect(screen.getByText('Test Bounty')).toBeInTheDocument();
    expect(screen.getByText('100')).toBeInTheDocument();
  });

  it('calls onClaim when button clicked', () => {
    const handleClaim = jest.fn();
    render(<BountyCard bounty={mockBounty} onClaim={handleClaim} />);

    fireEvent.click(screen.getByText('Claim'));
    expect(handleClaim).toHaveBeenCalledWith('1');
  });
});
```

### Hook Tests

```tsx
// hooks/__tests__/useBounties.test.tsx
import { renderHook, waitFor } from '@testing-library/react';
import { useBounties } from '../useBounties';

jest.mock('@/api/bounties', () => ({
  fetchBounties: jest.fn().mockResolvedValue([
    { id: '1', title: 'Bounty 1' },
  ]),
}));

describe('useBounties', () => {
  it('fetches bounties on mount', async () => {
    const { result } = renderHook(() => useBounties({}));

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.bounties).toHaveLength(1);
  });
});
```

## Database Testing

### Test Database Setup

```bash
# Create test database
createdb nexus_security_test

# Run migrations
DATABASE_URL=postgresql://localhost/nexus_security_test sqlx migrate run
```

### Transaction Rollback

```rust
#[sqlx::test]
async fn test_create_user(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "test@example.com").await?;
    assert!(user.id.is_some());
    Ok(())
    // Transaction automatically rolled back
}
```

## API Testing

### Using HTTPie

```bash
# Login
http POST localhost:8080/api/v1/auth/login \
  email=test@example.com password=password

# Create bounty
http POST localhost:8080/api/v1/bounties \
  Authorization:"Bearer $TOKEN" \
  file_id=uuid title="Test" reward_amount:=100
```

### Load Testing

```bash
# Install k6
brew install k6

# Run load test
k6 run tests/load/bounties.js
```

```javascript
// tests/load/bounties.js
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 100,
  duration: '30s',
};

export default function () {
  const res = http.get('http://localhost:8080/api/v1/bounties');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 200ms': (r) => r.timings.duration < 200,
  });
}
```

## Coverage

### Backend Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Frontend Coverage

```bash
npm test -- --coverage --watchAll=false
```

### Coverage Targets

| Type | Target |
|------|--------|
| Unit tests | 80% |
| Integration tests | 60% |
| Overall | 70% |

## CI/CD Testing

Tests run automatically on PR:

1. Lint checks
2. Unit tests
3. Integration tests
4. Security scan
5. Coverage report

## Best Practices

1. **Test behavior, not implementation**
2. **Use descriptive test names**
3. **One assertion per test (when possible)**
4. **Keep tests independent**
5. **Clean up test data**
6. **Mock external services**
7. **Test edge cases**
8. **Test error conditions**
