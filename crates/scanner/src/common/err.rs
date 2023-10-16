use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Upsert Error: source {src}, err {err}")]
    Upsert {
        src: String,
        #[source]
        err: DbErr,
    },
    #[error("Update Error: source {src}, err {err}")]
    Update {
        src: String,
        #[source]
        err: DbErr,
    },
    #[error("Create Error: source {src}, err {err}")]
    Create {
        src: String,
        #[source]
        err: DbErr,
    },
    #[error("Query Error: {0}")]
    Query(#[source] DbErr),

    #[error("NewDecimal Error: source {src}, err {err}")]
    NewDecimal { src: String, err: String },

    #[error("NewBigDecimal Error: source {0}")]
    NewBigDecimal(String),
}
