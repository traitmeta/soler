use entity::scanner_height::Model;
use scanner::{
    cache::{ContractAddrCache, ScannerContract},
    evms::eth,
    model::{
        contract::Query as ContractQuery,
        height::{Mutation, Query},
    },
    orm::conn::connect_db,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

//    println!("Calling accounts.");
//    let mut accounts = web3.eth().accounts().await?;
//    println!("Accounts: {:?}", accounts);
//    accounts.push("00a329c0648769a73afac7f9381e08fb43dbea72".parse().unwrap()`);

//    println!("Calling balance.");
//    for account in accounts {
//        let balance = web3.eth().balance(account, None).await?;
//        println!("Balance of {:?}: {}", account, balance);
//    }

    let conn = connect_db("mysql://root:meta@localhost/rust_test".to_owned())
        .await
        .unwrap();
    let mut height = 1999;

    let current_model = Query::select_one(&conn, "eth:5").await.unwrap();
    match current_model {
        Some(current) => height = current.height + 1,
        None => {
            tracing::debug!("not found eth:5");
            let insert_data = Model {
                id: 0,
                task_name: "eth:5".to_owned(),
                chain_name: "eth".to_owned(),
                height: 1998,
                created_at: None,
                updated_at: None,
            };
            let result = Mutation::create_scanner_height(&conn, insert_data)
                .await
                .expect("insert eth:5 to scanner height table err");
            tracing::debug!("insert eth:5 return :{:?}", result);
        }
    }

    let mut contract_addr_cache = ContractAddrCache::new();
    let (contracts, _) = ContractQuery::find_scanner_contract_in_page(&conn, 1, 100)
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
        _ => tracing::debug!("not found need hanlder tx in height: {}", height),
    }

    Mutation::update_height_by_task_name(&conn, "eth:5", height).await;

    Ok(())
}
