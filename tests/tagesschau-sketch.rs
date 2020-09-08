use anyhow::*;
use evmap_derive::ShallowCopy;
use itertools;
use itertools::Itertools;
use prettydiff;

#[derive(Debug, Eq, PartialEq, Hash, ShallowCopy)]
struct ArticleSnapshot {
    html: String,
}

#[async_std::test]
async fn fun() -> Result<()> {
    let (article_snapshots_r, mut article_snapshots_w) = evmap::new::<String, ArticleSnapshot>();

    println!("// fetch a page and get article urls");

    let url = r"https://www.tagesschau.de/archiv/meldungsarchiv100~_date-20200830.html";

    let html = surf_get_string(url).await?;

    let fragment = scraper::Html::parse_fragment(&html);
    let selector = scraper::Selector::parse(".linklist a").unwrap();

    let mut article_urls = vec![];

    let mut url = surf::url::Url::parse(url)?;
    for element in fragment.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with("/") {
                url.set_path(href);
                // sample: "https://www.tagesschau.de/ausland/griechenland-tuerkei-109.html"
                // but also: "https://www.tagesschau.de/archiv/meldungsarchiv100~_date-202006.html"

                article_urls.push(url.clone());
            }
        }
    }

    assert!(!article_urls.is_empty());

    println!("// fetch an article, store snapshot in map");

    if let Some(url) = article_urls.get(0) {
        let html = surf_get_string(url).await?;
        let modified_html = html.replace("Corona", "Morona");

        article_snapshots_w
            .insert(url.to_string(), ArticleSnapshot { html })
            .insert(
                url.to_string(),
                ArticleSnapshot {
                    html: modified_html,
                },
            )
            .refresh();
    }

    println!("// show articles");

    for (url, snapshots) in &article_snapshots_r.read().unwrap() {
        for snapshot in snapshots {
            let text = get_article_fulltext(&snapshot.html);
            dbg!(url, shorttext(&text));
        }
    }

    println!("// show article diffs");

    for (_url, snapshots) in &article_snapshots_r.read().unwrap() {
        // for (a, b) in itertools snapshots.iter().peekable() {
        //     prettydiff::diff_words(a, b);
        // }
        for (a, b) in snapshots.iter().tuples() {
            let a = shorttext(&get_article_fulltext(&a.html));
            let b = shorttext(&get_article_fulltext(&b.html));
            let diff = prettydiff::diff_words(&a, &b);
            println!("\n\n{}\n\n", diff);
        }
    }

    Ok(())
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
