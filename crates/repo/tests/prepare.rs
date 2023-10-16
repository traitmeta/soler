#[cfg(feature = "mock")]
pub fn prepare_mock_db() -> DatabaseConnection {
    let find_by_task_name_eth_5 = vec![log_receiver_chain::Model {
        id: 1,
        task_name: "eth:5".to_owned(),
        chain_name: "eth".to_owned(),
        height: 8899888,
        created_at: None,
        updated_at: None,
    }];

    let find_by_task_name_heco_256 = vec![log_receiver_chain::Model {
        id: 2,
        task_name: "heco:256".to_owned(),
        chain_name: "heco".to_owned(),
        height: 8899888,
        created_at: None,
        updated_at: None,
    }];

    let create_task_name_eth_10 = vec![log_receiver_chain::Model {
        id: 3,
        task_name: "eth:10".to_owned(),
        chain_name: "eth".to_owned(),
        height: 8899888,
        created_at: None,
        updated_at: None,
    }];

    let update_task_name_eth_5 = vec![log_receiver_chain::Model {
        id: 1,
        task_name: "eth:5".to_owned(),
        chain_name: "eth".to_owned(),
        height: 8899999,
        created_at: None,
        updated_at: None,
    }];
    MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![
            find_by_task_name_eth_5.clone(),
            find_by_task_name_heco_256,
            create_task_name_eth_10,
            find_by_task_name_eth_5.clone(),
            update_task_name_eth_5,
            find_by_task_name_eth_5,
        ])
        // 每一行对应着一个SQL语句，有点难搞哦
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 3,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 3,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 3,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 3,
                rows_affected: 2,
            },
        ])
        .into_connection()
}
