use ::entities::log_receiver_chain;
use ::entities::log_receiver_chain::Entity as ScannerHeight;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_one_by_task_name(
        db: &DbConn,
        task_name: &str,
    ) -> Result<Option<log_receiver_chain::Model>, DbErr> {
        ScannerHeight::find_by_task_name(task_name).one(db).await
    }

    // If ok, returns (scanner height models, num pages).
    pub async fn find_scanner_height_in_page(
        db: &DbConn,
        page: u64,
        scanner_height_per_page: u64,
    ) -> Result<(Vec<log_receiver_chain::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = ScannerHeight::find()
            .order_by_asc(log_receiver_chain::Column::Id)
            .paginate(db, scanner_height_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_scanner_height(
        db: &DbConn,
        form_data: log_receiver_chain::Model,
    ) -> Result<log_receiver_chain::ActiveModel, DbErr> {
        log_receiver_chain::ActiveModel {
            task_name: Set(form_data.task_name.to_owned()),
            chain_name: Set(form_data.chain_name.to_owned()),
            height: Set(form_data.height.to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_height_by_id(
        db: &DbConn,
        id: u64,
        form_data: log_receiver_chain::Model,
    ) -> Result<log_receiver_chain::Model, DbErr> {
        let height: log_receiver_chain::ActiveModel = ScannerHeight::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        log_receiver_chain::ActiveModel {
            id: height.id,
            task_name: Set(form_data.task_name.to_owned()),
            chain_name: Set(form_data.chain_name.to_owned()),
            height: Set(form_data.height.to_owned()),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn update_height_by_task_name(
        db: &DbConn,
        task_name: &str,
        height: u64,
    ) -> Result<log_receiver_chain::Model, DbErr> {
        ScannerHeight::update(log_receiver_chain::ActiveModel {
            id: Unchanged(1),
            task_name: Unchanged(task_name.to_owned()),
            height: Set(height.to_owned()),
            ..Default::default()
        })
        .filter(log_receiver_chain::Column::TaskName.eq(task_name))
        .exec(db)
        .await
    }

    pub async fn delete_task(db: &DbConn, id: u64) -> Result<DeleteResult, DbErr> {
        let post: log_receiver_chain::ActiveModel = ScannerHeight::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_task(db: &DbConn) -> Result<DeleteResult, DbErr> {
        ScannerHeight::delete_many().exec(db).await
    }
}
