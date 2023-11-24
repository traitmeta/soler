use ::entities::address_token_balances::{ActiveModel, Column, Entity, Model};
use entities::{address_token_balances::Relation, tokens};
use migration::OnConflict;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn finds_by_type(
        db: &DbConn,
        address: Vec<u8>,
        token_type: String,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(
                Condition::all()
                    .add(Column::AddressHash.eq(address))
                    .add(Column::TokenType.eq(Some(token_type))),
            )
            .all(db)
            .await
    }

    pub async fn find_by_type_with_relation(
        db: &DbConn,
        address: Vec<u8>,
        token_type: String,
    ) -> Result<Vec<(Model, Option<tokens::Model>)>, DbErr> {
        Entity::find()
            .find_also_related(tokens::Entity)
            .filter(
                Condition::all()
                    .add(Column::AddressHash.eq(address))
                    .add(Column::TokenType.eq(Some(token_type))),
            )
            .all(db)
            .await
    }

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
        stmt = format!("{} ON CONFLICT (\"address_hash\", \"token_contract_address_hash\", COALESCE(\"token_id\", -1), \"block_number\") DO NOTHING",  stmt);
        db.execute(Statement::from_string(DatabaseBackend::Postgres, stmt))
            .await
    }

    pub async fn save_copy<C>(
        db: &C,
        form_datas: &[Model],
    ) -> Result<InsertResult<ActiveModel>, DbErr>
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

        let res = Entity::insert_many(datas)
            .on_conflict(
                // CREATE UNIQUE INDEX "fetched_token_balances" ON "public"."address_token_balances" USING btree (
                //     "address_hash" "pg_catalog"."bytea_ops" ASC NULLS LAST,
                //     "token_contract_address_hash" "pg_catalog"."bytea_ops" ASC NULLS LAST,
                //     COALESCE(token_id, '-1'::integer::numeric) "pg_catalog"."numeric_ops" ASC NULLS LAST,
                //     "block_number" "pg_catalog"."int8_ops" ASC NULLS LAST
                //   );
                OnConflict::columns([
                    Column::AddressHash,
                    Column::TokenContractAddressHash,
                    Column::TokenId,
                    // Expr::expr(Func::coalesce([
                    //     Expr::col(Column::TokenId).into(),
                    //     Expr::val(-1).into(),
                    // ]))
                    // .into(),
                    Column::BlockNumber,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await;

        if matches!(res, Err(DbErr::RecordNotInserted)) {
            return Ok(InsertResult { last_insert_id: 0 });
        }

        res
    }
}
