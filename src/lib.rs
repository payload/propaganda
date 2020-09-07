use anyhow::*;
use async_std::prelude::*;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::prelude::*;
use tide::log;

pub mod db;
pub mod http;
pub mod mime;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleVersion {
    pub version: u64,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleSource {
    pub url: String,
    pub articles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub url: String,
    pub versions: Vec<String>,
}

/*pub async fn fetch_tagesschau_article() -> Result<ArticleVersion> {
    let url = r"https://www.tagesschau.de/ausland/demonstrationen-belarus-107.html";
    let bytes = surf_get_bytes(url).await?;
    let html = String::from_utf8_lossy(&bytes);

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

    let article = ArticleVersion {
        version: std::time::SystemTime::now().elapsed()?.as_secs(),
        content: fulltext,
    };

    Ok(article)
}*/

async fn surf_get_bytes(uri: impl AsRef<str>) -> Result<Vec<u8>> {
    surf::get(uri)
        .recv_bytes()
        .await
        .map_err(|err| anyhow!(err))
}

pub async fn db_to_json(db: &sled::Db) -> Result<serde_json::Value> {
    use serde_json::Map;

    let mut json_db = Map::new();
    for tree_name in db.tree_names() {
        let mut json_tree = Map::new();
        for entry in db.open_tree(&tree_name)?.into_iter() {
            if let Ok((key, value)) = entry {
                let key = string_from_ivec(&key);
                let value = string_from_ivec(&value);
                json_tree.insert(key, value.into());
            }
        }
        json_db.insert(string_from_ivec(&tree_name), json_tree.into());
    }
    Ok(json_db.into())
}

pub fn string_from_ivec(ivec: &sled::IVec) -> String {
    String::from_utf8_lossy(&ivec.to_vec()).to_string()
}

pub async fn fetch_sources(tree: sled::Tree) -> Result<()> {
    for entry in tree.iter() {
        if let Ok((_, value)) = entry {
            if let Ok(article_source) = bincode::deserialize::<ArticleSource>(&value.to_vec()) {
                log::info!("{}", article_source.url);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct App {
    pub db: sled::Db,

    articles: sled::Tree,
    article_sources: sled::Tree,
    article_versions: sled::Tree,
}

impl App {
    pub fn from_db(db: sled::Db) -> Result<Self> {
        Ok(App {
            articles: db.open_tree("articles")?,
            article_sources: db.open_tree("article_sources")?,
            article_versions: db.open_tree("article_versions")?,
            db,
        })
    }

    pub fn insert_default_source_if_empty(&self) -> Result<()> {
        if self.article_sources.is_empty() {
            self.insert_article_source(&ArticleSource {
                url: "https://www.tagesschau.de/archiv/meldungsarchiv100~_date-20200830.html"
                    .into(),
                articles: vec![],
            })?;
        }
        Ok(())
    }

    pub async fn fetch_source(&self) -> Result<()> {
        if let Some(src) =
            iter_sled::<ArticleSource>(&self.article_sources).nth(self.article_sources.len())
        {
            log::info!("fetch_source url={}", src.url);

            let bytes = surf_get_bytes(src.url).await?;
            let html = String::from_utf8_lossy(&bytes);

            let _fragment = scraper::Html::parse_fragment(&html);
            let _selector = scraper::Selector::parse("div.storywrapper").unwrap();
        }
        Ok(())
    }
}

impl App {
    fn insert_article_source(&self, src: &ArticleSource) -> Result<()> {
        self.article_sources
            .insert(&src.url, bincode::serialize(&src)?)?;
        Ok(())
    }
}

fn iter_sled<Value: DeserializeOwned>(tree: &sled::Tree) -> impl Iterator<Item = Value> {
    tree.iter()
        .filter_map(|r| r.ok())
        .map(|(_, value)| deserialize::<Value>(&value))
        .filter_map(|r| r.ok())
}

fn deserialize<Value: DeserializeOwned>(ivec: &sled::IVec) -> Result<Value> {
    Ok(bincode::deserialize::<Value>(ivec.as_ref())?)
}

