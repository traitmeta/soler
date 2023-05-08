use super::*;
pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

#[derive(Serialize)]
pub struct BaseResponse<T> {
    pub code: u64,
    pub message: String,
    pub data: T,
}

impl<T> BaseResponse<T> {
    pub fn success(data: T) -> Self {
        BaseResponse {
            code: 200,
            message: "OK".to_owned(),
            data,
        }
    }
}
