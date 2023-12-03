pub mod address;
pub mod block;
pub mod event;
pub mod helth;
pub mod response;
pub mod state;
pub mod token;
pub mod token_transfer;
pub mod transaction;
pub mod user;

use axum::{
    body::{Body, Bytes},
    extract::Path,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension, Json,
};
use chrono::NaiveDateTime;
use err::{AppError, CoreError};
use hex::FromHex;
use response::BaseResponse;
use serde::{Deserialize, Serialize};
use state::{get_conn, AppState};
use std::{collections::HashMap, sync::Arc};
use validator::Validate;

use crate::err;
