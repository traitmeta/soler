use crate::rest::RestClient;

pub struct FaucetClient {
    url: String,
    rest_client: RestClient,
}

impl FaucetClient {
    /// Faucet creates and funds accounts. This is a thin wrapper around that.
    pub fn new(url: String, rest_client: RestClient) -> Self {
        Self { url, rest_client }
    }

    /// This creates an account if it does not exist and mints the specified amount of coins into that account.
    pub fn fund_account(&self, auth_key: &str, amount: u64) {
        let res = reqwest::blocking::Client::new()
            .post(format!(
                "{}/mint?amount={}&auth_key={}",
                self.url, amount, auth_key
            ))
            .send()
            .unwrap();

        if res.status() != 200 {
            assert_eq!(
                res.status(),
                200,
                "{}",
                res.text().unwrap_or("".to_string()),
            );
        }
        for txn_hash in res.json::<serde_json::Value>().unwrap().as_array().unwrap() {
            self.rest_client
                .wait_for_transaction(txn_hash.as_str().unwrap())
        }
    }
}