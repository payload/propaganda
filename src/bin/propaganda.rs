use anyhow::*;
use async_std::prelude::*;
use async_trait::async_trait;
use propaganda::*;
use sqlx::prelude::*;
use tide::log;

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::with_level(tide::log::LevelFilter::Info);

    let app = App::from_db(sled::open("sled.db")?)?;
    app.insert_default_source_if_empty();

    let mut server = tide::with_state(app);

    server.at("/").get(|_| async {
        Ok(tide::Response::builder(200)
            .content_type(mime::html())
            .body(
                vec![
                    anchor("/db.json"),
                    anchor("/clear_db"),
                    anchor("/fetch_source"),
                ]
                .join("<br />"),
            )
            .build())
    });

    server
        .at("/db.json")
        .get(|req: Request| async move { Ok(db_to_json(&req.app().db).await?) });

    server.at("/clear_db").get(|req: Request| async {
        clear_db(&req.app().db).await?;
        Ok(req.redirect("/")?)
    });

    server
        .at("/fetch_source")
        .get(|req: Request| async { Ok(req.redirect("/")?) });

    server.listen("localhost:3000").await?;

    Ok(())
}

async fn run_tide() -> Result<()> {
    let db = sqlx::SqlitePool::new("file:propaganda.db").await?;

    db.acquire().await?;

    Ok(())
}


fn anchor(href: &str) -> String {
    format!("<a href={}>{}</a>", href, href)
}

async fn clear_db(db: &sled::Db) -> Result<()> {
    log::info!("clear_db");
    for tree_name in db.tree_names() {
        let tree = db.open_tree(tree_name)?;
        tree.clear()?;
        tree.flush_async().await?;
    }
    Ok(())
}

type Request = tide::Request<propaganda::App>;

trait RequestExt {
    fn app(&self) -> &propaganda::App;
    fn redirect(self, location: &str) -> Result<tide::Response>;
}

impl RequestExt for Request {
    fn app(&self) -> &propaganda::App {
        self.state()
    }

    fn redirect(self, location: &str) -> Result<tide::Response> {
        Ok(tide::Redirect::new(location).into())
    }
}

/*
fn update_articles() {
    for source in article_sources {
        for article in fetch_articles(source) {
            if let Some(existing_article) = get_existing_article(article) {
                if interesting_diff(existing_article, article) {
                    add_article_snapshot(article);
                }
            } else {
                add_new_article(article);
            }
        }
    }
}
*/
