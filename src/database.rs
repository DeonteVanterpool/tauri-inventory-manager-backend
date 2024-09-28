use crate::diesel::Connection;
use core::fmt;
use diesel::PgConnection;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use dotenvy::dotenv;
use std::env;
use std::error::Error;
use std::fmt::Display;

pub async fn establish_async_connection() -> AsyncPgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    AsyncPgConnection::establish(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub async fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

use deadpool::managed;

#[derive(Debug)]
pub enum PoolError {
    Fail,
}

impl Display for PoolError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

pub struct Manager {}

#[rocket::async_trait]
impl managed::Manager for Manager {
    type Type = AsyncPgConnection;
    type Error = PoolError;

    async fn create(&self) -> Result<AsyncPgConnection, PoolError> {
        Ok(establish_async_connection().await)
    }

    async fn recycle(&self, _: &mut AsyncPgConnection) -> managed::RecycleResult<PoolError> {
        Ok(())
    }
}
pub type Pool = managed::Pool<Manager>;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(
    connection: &mut PgConnection,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}
