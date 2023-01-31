mod prepare;

use entities::scanner_height;
use prepare::prepare_mock_db;
use scanner::model::height::{Mutation, Query};

#[tokio::test]
async fn main() {
    let db = &prepare_mock_db();

    {
        let post = Query::select_one(db, "eth:5").await.unwrap().unwrap();

        assert_eq!(post.id, 1);
    }

    {
        let post = Query::select_one(db, "heco:256").await.unwrap().unwrap();
        assert_eq!(post.id, 2)
    }

    {
        let post = Mutation::create_scanner_height(
            db,
            scanner_height::Model {
                id: 0,
                task_name: "eth:10".to_owned(),
                chain_name: "eth".to_owned(),
                height: 8899888,
                created_at: None,
                updated_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            post,
            scanner_height::ActiveModel {
                id: sea_orm::ActiveValue::Unchanged(3),
                task_name: sea_orm::ActiveValue::Unchanged("eth:10".to_owned()),
                chain_name: sea_orm::ActiveValue::Unchanged("eth".to_owned()),
                height: sea_orm::ActiveValue::Unchanged(8899888),
                created_at: sea_orm::ActiveValue::Unchanged(None),
                updated_at: sea_orm::ActiveValue::Unchanged(None)
            }
        );
    }

    {
        let post = Mutation::update_height_by_id(
            db,
            1,
            scanner_height::Model {
                id: 1,
                task_name: "eth:5".to_owned(),
                chain_name: "eth".to_owned(),
                height: 8899999,
                created_at: None,
                updated_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            post,
            scanner_height::Model {
                id: 1,
                task_name: "eth:5".to_owned(),
                chain_name: "eth".to_owned(),
                height: 8899999,
                created_at: None,
                updated_at: None
            }
        );
    }

    {
        let result = Mutation::delete_task(db, 1).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }

    {
        let result = Mutation::delete_all_task(db).await.unwrap();

        assert_eq!(result.rows_affected, 2);
    }
}
