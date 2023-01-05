use entity::scanner_height::Model;
use scanner::{
    cache::{ContractAddrCache, ScannerContract},
    evms::eth,
    model::{
        contract::Query as ContractQuery,
        height::{Mutation, Query},
    },
    orm::conn::connect_db, handler::block::current_height,
};
use sea_orm::DbConn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use web3::{
    transports::Http,
    types::{H160, U256},
    Web3,
};

#[tokio::main]
async fn main() -> web3::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "scanner=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let transport = web3::transports::Http::new("https://rpc.ankr.com/eth_goerli")?;
    let web3 = web3::Web3::new(transport);

    list_account_balances(&web3);

    let conn = connect_db("mysql://root:meta@localhost/rust_test".to_owned())
        .await
        .unwrap();

    let mut height = 0;
    if let Some(db_height) = current_height(&conn,"eth:5","eth").await{
        height =  db_height+ 1;
    }

    let contract_addr_cache = update_contract_cache(&conn).await;

    tracing::debug!("start handle height: {}", height);
    match eth::batch_get_tx_logs(height, &web3).await {
        (Some(ts), Some(logs)) => {
            for v in logs {
                if contract_addr_cache.exist(v.address.to_string()) {
                    tracing::debug!("catch log at ts: {}, detail : {:?}", ts, v);
                } else {
                    tracing::debug!(
                        "catch log at ts: {}, need not handler, topic : {:?}",
                        ts,
                        v.topics[0]
                    );
                }
            }
        }
        (_, _) => tracing::debug!("not found need hanlder tx in height: {}", height),
    }

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

async fn list_account_balances(web3: &Web3<Http>) -> Result<Vec<(H160, U256)>, web3::Error> {
    println!("Calling accounts.");
    let mut accounts = web3.eth().accounts().await?;
    println!("Accounts: {:?}", accounts);
    accounts.push("00a329c0648769a73afac7f9381e08fb43dbea72".parse().unwrap());

    println!("Calling balance.");
    let mut res: Vec<(H160, U256)> = Vec::new();
    for account in accounts {
        let balance = web3.eth().balance(account, None).await?;
        println!("Balance of {:?}: {}", account, balance);
        res.push((account, balance));
    }

    Ok(res)
}
