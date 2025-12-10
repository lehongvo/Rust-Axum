use std::env;

use crate::constants::{
    DEFAULT_ADMIN_PASS, DEFAULT_ADMIN_USER, DEFAULT_APP_PORT, DEFAULT_DATABASE_URL,
    DEFAULT_RATE_LIMIT_PER_MIN,
};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub app_port: u16,
    pub api_key: String,
    pub jwt_secret: String,
    pub admin_user: String,
    pub admin_pass: String,
    pub rate_limit_per_minute: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        // If DATABASE_URL is absent, default to docker-compose service
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.into());
        let app_port = env::var("APP_PORT")
            .unwrap_or_else(|_| DEFAULT_APP_PORT.to_string())
            .parse()
            .unwrap_or(DEFAULT_APP_PORT);
        let api_key = env::var("API_KEY").map_err(|_| "API_KEY must be set")?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set")?;
        let admin_user = env::var("ADMIN_USER").unwrap_or_else(|_| DEFAULT_ADMIN_USER.to_string());
        let admin_pass = env::var("ADMIN_PASS").unwrap_or_else(|_| DEFAULT_ADMIN_PASS.to_string());
        let rate_limit_per_minute = env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| DEFAULT_RATE_LIMIT_PER_MIN.to_string())
            .parse()
            .unwrap_or(DEFAULT_RATE_LIMIT_PER_MIN);

        Ok(Self {
            database_url,
            app_port,
            api_key,
            jwt_secret,
            admin_user,
            admin_pass,
            rate_limit_per_minute,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::env;
    use std::sync::Mutex as StdMutex;

    static ENV_LOCK: Lazy<StdMutex<()>> = Lazy::new(|| StdMutex::new(()));

    fn set_env(vars: &[(&str, &str)]) {
        for (k, v) in vars {
            unsafe {
                env::set_var(k, v);
            }
        }
    }

    #[test]
    fn loads_required_env() {
        let _g = ENV_LOCK.lock().unwrap();
        set_env(&[
            ("DATABASE_URL", "postgres://app:app@localhost:5432/app"),
            ("API_KEY", "k"),
            ("JWT_SECRET", "secret"),
        ]);
        // optional
        unsafe {
            env::remove_var("APP_PORT");
            env::remove_var("ADMIN_USER");
            env::remove_var("ADMIN_PASS");
            env::remove_var("RATE_LIMIT_PER_MINUTE");
        }

        let cfg = AppConfig::from_env().unwrap();
        assert_eq!(cfg.database_url, "postgres://app:app@localhost:5432/app");
        assert_eq!(cfg.app_port, DEFAULT_APP_PORT);
        assert_eq!(cfg.api_key, "k");
        assert_eq!(cfg.jwt_secret, "secret");
        assert_eq!(cfg.admin_user, "admin");
        assert_eq!(cfg.admin_pass, "password");
        assert_eq!(cfg.rate_limit_per_minute, 60);
    }

    #[test]
    fn missing_required_env_errors() {
        let _g = ENV_LOCK.lock().unwrap();
        for key in ["DATABASE_URL", "API_KEY", "JWT_SECRET"] {
            unsafe {
                env::remove_var(key);
            }
        }
        let err = AppConfig::from_env().unwrap_err();
        assert!(
            err.contains("must be set"),
            "expected missing var error, got {err}"
        );
    }
}
