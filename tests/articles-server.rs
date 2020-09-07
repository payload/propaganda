use anyhow::*;
use async_std::prelude::*;
use async_std::task;
use propaganda::db::ProvideArticles;
use propaganda::*;
use tide::{prelude::*, Request, Response};

type MyRequest = tide::Request<sqlx::SqlitePool>;

#[async_std::test]
async fn fun() -> Result<()> {
    let mut db_path = std::path::PathBuf::new();
    db_path.push(std::env::temp_dir());
    db_path.push("test.db");

    let _ = async_std::fs::remove_file(&db_path).await;
    
    let db_path = "file:".to_owned() + db_path.to_str().expect("db_path");
    let pool = sqlx::SqlitePool::new(&db_path).await.expect("SqlitePool::new");
    let mut conn = pool.acquire().await.expect("acquire");

    conn.ensure_created_tables().await.expect("ensure_created_tables");
    conn.insert_article("article1").await?;
    conn.insert_article("article2").await?;

    let mut server = tide::Server::with_state(pool);

    server.at("/get-articles").get(&http::get_articles);

    let join_server = task::spawn(server.listen("localhost:3000"));
    
    let mut response = surf::get("http://localhost:3000/get-articles").await.expect("surf::get");
    let response_str = &response.body_string().await.expect("body_string");

    join_server.cancel().await;

    assert!(response_str.contains("article1"));
    assert!(response_str.contains("article2"));

    Ok(())
}
