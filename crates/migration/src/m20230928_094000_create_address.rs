use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Addresses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Addresses::Hash)
                            .binary()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Addresses::FetchedCoinBalance).decimal_len(100, 0))
                    .col(ColumnDef::new(Addresses::FetchedCoinBalanceBlockNumber).big_integer())
                    .col(ColumnDef::new(Addresses::ContractCode).binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Addresses::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Addresses {
    Table,
    Hash,
    FetchedCoinBalance,
    FetchedCoinBalanceBlockNumber,
    ContractCode,
}
