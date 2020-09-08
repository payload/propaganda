use anyhow::*;
use propaganda::db::ProvideArticles;
use propaganda::*;

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::with_level(tide::log::LevelFilter::Info);

    let db_path = "sqlite:propaganda.db";
    let pool = sqlx::SqlitePool::new(&db_path)
        .await
        .expect("SqlitePool::new");

    let mut conn = pool.acquire().await?;
    conn.ensure_created_tables().await?;

    let mut server = tide::with_state(pool);

    server.at("/get_articles").get(&http::get_articles);
    server
        .at("/get_snaphot_metadatas_from_article")
        .get(&http::get_snaphot_metadatas_from_article);
    server.at("/insert_article").get(&http::insert_article);
    server.at("/get_snapshot").get(&http::get_snaphot);

    server
        .at("/favicon.ico")
        .get(|_| async { Ok(tide::Response::builder(200).build()) });

    // in case there is no body, make it html and show the debug_index
    // also show the error string in case there is any
    //
    // with some error handling and string matching this could be
    // a generic interactive HTTP API user interface
    server.with(tide::utils::After(|mut res: tide::Response| async {
        if res.len().unwrap_or_default() == 0 {
            res.set_content_type(mime::html());

            res.set_body(if let Some(err) = res.error() {
                format!("<h4>{}</h4>{}", err.to_string(), debug_index())
            } else {
                format!("<h4>{}</h4>{}", res.status().to_string(), debug_index())
            });
        }
        Ok(res)
    }));

    server.listen("localhost:8080").await.expect("listen");

    Ok(())
}

fn debug_index() -> String {
    fn anchor(href: &str, desc: &str) -> String {
        format!("<a href={}>{}</a> <span>{}</span>", href, href, desc)
    }

    vec![
        anchor("get_articles", ""),
        anchor("get_snaphot_metadatas_from_article", "url"),
        anchor("insert_article", "url"),
        anchor("get_snapshot", "id"),
    ]
    .join("<br />")
}
