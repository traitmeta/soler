use super::*;

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
#[allow(dead_code)]
pub struct Input {
    name: String,
    email: String,
}

pub async fn accept_form(Form(input): Form<Input>) {
    dbg!(&input);
}