pub mod jwt;

use async_trait::async_trait;
use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization,HeaderMap},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
