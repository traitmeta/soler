use ::entities::tokens::{ActiveModel, Column, Entity, Model};
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

    pub async fn find_by_hash(db: &DbConn, hash: &str) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::ContractAddressHash.eq(hash))
            .one(db)
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
}

pub struct Mutation;

impl Mutation {
    pub async fn create<C>(db: &C, form_data: &Model) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let model = form_data.clone().into_active_model();
        model.insert(db).await
    }

    pub async fn update_metadata<C>(db: &C, form_data: &Model) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            name: Set(form_data.name.to_owned()),
            symbol: Set(form_data.symbol.to_owned()),
            total_supply: Unchanged(form_data.total_supply),
            decimals: Set(form_data.decimals),
            r#type: Unchanged(form_data.r#type.to_owned()),
            cataloged: Unchanged(form_data.cataloged),
            contract_address_hash: Unchanged(form_data.contract_address_hash.to_vec()),
            inserted_at: Unchanged(form_data.inserted_at),
            updated_at: Unchanged(form_data.updated_at),
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

    // TODO replace Unchanged to Default
    pub async fn update_total_supply<C>(db: &C, form_data: &Model) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        // Bulk set attributes using ActiveModel
        let mut token = form_data.clone().into_active_model();
        token.total_supply = Set(form_data.total_supply);
        token.total_supply_updated_at_block= Set(form_data.total_supply_updated_at_block);

        Entity::update(token)
            .filter(Column::ContractAddressHash.eq(form_data.contract_address_hash.to_vec()))
            .exec(db)
            .await
        // ActiveModel {
        //     name: Unchanged(form_data.name.to_owned()),
        //     symbol: Unchanged(form_data.symbol.to_owned()),
        //     total_supply: Set(form_data.total_supply),
        //     decimals: Unchanged(form_data.decimals),
        //     r#type: Unchanged(form_data.r#type.to_owned()),
        //     cataloged: Unchanged(form_data.cataloged),
        //     contract_address_hash: Unchanged(form_data.contract_address_hash.to_vec()),
        //     inserted_at: Unchanged(form_data.inserted_at),
        //     updated_at: Unchanged(form_data.updated_at),
        //     holder_count: Unchanged(form_data.holder_count),
        //     skip_metadata: Unchanged(form_data.skip_metadata),
        //     fiat_value: Unchanged(form_data.fiat_value),
        //     circulating_market_cap: Unchanged(form_data.circulating_market_cap),
        //     total_supply_updated_at_block: Set(form_data.total_supply_updated_at_block),
        //     icon_url: Unchanged(form_data.icon_url.to_owned()),
        //     is_verified_via_admin_panel: Unchanged(form_data.is_verified_via_admin_panel),
        // }
        // .update(db)
        // .await
    }
}
