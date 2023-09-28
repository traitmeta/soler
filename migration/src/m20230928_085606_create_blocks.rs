use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Blocks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Blocks::Hash)
                            .binary()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Blocks::Consensus).boolean().not_null())
                    .col(ColumnDef::new(Blocks::Difficulty).decimal_len(50, 0))
                    .col(
                        ColumnDef::new(Blocks::GasLimit)
                            .decimal_len(100, 0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Blocks::GasUsed)
                            .decimal_len(100, 0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Blocks::MinerHash).binary().not_null())
                    .col(ColumnDef::new(Blocks::Nonce).binary().not_null())
                    .col(ColumnDef::new(Blocks::Number).big_integer().not_null())
                    .col(ColumnDef::new(Blocks::ParentHash).binary().not_null())
                    .col(ColumnDef::new(Blocks::Size).integer().not_null())
                    .col(
                        ColumnDef::new(Blocks::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Blocks::TotalDifficulty).integer().not_null())
                    .col(ColumnDef::new(Blocks::BaseFeePerGas).decimal_len(100, 0))
                    // .foreign_key(
                    //     ForeignKey::create()
                    //         .name("block_miner_hash_fkey")
                    //         .from(Blocks::Table, Blocks::MinerHash)
                    //         .to(Addresses::Table, Addresses::Hash),
                    // )
                    .to_owned(),
            )
            .await?;

        // manager
        //     .create_index(
        //         Index::create()
        //             .if_not_exists()
        //             .name("blocks_timestamp_index")
        //             .table(Blocks::Table)
        //             .col(Blocks::Timestamp)
        //             .to_owned(),
        //     )
        //     .await?;

        // manager
        //     .create_index(
        //         Index::create()
        //             .if_not_exists()
        //             .name("one_consensus_block_at_height")
        //             .table(Blocks::Table)
        //             .col(Blocks::Number)
        //             .to_owned(),
        //     )
        //     .await?;

        // manager
        //     .create_index(
        //         Index::create()
        //             .if_not_exists()
        //             .name("one_consensus_child_per_parent")
        //             .table(Blocks::Table)
        //             .col(Blocks::ParentHash)
        //             .to_owned(),
        //     )
        //     .await?;

        Ok(())
    }

    /* TODO not where with manager

    CREATE UNIQUE INDEX "one_consensus_block_at_height" ON "public"."blocks" USING btree (
      "number" "pg_catalog"."int8_ops" ASC NULLS LAST
    ) WHERE consensus;

    CREATE UNIQUE INDEX "one_consensus_child_per_parent" ON "public"."blocks" USING btree (
      "parent_hash" "pg_catalog"."bytea_ops" ASC NULLS LAST
    ) WHERE consensus;
    */
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Blocks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Blocks {
    Table,
    Consensus,
    Difficulty,
    GasLimit,
    GasUsed,
    Hash,
    MinerHash,
    Nonce,
    Number,
    ParentHash,
    Size,
    Timestamp,
    TotalDifficulty,
    BaseFeePerGas,
}
