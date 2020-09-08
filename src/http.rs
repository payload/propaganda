use crate::db::ProvideArticles;
use crate::mime;

use sqlx::SqlitePool;
use tide::{prelude::*, Request, Response, Result};

#[derive(Deserialize)]
struct UrlQuery {
    url: String,
}

#[derive(Deserialize)]
struct IdQuery {
    id: i32,
}

pub async fn insert_article(req: Request<SqlitePool>) -> Result<Response> {
    let mut provider = req.state().acquire().await.expect("conn");
    let query: UrlQuery = req.query()?;
    provider.insert_article(&query.url).await?;
    Ok(Response::new(200))
}

pub async fn get_articles(req: Request<SqlitePool>) -> Result<Response> {
    let mut provider = req.state().acquire().await?;
    let articles = provider.get_articles(0, 100).await?;
    Ok(Response::builder(200)
        .body(serde_json::to_string(&articles).expect("serde_json to_string articles"))
        .content_type(mime::json())
        .build())
}

pub async fn get_snaphot_metadatas_from_article(req: Request<SqlitePool>) -> Result<Response> {
    let mut provider = req.state().acquire().await?;
    let query: UrlQuery = req.query()?;
    let article = provider.get_article(&query.url).await?;
    let metadata = provider
        .get_snaphot_metadatas_from_article(&article)
        .await?;
    Ok(Response::builder(200)
        .body(serde_json::to_string(&metadata)?)
        .content_type(mime::json())
        .build())
}

pub async fn get_snaphot(req: Request<SqlitePool>) -> Result<Response> {
    let mut provider = req.state().acquire().await?;
    let query: IdQuery = req.query()?;
    let snapshot = provider.get_snaphot(query.id).await?;
    Ok(Response::builder(200)
        .body(serde_json::to_string(&snapshot).expect("serde_json to_string snapshot"))
        .content_type(mime::json())
        .build())
}
