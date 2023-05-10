use sea_orm::DatabaseConnection;

pub struct AppState {
    pub conn: DatabaseConnection,
}

pub fn get_conn(state: &AppState) -> &DatabaseConnection {
    &state.conn
}
