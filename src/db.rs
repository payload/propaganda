use anyhow::*;
use async_trait::async_trait;
use mockall::automock;
use sqlx::prelude::*;

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct Article {
    pub url: String,
    pub article_id: i32,
    pub updated_at: i32,
}

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct SnapshotMetadata {
    pub article_id: i32,
    pub snapshot_id: i32,
    pub archived_at: i32,
}

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct Snapshot {
    pub article_id: i32,
    pub snapshot_id: i32,
    pub archived_at: i32,
    pub html: String,
}

#[automock]
#[async_trait]
pub trait ProvideArticles {
    async fn ensure_created_tables(&mut self) -> Result<()>;
    async fn get_outdated_articles(&mut self, limit: i32) -> Result<Vec<Article>>;
    async fn get_articles(&mut self, offset: i32, limit: i32) -> Result<Vec<Article>>;
    async fn insert_article(&mut self, url: &str) -> Result<Article>;
    async fn update_article(&mut self, url: &str, updated_at: i32) -> Result<()>;
    async fn get_article(&mut self, url: &str) -> Result<Article>;

    async fn get_snaphot_metadatas_from_article(
        &mut self,
        article: &Article,
    ) -> Result<Vec<SnapshotMetadata>>;
    async fn get_youngest_snaphot(&mut self, article: &Article) -> Result<Option<Snapshot>>;
    async fn get_snaphot(&mut self, id: i32) -> Result<Snapshot>;
    async fn insert_snapshot(
        &mut self,
        article: &Article,
        archived_at: i32,
        html: &str,
    ) -> Result<()>;
}

#[async_trait]
impl ProvideArticles for sqlx::SqliteConnection {
    async fn ensure_created_tables(&mut self) -> Result<()> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS articles (
                article_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                url TEXT UNIQUE NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS snapshots (
                article_id INTEGER NOT NULL,
                snapshot_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                archived_at INTEGER NOT NULL,
                html STRING NOT NULL
            );
            ",
        )
        .execute(self)
        .await
        .void()
    }

    async fn get_outdated_articles(&mut self, limit: i32) -> Result<Vec<Article>> {
        sqlx::query_as::<_, Article>(
            r"
            SELECT url, article_id, updated_at
            FROM articles
            ORDER BY updated_at ASC
            LIMIT $1",
        )
        .bind(limit)
        .fetch_all(self)
        .await
        .anyhow()
    }

    async fn get_articles(&mut self, offset: i32, limit: i32) -> Result<Vec<Article>> {
        // TODO order by created_at
        sqlx::query_as::<_, Article>(
            r"
            SELECT url, article_id, updated_at
            FROM articles
            ORDER BY updated_at ASC
            LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self)
        .await
        .anyhow()
    }

    async fn insert_article(&mut self, url: &str) -> Result<Article> {
        sqlx::query_as(
            r"
            INSERT OR IGNORE INTO articles ( url, updated_at )
            VALUES ( $1, $2 );
            SELECT * FROM articles WHERE url = $3 ;",
        )
        .bind(url)
        .bind(0)
        .bind(url)
        .fetch_one(self)
        .await
        .anyhow()
    }

    async fn update_article(&mut self, url: &str, updated_at: i32) -> Result<()> {
        sqlx::query(
            r"
            UPDATE articles SET updated_at=$1 WHERE url=$2",
        )
        .bind(updated_at)
        .bind(url)
        .execute(self)
        .await
        .void()
    }

    async fn get_article(&mut self, url: &str) -> Result<Article> {
        sqlx::query_as(
            r"
            SELECT * FROM articles WHERE url = $1 LIMIT 1",
        )
        .bind(url)
        .fetch_one(self)
        .await
        .anyhow()
    }

    async fn get_snaphot_metadatas_from_article(
        &mut self,
        article: &Article,
    ) -> Result<Vec<SnapshotMetadata>> {
        sqlx::query_as(
            r"
            SELECT * FROM snapshots WHERE article_id = $1",
        )
        .bind(article.article_id)
        .fetch_all(self)
        .await
        .anyhow()
    }

    async fn get_youngest_snaphot(&mut self, article: &Article) -> Result<Option<Snapshot>> {
        sqlx::query_as(
            r"
            SELECT * FROM snapshots WHERE article_id = $1 ORDER BY archived_at DESC LIMIT 1",
        )
        .bind(article.article_id)
        .fetch_optional(self)
        .await
        .anyhow()
    }

    async fn get_snaphot(&mut self, id: i32) -> Result<Snapshot> {
        sqlx::query_as(
            r"
            SELECT * FROM snapshots WHERE snapshot_id = $1 LIMIT 1",
        )
        .bind(id)
        .fetch_one(self)
        .await
        .anyhow()
    }

    async fn insert_snapshot(
        &mut self,
        article: &Article,
        archived_at: i32,
        html: &str,
    ) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO snapshots (article_id, archived_at, html)
            VALUES ( $1, $2, $3 )",
        )
        .bind(article.article_id)
        .bind(archived_at)
        .bind(html)
        .execute(self)
        .await
        .void()
    }
}

trait VoidResult<T> {
    fn void(self) -> Result<()>;
    fn anyhow(self) -> Result<T>;
}

impl<T, E> VoidResult<T> for std::result::Result<T, E>
where
    E: Send + Sync + std::error::Error + 'static,
{
    fn void(self) -> Result<()> {
        self?;
        Ok(())
    }

    fn anyhow(self) -> Result<T> {
        self.map_err(&Error::new)
    }
}
