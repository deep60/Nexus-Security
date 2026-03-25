# Configuration Layout

This folder now has two levels of config:

1. Root environment summaries:
- `development.yaml`
- `test.yaml`
- `staging.yaml`
- `production.yaml`

2. Service-specific templates per environment:
- `config/development/`
- `config/staging/`
- `config/production/`

Service files currently included:
- `api-gateway.toml`
- `bounty-manager.toml`
- `user-service.env`
- `analysis-engine.env`

## Notes

- `api-gateway.toml` and `bounty-manager.toml` are TOML because those services support typed config structures.
- `user-service` and `analysis-engine` currently read environment variables, so `.env` templates are provided.
- Replace placeholder values (for example `${JWT_SECRET}`) with secret-manager values in non-dev environments.
