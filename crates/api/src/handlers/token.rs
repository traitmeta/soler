/*
    When process in api, need to rename transfer type
    - mint => to not zero and from is zero
    - burn => from not zero and to is zero
    - create => from and to is zero
    - transfer => from and to is not zero
*/
pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub const TOKEN_BURN: &str = "Token Burning";
pub const TOKEN_MINT: &str = "Token Minting";
pub const TOKEN_TRANSFER: &str = "Token Transfer";
pub const TOKEN_CREATION: &str = "Token Creation";

pub fn rename_transfer_type(from: Vec<u8>, to: Vec<u8>) -> String {
    if from == ZERO_ADDRESS.as_bytes().to_vec() {
        if to == ZERO_ADDRESS.as_bytes().to_vec() {
            return TOKEN_CREATION.to_string();
        } else {
            return TOKEN_MINT.to_string();
        }
    } else if to == ZERO_ADDRESS.as_bytes().to_vec() {
        return TOKEN_BURN.to_string();
    }

    TOKEN_TRANSFER.to_string()
}
