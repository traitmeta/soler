//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "address_token_balances")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub address_hash: Vec<u8>,
    pub block_number: i64,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub token_contract_address_hash: Vec<u8>,
    pub value: Option<BigDecimal>,
    pub value_fetched_at: Option<DateTime>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(column_type = "Decimal(Some((78, 0)))", nullable)]
    pub token_id: Option<BigDecimal>,
    pub token_type: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tokens::Entity",
        from = "Column::TokenContractAddressHash",
        to = "super::tokens::Column::ContractAddressHash",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Tokens,
}

impl Related<super::tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
