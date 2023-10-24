use config::db::DB;
use sea_orm::DbErr;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

// pub async fn migration(database_url: String) {
//     use migration::{Migrator, MigratorTrait};

//     let conn = Database::connect(&database_url).await.unwrap();
//     Migrator::up(&conn, None).await.unwrap();
// }

pub async fn connect_db(cfg: DB) -> Result<DatabaseConnection, DbErr> {
    let db_url = cfg.url();
    let mut opt = ConnectOptions::new(db_url);
    let log_level = cfg.from_usize();
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log_level)
        .set_schema_search_path("public"); // Setting default PostgreSQL schema

    Database::connect(opt).await
}

#[cfg(test)]
pub mod test {
    use config::db::DB;
    use tokio::runtime::Runtime;

    use super::connect_db;

    #[test]
    #[ignore]
    fn test_connect_db() {
        let db_cfg = DB {
            url: "localhost:3306".to_string(),
            schema: "mysql".to_string(),
            username: "root".to_string(),
            password: "meta".to_string(),
            database: "rust_test".to_string(),
            log_level: 3,
        };

        let runtime = Runtime::new().unwrap();
        runtime.block_on(connect_db(db_cfg)).unwrap();
    }
}
