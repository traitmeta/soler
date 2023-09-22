use ::entities::blocks;
use ::entities::blocks::Entity as Blocks;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_latest(
        db: &DbConn,
    ) -> Result<Option<blocks::Model>, DbErr> {
        Blocks::find_latest().one(db).await
    }

    // If ok, returns (scanner height models, num pages).
    pub async fn find_blocks_in_page(
        db: &DbConn,
        page: u64,
        blocks_per_page: u64,
    ) -> Result<(Vec<blocks::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Blocks::find()
            .order_by_asc(blocks::Column::Number)
            .paginate(db, blocks_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_block(
        db: &DbConn,
        form_data: blocks::Model,
    ) -> Result<blocks::ActiveModel, DbErr> {
        blocks::ActiveModel {
            difficulty: Set(form_data.difficulty),
            gas_limit: Set(form_data.gas_limit),
            gas_used: Set(form_data.gas_used),
            hash: Set(form_data.hash),
            miner_hash:  Set(form_data.miner_hash),
            nonce: Set(form_data.nonce),
            number: Set(form_data.number),
            parent_hash: Set(form_data.parent_hash),
            size: Set(form_data.size),
            timestamp: Set(form_data.timestamp),
            base_fee_per_gas: Set(form_data.base_fee_per_gas),
            ..Default::default()
        }
        .save(db)
        .await
    }
}
