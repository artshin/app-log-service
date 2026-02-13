//! Configuration management for the log server.
//!
//! Loads configuration from environment variables with sensible defaults.

use std::path::PathBuf;

/// Default port for the log server
const DEFAULT_PORT: u16 = 9006;

/// Default buffer capacity (number of log entries) - increased for hybrid logging
const DEFAULT_CAPACITY: usize = 10_000;

/// Default upload directory for client log uploads
const DEFAULT_UPLOAD_DIR: &str = "./uploads";

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Port to listen on
    pub port: u16,

    /// Buffer capacity (number of entries)
    pub capacity: usize,

    /// Verbose mode (show file/line metadata)
    pub verbose: bool,

    /// Directory for storing uploaded client logs
    pub upload_dir: PathBuf,

    /// Path to JWT public key for authentication
    pub jwt_public_key_path: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_PORT);

        let capacity = std::env::var("CAPACITY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_CAPACITY);

        let verbose = std::env::var("VERBOSE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let upload_dir = std::env::var("UPLOAD_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_UPLOAD_DIR));

        let jwt_public_key_path = std::env::var("JWT_PUBLIC_KEY_PATH").ok();

        Self {
            port,
            capacity,
            verbose,
            upload_dir,
            jwt_public_key_path,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            capacity: DEFAULT_CAPACITY,
            verbose: false,
            upload_dir: PathBuf::from(DEFAULT_UPLOAD_DIR),
            jwt_public_key_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 9006);
        assert_eq!(config.capacity, 10_000);
        assert!(!config.verbose);
        assert_eq!(config.upload_dir, PathBuf::from("./uploads"));
        assert!(config.jwt_public_key_path.is_none());
    }
}
