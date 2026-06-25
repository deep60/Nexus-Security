# Verdyx API documentation

The hand-maintained OpenAPI 3 spec lives at [`openapi.yaml`](./openapi.yaml).

## View the docs locally

```bash
# With redocly (one-shot)
npx @redocly/cli@latest preview-docs docs/api/openapi.yaml

# Or with swagger-ui in Docker
docker run -p 8000:8080 -e SWAGGER_JSON=/api/openapi.yaml \
  -v "$PWD/docs/api:/api" swaggerapi/swagger-ui
# then open http://localhost:8000
```

## Validate the spec in CI

```bash
npx @redocly/cli@latest lint docs/api/openapi.yaml
```

## Roadmap: generate from code

Add [`utoipa`](https://crates.io/crates/utoipa) annotations to the
api-gateway handlers and serve `/openapi.json` directly. Once that's in
place, this file becomes the generated artifact instead of a manual one.

Example annotation pattern:

```rust
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "OK",  body = AuthResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(/* ... */) -> Result<Json<AuthResponse>, ApiError> { /* ... */ }
```
