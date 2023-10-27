use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:16379").await?;
    client.set("hello", "world".into()).await?;
    let res = client.get("hello").await?;
    println!("get resut form server = {:?}", res);
    Ok(())
}
