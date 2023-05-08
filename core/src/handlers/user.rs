use super::{common::BaseResponse, *};

pub async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    // insert your application logic here
    let user = User {
        id: 919,
        name: payload.name,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(BaseResponse::success(user)))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    name: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    id: u64,
    name: String,
}
