use ::entities::blocks;
use ::entities::blocks::Entity as Blocks;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn select_one_by_task_name(
        db: &DbConn,
        task_name: &str,
    ) -> Result<Option<scanner_height::Model>, DbErr> {
        Blocks::find_by_task_name(task_name).one(db).await
    }

    // If ok, returns (scanner height models, num pages).
    pub async fn find_scanner_height_in_page(
        db: &DbConn,
        page: u64,
        scanner_height_per_page: u64,
    ) -> Result<(Vec<scanner_height::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Blocks::find()
            .order_by_asc(scanner_height::Column::Id)
            .paginate(db, scanner_height_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_block(
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
}
