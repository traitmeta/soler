use axum::{Form, response::{Html, IntoResponse}, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

pub async fn show_form() -> Html<&'static str> { 
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post">
                    <label for="name">
                        Enter your name:
                        <input type="text" name="name">
                    </label>
                    <label>
                        Enter your email:
                        <input type="text" name="email">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub name: String,
}

// its error, cannot parse param from body
pub async fn accept_form(Form(input): Form<UserInfo>) -> impl IntoResponse {
    (StatusCode::OK, Json(Response { name: input.name }))
}
