// 使用EVM的签名
use anyhow::Result; //导入包

use ethers_core::{
    k256::ecdsa::{self, SigningKey},
    rand::thread_rng,
    utils::keccak256,
};
use ethers_signers::{to_eip155_v, LocalWallet, Signer, Wallet};
use std::path::Path;

fn get_wallet_from_key_file() -> Wallet {
    let dir = "./keystore/key"; //keystore的钱包路径
    let wallet = Wallet::<SigningKey>::decrypt_keystore(&dir, "123456")?; //参数2是钱包密码
    wallet
}

fn random_wallet() -> wallet{
    let wallet = LocalWallet::new(&mut thread_rng());
    let wallet = wallet.with_chain_id(1u64);
    wallet
}

fn sign_msg() {
    let digest = md5::compute(b"\"hello2\"");
    let k256 = keccak256(&digest[0..8]).into();
    let mut sig = wallet.sign_hash(k256);   //里面有对recover_id加27操作
    sig.v = sig.v - 27;             // to_eip155_v(sig.v as u8 - 27, 1);
    let signstr = sig.to_vec();
    println!("{:?} {:?}", key, hex::encode(signstr));
}

fn sign_hash() {
    // const PREFIX: &str = "\x19Ethereum Signed Message:\n";
    let signature = wallet.sign_message("hello world").await?;
    signature.verify("hello world", wallet.address()).unwrap();
}
