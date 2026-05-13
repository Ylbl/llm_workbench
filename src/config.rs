use std::{
    env,
    net::{IpAddr, SocketAddr},
    num::ParseIntError,
    path::PathBuf,
};

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: Option<String>,
    pub app_data_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("APP_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse()
            .map_err(ConfigError::InvalidHost)?;

        let port_value = env::var("APP_PORT").unwrap_or_else(|_| "3000".to_string());
        let port = port_value
            .parse()
            .map_err(|source| ConfigError::InvalidPort {
                value: port_value,
                source,
            })?;

        let database_url = env::var("DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());

        let app_data_dir = env::var("APP_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data"));

        Ok(Self {
            host,
            port,
            database_url,
            app_data_dir,
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }

    #[cfg(test)]
    pub fn for_test() -> Self {
        Self {
            host: "127.0.0.1".parse().unwrap(),
            port: 3000,
            database_url: None,
            app_data_dir: PathBuf::from("test-data"),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("APP_HOST must be a valid IP address")]
    InvalidHost(#[source] std::net::AddrParseError),

    #[error("APP_PORT must be a valid TCP port, got `{value}`")]
    InvalidPort {
        value: String,
        #[source]
        source: ParseIntError,
    },
}
