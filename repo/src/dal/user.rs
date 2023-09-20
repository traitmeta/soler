use ::entities::user;
use ::entities::user::Entity as User;
use sea_orm::{DbConn, DbErr, Set,*};

pub struct Query;

impl Query {
    pub async fn select_one_by_user_name(
        db: &DbConn,
        user_name: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find_by_user_name(user_name).one(db).await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_user(
        db: &DbConn,
        form_data: user::Model,
    ) -> Result<user::ActiveModel, DbErr> {
        user::ActiveModel {
            user_name: Set(form_data.user_name.to_owned()),
            user_address: Set(form_data.user_address.to_owned()),
            user_email: Set(form_data.user_email.to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_height_by_id(
        db: &DbConn,
        id: u64,
        form_data: user::Model,
    ) -> Result<user::Model, DbErr> {
        let user_info: user::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

            user::ActiveModel {
            id: user_info.id,
            user_name: Set(form_data.user_name.to_owned()),
            user_address: Set(form_data.user_address.to_owned()),
            user_email: Set(form_data.user_email.to_owned()),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn update_address_by_user_name(
        db: &DbConn,
        user_name: &str,
        address: &str,
    ) -> Result<user::Model, DbErr> {
        User::update(user::ActiveModel {
            id: Unchanged(0),
            user_name: Unchanged(user_name.to_owned()),
            user_address: Set(address.to_owned()),
            ..Default::default()
        })
        .filter(user::Column::UserName.eq(user_name))
        .exec(db)
        .await
    }

    pub async fn delete_user(db: &DbConn, id: u64) -> Result<DeleteResult, DbErr> {
        let post: user::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_user(db: &DbConn) -> Result<DeleteResult, DbErr> {
        User::delete_many().exec(db).await
    }
}
