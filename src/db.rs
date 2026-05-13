use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{AppConfig, StateInitError};

pub async fn connect_and_migrate(config: &AppConfig) -> Result<Option<PgPool>, StateInitError> {
    let Some(database_url) = config.database_url.as_deref() else {
        return Ok(None);
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(StateInitError::Connect)?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(StateInitError::Migrate)?;

    Ok(Some(pool))
}
