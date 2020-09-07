use anyhow::*;
use propaganda::db::*;
use sqlx::prelude::*;
use async_std::{task, sync};

#[async_std::test]
async fn insert_update_and_get_outdated_articles() -> Result<()> {
    let mut db: sqlx::SqliteConnection = sqlx::SqliteConnection::connect("sqlite::").await?;

    db.ensure_created_tables().await?;
    db.insert_article("article1").await.expect("insert_article");
    db.insert_article("article2").await?;
    db.insert_article("article3").await?;
    db.update_article("article1", 42)
        .await
        .expect("update_article");

    let outdated = db
        .get_outdated_articles(2)
        .await
        .expect("get_outdated_articles");

    let outdated = outdated.into_iter().map(|a| a.url).collect::<Vec<_>>();
    assert_eq!(outdated, vec!["article2", "article3"]);

    Ok(())
}

#[async_std::test]
async fn insert_snapshots_and_get_snapshots() -> Result<()> {
    let mut db: sqlx::SqliteConnection = sqlx::SqliteConnection::connect("sqlite::").await?;

    db.ensure_created_tables().await?;
    let article1 = db.insert_article("article1").await.expect("insert_article");
    let article2 = db.insert_article("article2").await?;

    let archived_at = 5;
    let html1 = "Will it work? Questions asked in a test.";
    db.insert_snapshot(&article1, archived_at, html1)
        .await
        .expect("insert_snapshot");

    let archived_at = 12;
    let html2 = "Will it work? Exclusive interview in a test.";
    db.insert_snapshot(&article1, archived_at, html2).await?;

    let archived_at = 7;
    let html3 = "Think about a cat.";
    db.insert_snapshot(&article2, archived_at, html3).await?;

    let snapshots = db.get_snaphot_metadatas_from_article(&article1).await?;
    assert_eq!(snapshots.len(), 2);
    let snapshot1 = db.get_snaphot(snapshots[0].snapshot_id).await?;
    let snapshot2 = db.get_snaphot(snapshots[1].snapshot_id).await?;
    assert_eq!(snapshot1.html, html1);
    assert_eq!(snapshot2.html, html2);

    Ok(())
}

#[async_std::test]
async fn pool_access() -> Result<()> {
    let pool = sqlx::SqlitePool::new("sqlite::out-of-memory.db").await?;
    let mut db = &mut *pool.acquire().await?;

    let mut conn: sqlx::SqliteConnection = sqlx::SqliteConnection::connect("sqlite::").await?;
    let mut mu_db = sync::Arc::new(sync::Mutex::new(conn));
    
    {
        let mut db = &mut mu_db.lock().await;

        db.ensure_created_tables().await?;
        db.insert_article("article1").await.expect("insert_article");
        db.insert_article("article2").await?;
        db.insert_article("article3").await?;
        db.update_article("article1", 42)
            .await
            .expect("update_article");

    }

    // let mut db = &mut *pool.acquire().await?;
    let outdated = {
        let mut db = &mut mu_db.lock().await;

        let outdated = db
            .get_outdated_articles(2)
            .await
            .expect("get_outdated_articles");

        outdated.into_iter().map(|a| a.url).collect::<Vec<_>>()
    };

    assert_eq!(outdated, vec!["article2", "article3"]);

    Ok(())
}
