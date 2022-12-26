use ::entity::scanner_contract;
use ::entity::scanner_contract::Entity as ScannerContract;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_one(
        db: &DbConn,
        task_name: &str,
    ) -> Result<Option<scanner_contract::Model>, DbErr> {
        ScannerContract::find()
            .filter(scanner_contract::Column::ChainId.contains(task_name))
            .one(db)
            .await
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_scanner_contract_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<scanner_contract::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = ScannerContract::find()
            .order_by_asc(scanner_contract::Column::Id)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_scanner_contract(
        db: &DbConn,
        form_data: scanner_contract::Model,
    ) -> Result<scanner_contract::ActiveModel, DbErr> {
        scanner_contract::ActiveModel {
            id: Set(0),
            chain_id: Set(form_data.chain_id),
            chain_name: Set(form_data.chain_name.to_owned()),
            address: Set(form_data.address.to_owned()),
            event_sign: Set(form_data.event_sign.to_owned()),
        }
        .save(db)
        .await
    }

    pub async fn update_height_by_id(
        db: &DbConn,
        id: u64,
        form_data: scanner_contract::Model,
    ) -> Result<scanner_contract::Model, DbErr> {
        let event: scanner_contract::ActiveModel = ScannerContract::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        scanner_contract::ActiveModel {
            id: event.id,
            chain_id: Set(form_data.chain_id.to_owned()),
            chain_name: Set(form_data.chain_name.to_owned()),
            address: Set(form_data.address.to_owned()),
            event_sign: Set(form_data.event_sign.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_task(db: &DbConn, id: u64) -> Result<DeleteResult, DbErr> {
        let post: scanner_contract::ActiveModel = ScannerContract::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_task(db: &DbConn) -> Result<DeleteResult, DbErr> {
        ScannerContract::delete_many().exec(db).await
    }
}
