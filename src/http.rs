use crate::db::ProvideArticles;
use crate::mime;

use anyhow::*;
use tide::{Request, Response};

pub async fn insert_article(req: Request<sqlx::SqlitePool>) -> Result<Response> {
    let mut provider = req.state().acquire().await.expect("a");
    let url: String = req.param("url")?;
    provider.insert_article(&url).await?;
    Ok(Response::new(200))
}

pub async fn get_articles(req: Request<sqlx::SqlitePool>) -> tide::Result<Response> {
    let mut provider = req.state().acquire().await.expect("a");
    let articles = provider.get_articles(0, 100).await.expect("b");
    Ok(Response::builder(200)
        .body(serde_json::to_string(&articles).expect("c"))
        .content_type(mime::json())
        .build())
}

pub async fn get_snaphot_metadatas_from_article(
    req: Request<sqlx::SqlitePool>,
) -> Result<Response> {
    let mut provider = req.state().acquire().await.expect("a");
    let url: String = req.param("url")?;
    let article = provider.get_article(&url).await?;
    let metadata = provider
        .get_snaphot_metadatas_from_article(&article)
        .await?;
    Ok(Response::builder(200)
        .body(serde_json::to_string(&metadata)?)
        .content_type(mime::json())
        .build())
}
