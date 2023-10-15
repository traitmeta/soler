use ::entities::internal_transactions::{ActiveModel, Column, Entity, Model};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_by_hash(db: &DbConn, hash: Vec<u8>) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::TransactionHash.eq(hash))
            .all(db)
            .await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create<C>(db: &C, form_datas: &[Model]) -> Result<InsertResult<ActiveModel>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut batch = vec![];
        for form_data in form_datas.iter() {
            let data = form_data.clone().into_active_model();
            batch.push(data);
        }

        Entity::insert_many(batch).exec(db).await
    }
}
