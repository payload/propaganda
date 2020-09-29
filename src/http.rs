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

#[derive(Serialize)]
struct Article2 {
    headline: String,
    hasChanges: bool,
    siteName: String,
    changesNumber: i32,
    timespan: i32,
    fetchtime: i32,
    compareFirstLastUrl: String,
    recentChangesUrl: String,
    lastSnapshotUrl: String,
    firstSnapshotUrl: String,
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

    let article_id = if let Ok(query) = req.query::<UrlQuery>() {
        provider.get_article(&query.url).await.map(|a| a.article_id)
    } else {
        Ok(req.query::<IdQuery>().map(|q| q.id)?)
    }?;

    let metadata = provider
        .get_snaphot_metadatas_from_article(article_id)
        .await?;
    Ok(Response::builder(200)
        .body(serde_json::to_string(&metadata)?)
        .content_type(mime::json())
        .build())
}

pub async fn get_snaphot(req: Request<SqlitePool>) -> Result<Response> {
    let mut provider = req.state().acquire().await?;
    let query: IdQuery = req.query()?;
    let mut snapshot = provider.get_snaphot(query.id).await?;

    fn get_article_fulltext(html: &str) -> String {
        let fragment = scraper::Html::parse_fragment(&html);
        let mut fulltext = String::new();
        
        if let Ok(selector) = scraper::Selector::parse("div.storywrapper") {
            for element in fragment.select(&selector) {
                for text in element.text() {
                    let text = text.trim();
                    if !text.is_empty() {
                        fulltext.push_str(text);
                        fulltext.push_str("\n");
                    }
                }
            }
        } else if let Ok(selector) = scraper::Selector::parse("div#content > p") {
            for element in fragment.select(&selector) {
                for text in element.text() {
                    let text = text.trim();
                    if !text.is_empty() {
                        fulltext.push_str(text);
                        fulltext.push_str("x\n");
                    }
                }
            }
        } else {
            fulltext.push_str(html);
        }
        fulltext
    }

    let fulltext = get_article_fulltext(&snapshot.html);
    snapshot.html = fulltext;

    Ok(Response::builder(200)
        .body(serde_json::to_string(&snapshot).expect("serde_json to_string snapshot"))
        .content_type(mime::json())
        .build())
}
