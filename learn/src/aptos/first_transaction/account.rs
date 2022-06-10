use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey};
use rand::{rngs::OsRng, Rng, SeedableRng};
use tiny_keccak::{Hasher, Sha3};

pub struct Account {
    pub signing_key :SecretKey,
}

impl Account{
    pub fn new(priv_key_bytes: Option<Vec<u8>>) -> Self{
        let signing_key = match priv_key_bytes{
            Some(key) => SecretKey::from_bytes(&key).unwrap(),
            None => SecretKey::generate(&mut rand::rngs::StdRng::from_seed(OsRng.gen())),
        };
        Account{signing_key}
    }

    pub fn address(&self) -> String{
        self.auth_key()
    }

    pub fn auth_key(&self)->String{
        let mut sha3 = Sha3::v256();
        sha3.update(PublicKey::from(&self.signing_key).as_bytes());
        sha3.update(&vec![0u8]);

        let mut output = [0u8;32];
        sha3.finalize(&mut output);
        hex::encode(output)
    }

    pub fn pub_key(&self) -> String{
        hex::encode(PublicKey::from(&self.signing_key).as_bytes())
    }
}