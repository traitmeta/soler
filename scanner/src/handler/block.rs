use entities::scanner_height::Model;
use sea_orm::DbConn;

use crate::repo::height::{Mutation, Query};

pub async fn current_height(conn: &DbConn, task_name: &str, chain_name: &str) -> Option<u64> {
    let current_model = Query::select_one_by_task_name(conn, task_name)
        .await
        .unwrap();
    match current_model {
        Some(current) => return Some(current.height),
        None => {
            tracing::debug!("not found {}", task_name);
            let insert_data = Model {
                id: 0,
                task_name: task_name.to_owned(),
                chain_name: chain_name.to_owned(),
                height: 1,
                created_at: None,
                updated_at: None,
            };
            let result = Mutation::create_scanner_height(conn, insert_data)
                .await
                .unwrap_or_else(|_| panic!("insert {} to scanner height table err", task_name));
            tracing::debug!("insert {} return :{:?}", task_name, result);
        }
    }
    None
}
