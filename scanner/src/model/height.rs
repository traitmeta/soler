use ::entity::scanner_height;
use ::entity::scanner_height::Entity as ScannerHeight;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_one(
        db: &DbConn,
        task_name: &str,
    ) -> Result<Option<scanner_height::Model>, DbErr> {
        ScannerHeight::find()
            .filter(scanner_height::Column::TaskName.contains(task_name))
            .one(db)
            .await
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_scanner_height_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<scanner_height::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = ScannerHeight::find()
            .order_by_asc(scanner_height::Column::Id)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_scanner_height(
        db: &DbConn,
        form_data: scanner_height::Model,
    ) -> Result<scanner_height::ActiveModel, DbErr> {
        scanner_height::ActiveModel {
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
        form_data: scanner_height::Model,
    ) -> Result<scanner_height::Model, DbErr> {
        let height: scanner_height::ActiveModel = ScannerHeight::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        scanner_height::ActiveModel {
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
    ) -> Result<scanner_height::Model, DbErr> {
        ScannerHeight::update(scanner_height::ActiveModel {
            id:Unchanged(1),
            task_name: Unchanged(task_name.to_owned()),
            height: Set(height.to_owned()),
            ..Default::default()
        })
        .filter(scanner_height::Column::TaskName.eq(task_name))
        .exec(db)
        .await
    }

    pub async fn delete_task(db: &DbConn, id: u64) -> Result<DeleteResult, DbErr> {
        let post: scanner_height::ActiveModel = ScannerHeight::find_by_id(id)
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
