use ::entities::addresses::{ActiveModel, Column, Entity, Model};
use migration::OnConflict;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_by_hash(db: &DbConn, hash: Vec<u8>) -> Result<Option<Model>, DbErr> {
        Entity::find().filter(Column::Hash.eq(hash)).one(db).await
    }

    pub async fn filter_no_featched(db: &DbConn, block_height: i64) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(
                Condition::any()
                    .add(Column::FetchedCoinBalanceBlockNumber.lte(Some(block_height)))
                    .add(Column::FetchedCoinBalanceBlockNumber.is_null()),
            )
            .limit(50)
            .all(db)
            .await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn save<C>(db: &C, form_datas: &[Model]) -> Result<InsertResult<ActiveModel>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut datas = vec![];
        for form_data in form_datas.iter() {
            let model = ActiveModel {
                fetched_coin_balance: Unchanged(form_data.fetched_coin_balance),
                fetched_coin_balance_block_number: Unchanged(
                    form_data.fetched_coin_balance_block_number,
                ),
                hash: Set(form_data.hash.clone()),
                contract_code: match &form_data.contract_code {
                    Some(code) => Set(Some(code.clone())),
                    None => Unchanged(None),
                },
                inserted_at: Unchanged(form_data.inserted_at),
                updated_at: Set(form_data.updated_at),
                nonce: match form_data.nonce {
                    Some(nonce) => Set(Some(nonce)),
                    None => Unchanged(None),
                },
                decompiled: Unchanged(form_data.decompiled),
                verified: Unchanged(form_data.verified),
                gas_used: Unchanged(form_data.gas_used),
                transactions_count: Unchanged(form_data.transactions_count),
                token_transfers_count: Unchanged(form_data.token_transfers_count),
            };
            datas.push(model);
        }

        if datas.is_empty() {
            return Err(DbErr::RecordNotInserted);
        }

        Entity::insert_many(datas)
            .on_conflict(
                OnConflict::column(Column::Hash)
                    .update_column(Column::Nonce)
                    .to_owned(),
            )
            .exec(db)
            .await
    }
}
