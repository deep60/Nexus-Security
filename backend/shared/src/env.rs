//! Environment helpers shared across services.

use std::env;

/// True when the service is running with `ENVIRONMENT=production`.
///
/// Matches the convention used by `api-gateway`'s config loader and CORS
/// middleware: the variable is read directly, and anything other than
/// `production` (including an unset variable) is treated as non-production.
pub fn is_production() -> bool {
    env::var("ENVIRONMENT")
        .map(|e| e.trim().eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

/// Read a credential from the environment, falling back to a development
/// default only outside production.
///
/// The MinIO defaults baked into the compose stack are a convenience for local
/// work. In production they are actively dangerous: a service that silently
/// falls back would either point at the wrong object store or authenticate with
/// well-known credentials, and would do so while looking perfectly healthy. So
/// a missing or blank value is fatal there instead.
pub fn credential_or_dev_default(var: &str, dev_default: &str) -> anyhow::Result<String> {
    match env::var(var) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ if is_production() => anyhow::bail!(
            "{var} must be set when ENVIRONMENT=production; \
             refusing to fall back to the development default"
        ),
        _ => Ok(dev_default.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ENVIRONMENT` is process-global, so these cases share one test to keep
    /// them from racing each other under the default parallel test runner.
    #[test]
    fn credential_resolution_depends_on_environment() {
        let restore = env::var("ENVIRONMENT").ok();
        let set_environment = |v: Option<&str>| match v {
            Some(v) => env::set_var("ENVIRONMENT", v),
            None => env::remove_var("ENVIRONMENT"),
        };

        // Development: a missing variable falls back.
        set_environment(Some("development"));
        env::remove_var("VERDYX_TEST_CRED");
        assert_eq!(
            credential_or_dev_default("VERDYX_TEST_CRED", "dev-default").unwrap(),
            "dev-default"
        );

        // Production: a missing variable is fatal.
        set_environment(Some("production"));
        assert!(credential_or_dev_default("VERDYX_TEST_CRED", "dev-default").is_err());

        // Production: a blank value is treated as missing.
        env::set_var("VERDYX_TEST_CRED", "   ");
        assert!(credential_or_dev_default("VERDYX_TEST_CRED", "dev-default").is_err());

        // Production: a real value is used.
        env::set_var("VERDYX_TEST_CRED", "real-secret");
        assert_eq!(
            credential_or_dev_default("VERDYX_TEST_CRED", "dev-default").unwrap(),
            "real-secret"
        );

        env::remove_var("VERDYX_TEST_CRED");
        set_environment(restore.as_deref());
    }
}
