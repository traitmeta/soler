use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey};
use crate::account::{self, Account};
use hex::ToHex;

#[derive(Clone)]
pub struct RestClient{
    url:String,
}

impl RestClient{
    pub fn new(url:String) -> Self{
        Self{url}
    }

    /// Returns the sequence number and authentication key for an account
    pub fn account(&self, account_address:&str) -> serde_json::Value {
        let res = reqwest::blocking::get(format!("{}/accounts/{}",self.url,account_address)).unwrap();

        if res.status() !=200 {
            assert_eq!(res.status(),200,"{} - {}",res.text().unwrap_or("".to_string()),account_address);
        }

        res.json().unwrap()
    }

    /// Returns all resources associated with the account
    pub fn account_resource(&self, account_address:&str,resource_type:&str) -> Option<serde_json::Value>{
        let res = reqwest::blocking::get(format!(
            "{}/accounts/{}/resource/{}",
            self.url,account_address,resource_type,
        )).unwrap();
        
        if res.status() ==404{
            None
        }else if res.status()!=200{
            assert_eq!(res.status(),200,"{} - {}",res.text().unwrap_or("".to_string()),account_address);
            unreachable!()
        }else{
            Some(res.json().unwrap())
        }
    }

    pub fn generate_transaction(&self,sender :&str, payload:serde_json::Value) -> serde_json::Value{
        let account_res = self.account(sender);
        let seq_num = account_res.get("sequence_number")
        .unwrap().as_str().unwrap().parse::<u64>().unwrap();

        let expiration_time_secs = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs()+600;

        serde_json::json!({
            "sender":format!("0x{}",sender),
            "sequence_number": seq_num.to_string(),
            "max_gas_amount":"1000",
            "gas_unit_price":"1",
            "gas_currency_code": "XUS",
            "expiration_timestamp_secs": expiration_time_secs.to_string(),
            "payload": payload,
        })
    }

    pub fn sign_transaction(&self, account_from:&mut Account,mut txn_request:serde_json::Value)->serde_json::Value{
        let res = reqwest::blocking::Client::new()
        .post(format!("{}/tracsactions/signing_message",self.url))
        .body(txn_request.to_string())
        .send().unwrap();

        if res.status()!=200{
            assert_eq!(res.status(),200,"{} - {}",res.text().unwrap_or("".to_string()),txn_request.as_str().unwrap_or(""));
        }
        let body: serde_json::Value = res.json().unwrap();
        let to_sign_hex = Box::new(body.get("message").unwrap().as_str().unwrap());
        let to_sign = hex::decode(&to_sign_hex[2..]).unwrap();
        let signature:String = ExpandedSecretKey::from(&account_from.signing_key)
        .sign(&to_sign,&PublicKey::from(&account_from.signing_key)).encode_hex();

        let signature_payload = serde_json::json!({
            "type": "ed25519_signature",
            "public_key": format!("0x{}", account_from.pub_key()),
            "signature": format!("0x{}", signature),
        });
        txn_request
            .as_object_mut()
            .unwrap()
            .insert("signature".to_string(), signature_payload);
        txn_request
    }

    /// Submits a signed transaction to the blockchain.
    pub fn submit_transaction(&self, txn_request: &serde_json::Value) -> serde_json::Value {
        let res = reqwest::blocking::Client::new()
            .post(format!("{}/transactions", self.url))
            .body(txn_request.to_string())
            .header("Content-Type", "application/json")
            .send()
            .unwrap();

        if res.status() != 202 {
            assert_eq!(
                res.status(),
                202,
                "{} - {}",
                res.text().unwrap_or("".to_string()),
                txn_request.as_str().unwrap_or(""),
            );
        }
        res.json().unwrap()
    }

    /// Submits a signed transaction to the blockchain.
    pub fn execution_transaction_with_payload(
        &self,
        account_from: &mut Account,
        payload: serde_json::Value,
    ) -> String {
        let txn_request = self.generate_transaction(&account_from.address(), payload);
        let signed_txn = self.sign_transaction(account_from, txn_request);
        let res = self.submit_transaction(&signed_txn);
        res.get("hash").unwrap().as_str().unwrap().to_string()
    }

    pub fn transaction_pending(&self, transaction_hash: &str) -> bool {
        let res = reqwest::blocking::get(format!("{}/transactions/{}", self.url, transaction_hash))
            .unwrap();

        if res.status() == 404 {
            return true;
        }

        if res.status() != 200 {
            assert_eq!(
                res.status(),
                200,
                "{} - {}",
                res.text().unwrap_or("".to_string()),
                transaction_hash,
            );
        }

        res.json::<serde_json::Value>()
            .unwrap()
            .get("type")
            .unwrap()
            .as_str()
            .unwrap()
            == "pending_transaction"
    }

    /// Waits up to 10 seconds for a transaction to move past pending state.
    pub fn wait_for_transaction(&self, txn_hash: &str) {
        let mut count = 0;
        while self.transaction_pending(txn_hash) {
            assert!(count < 10, "transaction {} timed out", txn_hash);
            thread::sleep(Duration::from_secs(1));
            count += 1;
        }
    }

     /// Returns the test coin balance associated with the account
     pub fn account_balance(&self, account_address: &str) -> Option<u64> {
        self.account_resource(account_address, "0x1::Coin::CoinStore<0x1::TestCoin::TestCoin>")
            .unwrap()["data"]["coin"]["value"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
    }

    /// Transfer a given coin amount from a given Account to the recipient's account address.
    /// Returns the sequence number of the transaction used to transfer
    pub fn transfer(&self, account_from: &mut Account, recipient: &str, amount: u64) -> String {
        let payload = serde_json::json!({
            "type": "script_function_payload",
            "function": "0x1::Coin::transfer",
            "type_arguments": ["0x1::TestCoin::TestCoin"],
            "arguments": [format!("0x{}", recipient), amount.to_string()]
        });
        let txn_request = self.generate_transaction(&account_from.address(), payload);
        let signed_txn = self.sign_transaction(account_from, txn_request);
        let res = self.submit_transaction(&signed_txn);

        res.get("hash").unwrap().as_str().unwrap().to_string()
    }
}