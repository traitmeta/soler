pub mod block;
pub mod body_parser;
pub mod err;
pub mod event;
pub mod helth;
pub mod response;
pub mod state;
pub mod token;
pub mod token_transfer;
pub mod transaction;
pub mod user;

use async_trait::async_trait;
use axum::{
    body::{self, Body, BoxBody, Bytes, Full},
    extract::FromRequest,
    extract::Path,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension, Json,
};
// use chrono::NaiveDateTime;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use err::{AppError, CoreError};
use response::BaseResponse;
use state::{get_conn, AppState};
