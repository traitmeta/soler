//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "scanner_contract")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub chain_name: String,
    pub chain_id: u32,
    pub address: String,
    pub event_sign: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
