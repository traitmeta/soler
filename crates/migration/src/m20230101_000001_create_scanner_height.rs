use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ScannerHeight::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScannerHeight::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ScannerHeight::TaskName).string().not_null())
                    .col(ColumnDef::new(ScannerHeight::ChainName).string().not_null())
                    .col(ColumnDef::new(ScannerHeight::Height).integer().not_null())
                    .col(ColumnDef::new(ScannerHeight::CreatedAt).date().not_null())
                    .col(ColumnDef::new(ScannerHeight::UpdatedAt).date().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ScannerHeight::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ScannerHeight {
    Table,
    Id,
    TaskName,
    ChainName,
    Height,
    CreatedAt,
    UpdatedAt,
}
