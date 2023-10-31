use ::entities::address_token_balances::{ActiveModel, Column, Entity, Model};
use entities::address_token_balances::Relation;
use sea_orm::*;

pub struct Query;

impl Query {
    // Builds an `Ecto.Query` to fetch the unfetched token balances.
    // Unfetched token balances are the ones that have the column `value_fetched_at` nil or the value is null. This query also
    // ignores the burn_address for tokens ERC-721 since the most tokens ERC-721 don't allow get the
    // balance for burn_address.
    pub async fn unfetched_token_balances(db: &DbConn) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .join(JoinType::LeftJoin, Relation::Tokens.def())
            .filter(
                // ((tb.address_hash != @burn_address_hash and t.type == "ERC-721") or t.type == "ERC-20" or t.type == "ERC-1155") and
                // (is_nil(tb.value_fetched_at) or is_nil(tb.value))
                Condition::all()
                    .add(
                        Condition::any()
                            .add(
                                Condition::any()
                                    .add(
                                        Column::AddressHash.ne(
                                            "0x0000000000000000000000000000000000000000"
                                                .as_bytes()
                                                .to_vec(),
                                        ),
                                    )
                                    .add(Column::TokenType.eq(Some("ERC-721".to_string()))),
                            )
                            .add(Column::TokenType.eq(Some("ERC-20".to_string())))
                            .add(Column::TokenType.eq(Some("ERC-1155".to_string()))),
                    )
                    .add(
                        Condition::any()
                            .add(Column::ValueFetchedAt.is_null())
                            .add(Column::Value.is_null()),
                    ),
            )
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
