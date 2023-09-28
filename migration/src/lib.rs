pub use sea_orm_migration::prelude::*;

mod m20230101_000001_create_scanner_height;
mod m20230928_085606_create_blocks;
mod m20230928_094000_create_address;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230101_000001_create_scanner_height::Migration),
            Box::new(m20230928_085606_create_blocks::Migration),
            Box::new(m20230928_094000_create_address::Migration),
        ]
    }
}
