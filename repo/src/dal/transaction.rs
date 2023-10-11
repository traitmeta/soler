use ::entities::transactions::{ActiveModel, Column, Entity, Model};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_by_hash(db: &DbConn, hash: Vec<u8>) -> Result<Option<Model>, DbErr> {
        Entity::find().filter(Column::Hash.eq(hash)).one(db).await
    }

    pub async fn find_in_page_block(
        db: &DbConn,
        block_height: Option<i64>,
        page: Option<u64>,
        blocks_per_page: Option<u64>,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        // Setup paginator
        let mut query = Entity::find();
        if block_height.is_none() {
            query = query.filter(Column::BlockNumber.eq(block_height));
        }
        query = query
            .order_by(Column::BlockNumber, Order::Desc)
            .order_by(Column::Index, Order::Desc);

        let per_pages = match blocks_per_page {
            Some(p) => p,
            None => 20,
        };

        let paginator = query.paginate(db, per_pages);
        let num_pages = paginator.num_pages().await?;
        let real_page = match page {
            Some(p) => p,
            None => 1,
        };
        // Fetch paginated posts
        paginator
            .fetch_page(real_page - 1)
            .await
            .map(|p| (p, num_pages))
    }

    // If ok, returns (scanner height models, num pages).
    pub async fn find_in_page(
        db: &DbConn,
        page: u64,
        blocks_per_page: u64,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Entity::find()
            .order_by_asc(Column::Index)
            .paginate(db, blocks_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create<C>(
        db: &C,
        form_datas: &Vec<Model>,
    ) -> Result<InsertResult<ActiveModel>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut batch = vec![];
        for form_data in form_datas.iter() {
            let data = ActiveModel {
                cumulative_gas_used: Set(form_data.cumulative_gas_used),
                error: Set(form_data.error.to_owned()),
                gas: Set(form_data.gas),
                gas_price: Set(form_data.gas_price),
                gas_used: Set(form_data.gas_used),
                hash: Set(form_data.hash.to_owned()),
                index: Set(form_data.index),
                input: Set(form_data.input.to_owned()),
                nonce: Set(form_data.nonce),
                r: Set(form_data.r.to_owned()),
                s: Set(form_data.s.to_owned()),
                status: Set(form_data.status),
                v: Set(form_data.v),
                value: Set(form_data.value),
                block_hash: Set(form_data.block_hash.to_owned()),
                block_number: Set(form_data.block_number),
                from_address_hash: Set(form_data.from_address_hash.to_owned()),
                to_address_hash: Set(form_data.to_address_hash.to_owned()),
                created_contract_address_hash: Set(form_data
                    .created_contract_address_hash
                    .to_owned()),
                created_contract_code_indexed_at: Set(form_data.created_contract_code_indexed_at),
                earliest_processing_start: Set(form_data.earliest_processing_start),
                old_block_hash: Set(form_data.old_block_hash.to_owned()),
                revert_reason: Set(form_data.revert_reason.to_owned()),
                max_priority_fee_per_gas: Set(form_data.max_priority_fee_per_gas),
                max_fee_per_gas: Set(form_data.max_fee_per_gas),
                r#type: Set(form_data.r#type),
                has_error_in_internal_txs: Set(form_data.has_error_in_internal_txs),
                inserted_at: Set(form_data.inserted_at),
                updated_at: Set(form_data.updated_at),
                ..Default::default()
            };
            batch.push(data);
        }

        Entity::insert_many(batch).exec(db).await
    }
}
