//! Database operations

mod entries;

use std::str::FromStr;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

/// Set up connection
pub async fn init(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{path}"))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}