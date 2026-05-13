use std::sync::Arc;

use sqlx::PgPool;
use thiserror::Error;

use crate::config::AppConfig;
use crate::{ApiError, db};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Option<PgPool>,
}

impl AppState {
    pub async fn from_config(config: AppConfig) -> Result<Self, StateInitError> {
        let db = db::connect_and_migrate(&config).await?;

        Ok(Self {
            config: Arc::new(config),
            db,
        })
    }

    pub fn new_without_db(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
            db: None,
        }
    }

    pub fn database(&self) -> Result<&PgPool, ApiError> {
        self.db
            .as_ref()
            .ok_or_else(|| ApiError::database("Database is not configured"))
    }
}

#[derive(Debug, Error)]
pub enum StateInitError {
    #[error("database connection failed")]
    Connect(#[source] sqlx::Error),

    #[error("database migration failed")]
    Migrate(#[source] sqlx::migrate::MigrateError),
}
