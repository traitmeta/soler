use super::{
    err::AppError,
    response::BaseResponse,
    state::{get_conn, AppState},
    Json,
};
use ::entities::user;
use axum::{extract::Query, response::IntoResponse, Extension};
use hyper::StatusCode;
use repo::dal::user::{Mutation as UserMutation, Query as UserQuery};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    name: String,
    address: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserParams {
    pub name: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsersParams {
    pub name: String,
    pub page_size: Option<usize>,
    pub page: Option<usize>,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    name: String,
    address: String,
}

pub async fn create_user(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let conn = get_conn(&state);

    // insert your application logic here
    let user_model = user::Model {
        id: 0,
        user_name: payload.name.to_owned(),
        user_address: payload.address.to_owned(),
        user_email: payload.email.to_owned(),
        created_at: None,
        updated_at: None,
    };
    let _create_res = UserMutation::create_user(conn, user_model)
        .await
        .map_err(AppError::from);

    let res = User {
        name: payload.name.to_owned(),
        address: payload.address.to_owned(),
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(BaseResponse::success(res)))
}

pub async fn get_user_info(
    Extension(state): Extension<Arc<AppState>>,
    Query(params): Query<UsersParams>,
) -> Result<Json<BaseResponse<User>>, AppError> {
    let conn = get_conn(&state);
    let user_info = UserQuery::select_one_by_user_name(conn, &params.name)
        .await
        .map_err(AppError::from)?;
    let mut res = User {
        name: "".to_string(),
        address: "".to_string(),
    };

    if let Some(user) = user_info {
        res.address = user.user_address;
        res.name = user.user_name;
    }

    Ok(Json(BaseResponse::success(res)))
}
