use ethers::providers::{Provider,Http, Middleware};
use repo::{
    dal::{contract::Query as ContractQuery, height::Mutation},
    orm::conn::connect_db,
};
use scanner::{
    cache::{ContractAddrCache, ScannerContract},
    evms::eth,
    handler::block::current_height,
};
use sea_orm::DbConn;
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use web3::{
    types::{H160, U256},
    Web3,
};
use config::db::DB;

#[tokio::main]
#[instrument]
async fn main() -> web3::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "scanner=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let provider = Provider::try_from("https://rpc.ankr.com/eth_goerli").unwrap();

    list_account_balances(&provider).await?;

    let db_cfg = DB {
        url: "localhost:3306".to_string(),
        schema: "mysql".to_string(),
        username: "root".to_string(),
        password: "meta".to_string(),
        database: "rust_test".to_string(),
    };
    let conn = connect_db(db_cfg).await.unwrap();

    let mut height = 0;
    if let Some(db_height) = current_height(&conn, "eth:5", "eth").await {
        height = db_height + 1;
    }

    let contract_addr_cache = update_contract_cache(&conn).await;

    tracing::debug!("start handle height: {}", height);
    let res = Mutation::update_height_by_task_name(&conn, "eth:5", height).await;
    match res {
        Err(e) => tracing::debug!("hanlder height {} failed,err:{}", height, e),
        _ => tracing::debug!("hanlded height: {}", height),
    }

    Ok(())
}

async fn update_contract_cache(conn: &DbConn) -> ContractAddrCache {
    let mut contract_addr_cache = ContractAddrCache::new();
    let (contracts, _) = ContractQuery::find_scanner_contract_in_page(conn, 1, 100)
        .await
        .unwrap();
    for v in contracts {
        let data = ScannerContract {
            chain_name: v.chain_name,
            chain_id: v.chain_id,
            address: v.address,
            event_sign: v.event_sign,
        };
        contract_addr_cache.insert(data.cache_key(), data);
    }

    contract_addr_cache
}

async fn list_account_balances(web3: &Provider<Http>) -> Result<Vec<(H160, U256)>, web3::Error> {
    println!("Calling accounts.");
    let mut accounts = web3.get_accounts().await.unwrap();
    println!("Accounts: {accounts:?}");
    accounts.push("00a329c0648769a73afac7f9381e08fb43dbea72".parse().unwrap());

    println!("Calling balance.");
    let mut res: Vec<(H160, U256)> = Vec::new();
    for account in accounts {
        let balance = web3.get_balance(account, None).await.unwrap();
        println!("Balance of {account:?}: {balance}");
        res.push((account, balance));
    }

    Ok(res)
}
