use ::entities::tokens::{ActiveModel, Column, Entity, Model};
use chrono::Utc;
use migration::OnConflict;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_latest(db: &DbConn) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .order_by_desc(Column::ContractAddressHash)
            .limit(1)
            .one(db)
            .await
    }

    pub async fn find_by_hash(db: &DbConn, hash: Vec<u8>) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::ContractAddressHash.eq(hash))
            .one(db)
            .await
    }

    pub async fn find_by_contract_addresses(
        db: &DbConn,
        addresses: Vec<Vec<u8>>,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::ContractAddressHash.is_in(addresses))
            .all(db)
            .await
    }

    // If ok, returns (scanner height models, num pages).
    pub async fn find_in_page(
        db: &DbConn,
        page: u64,
        blocks_per_page: u64,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Entity::find()
            .order_by_asc(Column::ContractAddressHash)
            .paginate(db, blocks_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn filter_not_skip_metadata(db: &DbConn, r_type: &str) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(
                Condition::any()
                    .add(Column::SkipMetadata.eq(Some(false)))
                    .add(Column::SkipMetadata.is_null()),
            )
            .filter(Column::Type.eq(r_type.to_string()))
            .limit(50)
            .all(db)
            .await
    }

    pub async fn filter_uncataloged_and_no_skip_metadata(
        db: &DbConn,
        r_type: &str,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(
                Condition::all()
                    .add(
                        Condition::any()
                            .add(Column::Cataloged.eq(Some(false)))
                            .add(Column::Cataloged.is_null()),
                    )
                    .add(
                        Condition::any()
                            .add(Column::SkipMetadata.eq(Some(false)))
                            .add(Column::SkipMetadata.is_null()),
                    ),
            )
            .filter(Column::Type.eq(r_type.to_string()))
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
            let model = form_data.clone().into_active_model();
            datas.push(model);
        }

        if datas.is_empty() {
            return Err(DbErr::RecordNotInserted);
        }

        let res = Entity::insert_many(datas)
            .on_conflict(
                OnConflict::column(Column::ContractAddressHash)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        if matches!(res, Err(DbErr::RecordNotInserted)) {
            return Ok(InsertResult {
                last_insert_id: vec![],
            });
        }

        res
    }
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

    pub async fn update_metadata<C>(db: &C, form_data: &Model) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            name: Set(form_data.name.to_owned()),
            symbol: Set(form_data.symbol.to_owned()),
            total_supply: Set(form_data.total_supply.to_owned()),
            decimals: Set(form_data.decimals),
            r#type: Unchanged(form_data.r#type.to_owned()),
            cataloged: Set(form_data.cataloged),
            contract_address_hash: Unchanged(form_data.contract_address_hash.to_vec()),
            inserted_at: Unchanged(form_data.inserted_at),
            updated_at: Set(Utc::now().naive_utc()),
            holder_count: Unchanged(form_data.holder_count),
            skip_metadata: Set(form_data.skip_metadata),
            fiat_value: Unchanged(form_data.fiat_value),
            circulating_market_cap: Unchanged(form_data.circulating_market_cap),
            total_supply_updated_at_block: Unchanged(form_data.total_supply_updated_at_block),
            icon_url: Unchanged(form_data.icon_url.to_owned()),
            is_verified_via_admin_panel: Unchanged(form_data.is_verified_via_admin_panel),
        }
        .update(db)
        .await
    }

    pub async fn update_total_supply<C>(db: &C, form_data: &Model) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        // Bulk set attributes using ActiveModel
        let mut token = form_data.clone().into_active_model();
        token.total_supply = Set(form_data.total_supply.to_owned());
        token.total_supply_updated_at_block = Set(form_data.total_supply_updated_at_block);
        token.updated_at = Set(Utc::now().naive_utc());

        Entity::update(token)
            .filter(Column::ContractAddressHash.eq(form_data.contract_address_hash.to_vec()))
            .exec(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::orm::conn::connect_db;
    use ::entities::tokens::Model;
    use bigdecimal::{BigDecimal, FromPrimitive};
    use chrono::Utc;
    use config::db::DB;
    use ethers::types::H160;
    use sea_orm::prelude::Decimal;

    use super::{Mutation, Query};

    fn setup_database() -> DB {
        DB {
            url: "172.22.215.113:5432".to_string(),
            schema: "postgres".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "soler".to_string(),
            log_level: 3,
        }
    }

    #[test]
    #[ignore]
    fn test_create() {
        let db_cfg = setup_database();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let conn = rt.block_on(connect_db(db_cfg)).unwrap();
        let model = Model {
            name: None,
            symbol: None,
            total_supply: None,
            decimals: None,
            r#type: "ERC20".to_string(),
            cataloged: None,
            contract_address_hash: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
                .as_bytes()
                .to_vec(),
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            holder_count: None,
            skip_metadata: None,
            fiat_value: None,
            circulating_market_cap: None,
            total_supply_updated_at_block: None,
            icon_url: None,
            is_verified_via_admin_panel: None,
        };
        if let Ok(db_model) = rt.block_on(Mutation::create(&conn, &vec![model])) {
            println!("{:?}", db_model);
        }
    }

    #[test]
    #[ignore]
    fn test_update_total_supply() {
        let db_cfg = setup_database();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let conn = rt.block_on(connect_db(db_cfg)).unwrap();
        let addr: H160 = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .unwrap();
        let mut db_model = rt
            .block_on(Query::find_by_hash(&conn, addr.as_bytes().to_vec()))
            .unwrap()
            .unwrap();

        db_model.total_supply = BigDecimal::from_i32(101);
        db_model.total_supply_updated_at_block = Some(1993);
        rt.block_on(Mutation::update_total_supply(&conn, &db_model))
            .unwrap();
    }

    #[test]
    #[ignore]
    fn test_update_total_supply_without_query() {
        let db_cfg = setup_database();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let conn = rt.block_on(connect_db(db_cfg)).unwrap();
        let model = Model {
            name: None,
            symbol: None,
            total_supply: BigDecimal::from_i32(666),
            decimals: None,
            r#type: "ERC20".to_string(),
            cataloged: None,
            contract_address_hash: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
                .as_bytes()
                .to_vec(),
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            holder_count: None,
            skip_metadata: Some(false),
            fiat_value: None,
            circulating_market_cap: None,
            total_supply_updated_at_block: Some(222),
            icon_url: None,
            is_verified_via_admin_panel: None,
        };
        if let Ok(db_model) = rt.block_on(Mutation::update_total_supply(&conn, &model)) {
            println!("{:?}", db_model);
        }
    }
}
