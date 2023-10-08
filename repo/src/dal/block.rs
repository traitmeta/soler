use ::entities::blocks::{ActiveModel, Column, Entity as Blocks, Model};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_latest(db: &DbConn) -> Result<Option<Model>, DbErr> {
        Blocks::find()
            .order_by_desc(Column::Number)
            .limit(1)
            .one(db)
            .await
    }

    pub async fn find_by_hash(db: &DbConn, hash: &str) -> Result<Option<Model>, DbErr> {
        Blocks::find().filter(Column::Hash.eq(hash)).one(db).await
    }

    // If ok, returns (scanner height models, num pages).
    pub async fn find_in_page(
        db: &DbConn,
        page: u64,
        blocks_per_page: u64,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Blocks::find()
            .order_by_asc(Column::Number)
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
        ActiveModel {
            difficulty: Set(form_data.difficulty),
            gas_limit: Set(form_data.gas_limit),
            gas_used: Set(form_data.gas_used),
            hash: Set(form_data.hash.to_owned()),
            miner_hash: Set(form_data.miner_hash.to_owned()),
            nonce: Set(form_data.nonce.to_owned()),
            number: Set(form_data.number),
            parent_hash: Set(form_data.parent_hash.to_owned()),
            size: Set(form_data.size),
            timestamp: Set(form_data.timestamp),
            base_fee_per_gas: Set(form_data.base_fee_per_gas),
            total_difficulty: Set(form_data.total_difficulty),
            consensus: Set(form_data.consensus),
            refetch_needed: Set(form_data.refetch_needed),
            is_empty: Set(form_data.is_empty),
            inserted_at: Set(form_data.inserted_at),
            updated_at: Set(form_data.updated_at),
            ..Default::default()
        }
        .insert(db)
        .await
    }
}