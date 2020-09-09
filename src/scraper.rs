use std::time::Duration;
use xactor::*;

#[message(result = "()")]
#[derive(Clone, Debug)]
struct DumpArticleUrls;

pub struct Scraper;

#[async_trait::async_trait]
impl Actor for Scraper {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.send_interval(DumpArticleUrls, Duration::from_secs(1));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<DumpArticleUrls> for Scraper {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: DumpArticleUrls) -> () {
        println!("hello");
    }
}
