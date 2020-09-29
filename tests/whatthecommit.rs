use anyhow::*;
use async_std::task;
use futures::prelude::*;
use std::time::Duration;
use propaganda::db::ProvideArticles;
use sqlx::prelude::*;

const url: &str = "http://whatthecommit.com/";

#[async_std::test]
async fn fun() -> Result<()> {
    let mut db: sqlx::SqliteConnection = sqlx::SqliteConnection::connect("sqlite::").await?;
    db.ensure_created_tables().await?;

    fetch_whatthecommit(&mut db).await?;
    fetch_whatthecommit(&mut db).await?;
    fetch_whatthecommit(&mut db).await?;
    fetch_whatthecommit(&mut db).await?;

    let article = db.get_article(url).await?;
    let metadatas = db.get_snaphot_metadatas_from_article(article.article_id).await?;
    
    println!("{:?}", metadatas);

    for metadata in metadatas {
        let snapshot = db.get_snaphot(metadata.snapshot_id).await?;
        println!("{}", get_article_fulltext(&snapshot.html)?);
    }

    Ok(())
}

async fn fetch_whatthecommit<T>(conn: &mut T) -> Result<()> where T: Send + propaganda::db::ProvideArticles {
    let article = conn.insert_article(url).await?;
    insert_snapshot(conn, &article).await?;
    Ok(())
}

async fn insert_snapshot<T>(provider: &mut T, article: &propaganda::db::Article) -> Result<()> where T: Send + propaganda::db::ProvideArticles {
    let html = surf_get_string(&article.url).await?;
    provider.insert_snapshot(article, timestamp(), &html).await?;
    Ok(())
}

fn timestamp() -> i32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32
}

async fn surf_get_string(uri: impl AsRef<str>) -> Result<String> {
    println!("surf_get_string {}", uri.as_ref());

    surf::get(uri)
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))
}

fn get_article_fulltext(html: &str) -> Result<String> {
    let fragment = scraper::Html::parse_fragment(&html);
    let mut fulltext = String::new();

    for selector in &[ "div.storywrapper", "div#content > p:first-child" ] {
        let selector = scraper::Selector::parse(selector).unwrap();
        let elements: Vec<scraper::ElementRef> = fragment.select(&selector).collect();

        if !elements.is_empty() {
            for element in fragment.select(&selector) {
                for text in element.text() {
                    let text = text.trim();
                    if !text.is_empty() {
                        fulltext.push_str(text);
                        fulltext.push_str("\n");
                    }
                }
            }

            break;
        }
    }
    
    Ok(fulltext)
}
