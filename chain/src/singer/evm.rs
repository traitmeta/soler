use std::io::Read;

// 使用EVM的签名
use anyhow::Result; //导入包

use ethers_core::{
    k256::ecdsa::{self, SigningKey},
    rand::thread_rng,
    utils::keccak256,
};
use ethers_signers::{to_eip155_v, LocalWallet, Signer, Wallet};

fn get_wallet_from_key_file() -> Wallet<SigningKey> {
    let dir = "./keystore/key"; //keystore的钱包路径
    let wallet = Wallet::<SigningKey>::decrypt_keystore(&dir, "123456").unwrap(); //参数2是钱包密码
    wallet
}

fn random_wallet() -> Wallet<SigningKey> {
    let wallet = LocalWallet::new(&mut thread_rng());
    let wallet = wallet.with_chain_id(1u64);
    wallet
}

pub fn sign_msg(wallet: Wallet<SigningKey>) {
    let digest = md5::compute(b"\"hello2\"");
    let k256 = keccak256(&digest[0..8]).into();
    let sig = wallet.sign_hash(k256).unwrap(); //里面有对recover_id加27操作
    to_eip155_v(sig.v as u8 - 27, 1); // sig.v = sig.v - 27;
    let signstr = sig.to_vec();
    println!("{:?} {:?}", digest.bytes(), hex::encode(signstr));
}

pub async fn sign_hash(wallet: Wallet<SigningKey>) {
    // const PREFIX: &str = "\x19Ethereum Signed Message:\n";
    let signature = wallet.sign_message("hello world").await.unwrap();
    signature.verify("hello world", wallet.address()).unwrap()
}
