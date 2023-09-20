//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use super::sea_orm_active_enums::TransactionActionsProtocol;
use super::sea_orm_active_enums::TransactionActionsType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "transaction_actions")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Binary(BlobSize::Blob(None))"
    )]
    pub hash: Vec<u8>,
    pub protocol: TransactionActionsProtocol,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: Json,
    pub r#type: TransactionActionsType,
    #[sea_orm(primary_key, auto_increment = false)]
    pub log_index: i32,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transactions::Entity",
        from = "Column::Hash",
        to = "super::transactions::Column::Hash",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Transactions,
}

impl Related<super::transactions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
