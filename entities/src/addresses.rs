//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "addresses")]
pub struct Model {
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub fetched_coin_balance: Option<Decimal>,
    pub fetched_coin_balance_block_number: Option<i64>,
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Binary(BlobSize::Blob(None))"
    )]
    pub hash: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub contract_code: Option<Vec<u8>>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    pub nonce: Option<i32>,
    pub decompiled: Option<bool>,
    pub verified: Option<bool>,
    pub gas_used: Option<i64>,
    pub transactions_count: Option<i32>,
    pub token_transfers_count: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}