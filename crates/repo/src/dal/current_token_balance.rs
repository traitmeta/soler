use entities::address_current_token_balances::{ActiveModel, Entity, Model};
use sea_orm::*;

pub struct Query;

impl Query {}

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

    pub async fn save<C>(db: &C, form_datas: &[Model]) -> Result<ExecResult, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut datas = vec![];
        for form_data in form_datas.iter() {
            let mut model = form_data.clone().into_active_model();
            model.id = ActiveValue::NotSet;
            datas.push(model);
        }

        if datas.is_empty() {
            return Err(DbErr::RecordNotInserted);
        }

        let mut stmt = Entity::insert_many(datas)
            .build(DatabaseBackend::Postgres)
            .to_string();
        stmt = format!("{} ON CONFLICT (\"address_hash\", \"token_contract_address_hash\", COALESCE(\"token_id\", -1)) DO NOTHING",  stmt);
        db.execute(Statement::from_string(DatabaseBackend::Postgres, stmt))
            .await
    }
}
