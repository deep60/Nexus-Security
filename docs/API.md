# Nexus Security API Documentation

Base URL: `https://api.nexus-security.com/v1`

## Authentication

All authenticated endpoints require a JWT token in the Authorization header:

```http
Authorization: Bearer <token>
```

### Obtain Token

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your_password"
}
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

## Endpoints

### Authentication

#### Register User

```http
POST /auth/register
```

**Request Body:**

```json
{
  "email": "user@example.com",
  "username": "analyst123",
  "password": "SecurePass123!",
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f..."
}
```

**Response:** `201 Created`

```json
{
  "id": "uuid",
  "email": "user@example.com",
  "username": "analyst123",
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f...",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### Refresh Token

```http
POST /auth/refresh
```

**Request Body:**

```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### File Analysis

#### Submit File for Analysis

```http
POST /analyze
Content-Type: multipart/form-data
Authorization: Bearer <token>
```

**Form Fields:**

- `file` (required): Binary file data (max 100MB)
- `bounty_id` (optional): UUID of associated bounty
- `priority` (optional): `normal` | `high`

**Response:** `202 Accepted`

```json
{
  "file_id": "uuid",
  "status": "queued",
  "estimated_time": 120,
  "position_in_queue": 5
}
```

#### Get Analysis Results

```http
GET /analyze/{file_id}
Authorization: Bearer <token>
```

**Response:**

```json
{
  "file_id": "uuid",
  "filename": "suspicious.exe",
  "file_size": 1048576,
  "verdict": "malicious",
  "confidence": 0.95,
  "threat_level": "high",
  "analysis_data": {
    "hashes": {
      "md5": "d41d8cd98f00b204e9800998ecf8427e",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb924..."
    },
    "yara_matches": [
      {
        "rule": "SuspiciousImports",
        "severity": "high",
        "description": "Detects suspicious API imports"
      }
    ],
    "static_analysis": {
      "file_type": "PE32 executable",
      "entropy": 7.2,
      "imports_count": 45
    }
  },
  "completed_at": "2024-01-15T10:32:00Z"
}
```

### Bounties

#### List Bounties

```http
GET /bounties
Authorization: Bearer <token>
```

**Query Parameters:**

- `status`: `open` | `claimed` | `resolved` | `expired`
- `min_reward`: Minimum reward amount
- `sort`: `created_at` | `reward_amount` | `expires_at`
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 20)

#### Create Bounty

```http
POST /bounties
Authorization: Bearer <token>
```

**Request Body:**

```json
{
  "file_id": "uuid",
  "title": "Analysis of suspicious DLL",
  "description": "Found in phishing email",
  "reward_amount": 100,
  "token_address": "0x...",
  "expires_at": "2024-01-20T23:59:59Z"
}
```

#### Submit Analysis to Bounty

```http
POST /bounties/{bounty_id}/submit
Authorization: Bearer <token>
```

**Request Body:**

```json
{
  "verdict": "malicious",
  "confidence": 0.85,
  "analysis_report": "Detailed findings..."
}
```

### Users

#### Get Current User

```http
GET /users/me
Authorization: Bearer <token>
```

### Reputation

#### Get Leaderboard

```http
GET /reputation/leaderboard
```

## WebSocket API

Connect to: `wss://api.nexus-security.com/ws`

### Event Types

- `analysis:progress` - Analysis progress updates
- `bounty:update` - Bounty status changes
- `reward:distributed` - Reward distribution events

## Error Responses

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message"
  },
  "request_id": "req_abc123"
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or expired token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `RATE_LIMITED` | 429 | Too many requests |

## Rate Limits

- **Authenticated**: 100 requests/minute
- **File Upload**: 10 uploads/hour

## SDKs & Tools

- [OpenAPI Specification](api/openapi.yaml)
- [Postman Collection](api/postman/)
