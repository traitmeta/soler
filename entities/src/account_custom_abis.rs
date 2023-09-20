//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "account_custom_abis")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub identity_id: i64,
    #[sea_orm(column_type = "JsonBinary")]
    pub abi: Json,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub address_hash_hash: Option<Vec<u8>>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub address_hash: Option<Vec<u8>>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub name: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account_identities::Entity",
        from = "Column::IdentityId",
        to = "super::account_identities::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AccountIdentities,
}

impl Related<super::account_identities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountIdentities.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
