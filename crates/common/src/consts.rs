pub const ERC20: &str = "ERC-20";
pub const ERC1155: &str = "ERC-1155";
pub const ERC721: &str = "ERC-721";
pub const WETH: &str = "WTH";
pub const UNKNOWN: &str = "Unknown";

pub enum TokenKind {
    ERC20,
    ERC721,
    ERC1155,
    None,
}

// safeCreate2(bytsafes32 salt, bytes initializationCode)
pub const SAFE_CREATE2_METHOD_ID: &str = "0x64e03087";
pub const TRANSFER_METHOD_ID: &str = "0xa9059cbb";

pub const TOKEN_TRANSFER_SIGNATURE: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
pub const WETH_DEPOSIT_SIGNATURE: &str =
    "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c";
pub const WETH_WITHDRAWAL_SIGNATURE: &str =
    "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";
pub const ERC1155_SINGLE_TRANSFER_SIGNATURE: &str =
    "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62";
pub const ERC1155_BATCH_TRANSFER_SIGNATURE: &str =
    "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb";
pub const BRIDGE_HASH: &str = "0x3c798bbcf33115b42c728b8504cff11dd58736e9fa789f1cda2738db7d696b2a";

pub const BURN_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
