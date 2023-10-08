//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "address_to_tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub address_hash: Vec<u8>,
    pub tag_id: i32,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::address_tags::Entity",
        from = "Column::TagId",
        to = "super::address_tags::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    AddressTags,
}

impl Related<super::address_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AddressTags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}