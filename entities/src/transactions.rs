//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub cumulative_gas_used: Option<Decimal>,
    pub error: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))")]
    pub gas: Decimal,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub gas_price: Option<Decimal>,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub gas_used: Option<Decimal>,
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Binary(BlobSize::Blob(None))"
    )]
    pub hash: Vec<u8>,
    pub index: Option<i32>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub input: Vec<u8>,
    pub nonce: i32,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))")]
    pub r: Decimal,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))")]
    pub s: Decimal,
    pub status: Option<i32>,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))")]
    pub v: Decimal,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))")]
    pub value: Decimal,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub block_hash: Option<Vec<u8>>,
    pub block_number: Option<i32>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub from_address_hash: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub to_address_hash: Option<Vec<u8>>,
    // Denormalized `internal_transaction` `created_contract_address_hash` populated only when `to_address_hash` is nil.
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub created_contract_address_hash: Option<Vec<u8>>,
    //  when created `address` code was fetched by `Indexer`
    pub created_contract_code_indexed_at: Option<DateTime>,
    // If the pending transaction fetcher was alive and received this transaction, we can
    //  be sure that this transaction did not start processing until after the last time we fetched pending transactions,
    //  so we annotate that with this field. If it is `nil`, that means we don't have a lower bound for when it started
    //  processing.
    pub earliest_processing_start: Option<DateTime>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub old_block_hash: Option<Vec<u8>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub revert_reason: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub max_priority_fee_per_gas: Option<Decimal>,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub max_fee_per_gas: Option<Decimal>,
    pub r#type: Option<i32>,
    pub has_error_in_internal_txs: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::blocks::Entity",
        from = "Column::BlockHash",
        to = "super::blocks::Column::Hash",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Blocks,
    #[sea_orm(has_many = "super::internal_transactions::Entity")]
    InternalTransactions,
    #[sea_orm(has_many = "super::logs::Entity")]
    Logs,
    #[sea_orm(has_many = "super::token_transfers::Entity")]
    TokenTransfers,
    #[sea_orm(has_many = "super::transaction_actions::Entity")]
    TransactionActions,
    #[sea_orm(has_many = "super::transaction_forks::Entity")]
    TransactionForks,
}

impl Related<super::blocks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Blocks.def()
    }
}

impl Related<super::internal_transactions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InternalTransactions.def()
    }
}

impl Related<super::logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Logs.def()
    }
}

impl Related<super::token_transfers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TokenTransfers.def()
    }
}

impl Related<super::transaction_actions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransactionActions.def()
    }
}

impl Related<super::transaction_forks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransactionForks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
