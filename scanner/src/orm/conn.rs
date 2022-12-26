use std::time::Duration;

use migration::DbErr;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log;

pub async fn migration(database_url: String){
    use migration::{Migrator, MigratorTrait};

    let connection = Database::connect(&database_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
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
        .set_schema_search_path("my_schema".into()); // Setting default PostgreSQL schema

    Database::connect(opt).await
}

#[cfg(test)]
pub mod test {
    use super::connect_db;

    #[test]
    fn test_connect_db() {
        let url = "mysql://root:meta@localhost/rust_test".to_string();
        connect_db(url);
    }
}
