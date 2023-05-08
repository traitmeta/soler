use super::*;

// basic handler that responds with a static string
pub async fn info() -> &'static str {
    "INFO: OK!"
}

pub async fn home() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
