use std::time::Duration;

use migration::DbErr;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log;

pub async fn migration(database_url: String) {
    use migration::{Migrator, MigratorTrait};

    let conn = Database::connect(&database_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();
}

pub async fn connect_db(url: String) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("public".into()); // Setting default PostgreSQL schema

    Database::connect(opt).await
}

#[cfg(test)]
pub mod test {
    use tokio::runtime::Runtime;

    use super::connect_db;

    #[test]
    #[ignore]
    fn test_connect_db() {
        let url = "mysql://root:meta@localhost/rust_test".to_string();
        let runtime = Runtime::new().unwrap();
        runtime.block_on(connect_db(url)).unwrap();
    }
}
