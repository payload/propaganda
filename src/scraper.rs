use crate::db::ProvideArticles;
use anyhow::anyhow;
use std::time::Duration;
use xactor::*;

#[message(result = "()")]
#[derive(Clone, Debug)]
struct DumpArticleUrls;

#[message(result = "()")]
#[derive(Clone, Debug)]
struct FetchTopArticle;

pub struct Scraper {
    pool: sqlx::SqlitePool,
}

impl Scraper {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Actor for Scraper {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.send_interval(FetchTopArticle, Duration::from_secs(10));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<DumpArticleUrls> for Scraper {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: DumpArticleUrls) -> () {
        let urls = self
            .pool
            .acquire()
            .await
            .unwrap()
            .get_articles(0, 100)
            .await
            .unwrap()
            .into_iter()
            .map(|a| a.url)
            .collect::<Vec<String>>()
            .join("\n");
        println!("{}", urls);
    }
}

#[async_trait::async_trait]
impl Handler<FetchTopArticle> for Scraper {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: FetchTopArticle) -> () {
        let mut conn = self.pool.acquire().await.unwrap();
        if let Some(outdated) = conn.get_outdated_articles(1).await.unwrap().get(0) {
            let html = surf_get_string(&outdated.url).await.unwrap();

            // TODO saving time as i32 is not good
            let archived_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let archived_at = archived_at as i32;
            if let Err(err) = conn.insert_snapshot(&outdated, archived_at, &html).await {
                println!("oh no: {}", err);
            }
        }
    }
}

async fn surf_get_string(uri: impl AsRef<str>) -> Result<String> {
    println!("surf_get_string {}", uri.as_ref());

    surf::get(uri)
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))
}

fn get_article_fulltext(html: &str) -> String {
    let fragment = scraper::Html::parse_fragment(&html);
    let selector = scraper::Selector::parse("div.storywrapper").unwrap();
    let mut fulltext = String::new();
    for element in fragment.select(&selector) {
        for text in element.text() {
            let text = text.trim();
            if !text.is_empty() {
                fulltext.push_str(text);
                fulltext.push_str("\n");
            }
        }
    }
    fulltext
}

fn shorttext(text: &str) -> String {
    text.chars().take(400).collect()
}
