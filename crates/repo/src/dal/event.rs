use ::entities::logs::{ActiveModel, Column, Entity as Events, Model};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_by_tx_hash(db: &DbConn, hash: Vec<u8>) -> Result<Vec<Model>, DbErr> {
        Events::find()
            .filter(Column::TransactionHash.eq(hash))
            .all(db)
            .await
    }

    pub async fn find_by_hash(db: &DbConn, hash: Vec<u8>) -> Result<Option<Model>, DbErr> {
        Events::find()
            .filter(Column::AddressHash.eq(hash))
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
        let paginator = Events::find()
            .order_by_asc(Column::Index)
            .paginate(db, blocks_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create<C>(db: &C, form_datas: &[Model]) -> Result<InsertResult<ActiveModel>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut datas = vec![];
        for form_data in form_datas.iter() {
            let data = ActiveModel {
                data: Set(form_data.data.clone()),
                index: Set(form_data.index),
                r#type: Set(form_data.r#type.to_owned()),
                first_topic: Set(form_data.first_topic.to_owned()),
                second_topic: Set(form_data.second_topic.to_owned()),
                third_topic: Set(form_data.first_topic.to_owned()),
                fourth_topic: Set(form_data.fourth_topic.to_owned()),
                inserted_at: Unchanged(form_data.inserted_at),
                updated_at: Set(form_data.updated_at),
                address_hash: Set(form_data.address_hash.to_owned()),
                transaction_hash: Set(form_data.transaction_hash.to_owned()),
                block_hash: Set(form_data.block_hash.to_owned()),
                block_number: Set(form_data.block_number),
            };
            datas.push(data);
        }

        if datas.is_empty() {
            return Err(DbErr::RecordNotInserted);
        }

        Events::insert_many(datas).exec(db).await
    }
}
