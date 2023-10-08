//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "last_fetched_counters")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub counter_type: String,
    #[sea_orm(column_type = "Decimal(Some((100, 0)))", nullable)]
    pub value: Option<Decimal>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}