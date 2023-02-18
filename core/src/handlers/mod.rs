pub mod body_parser;
pub mod common;
pub mod err;
pub mod form;
pub mod response;
pub mod user;

use async_trait::async_trait;
use axum::{
    body::{self, Body, BoxBody, Bytes, Full},
    extract::FromRequest,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Form, Json,
};
use serde::{Deserialize, Serialize};
