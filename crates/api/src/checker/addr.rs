use hex::FromHex;

use crate::err::{AppError, CoreError};

pub fn check_address(address: String) -> Result<Vec<u8>, AppError> {
    if address.len() != 66 || !(address.starts_with("0x") || address.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(address)));
    }

    Vec::from_hex(&address[2..address.len()]).map_err(AppError::from)
}
