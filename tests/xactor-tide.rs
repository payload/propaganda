use xactor::*;

#[message(result = "String")]
struct ToUppercase(String);

struct MyActor;

impl Actor for MyActor {}

#[async_trait::async_trait]
impl Handler<ToUppercase> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: ToUppercase) -> String {
        msg.0.to_uppercase()
    }
}

#[async_std::test]
async fn fun() -> Result<()> {
    let mut server = tide::Server::new();
    server
        .at("")
        .get(|_| async { Ok(tide::Response::builder(200).body("works").build()) });

    let addr = MyActor.start().await?;
    xactor::spawn(server.clone().listen("localhost:8000"));

    let res = addr.call(ToUppercase("lowercase".to_string())).await?;
    assert_eq!(res, "LOWERCASE");

    use tide::http::*;
    let req = Request::new(Method::Get, Url::parse("https://localhost:8080")?);
    let mut res: tide::http::Response = server.respond(req).await.unwrap();
    let body = res.body_string().await.unwrap();
    assert_eq!(body, "works");

    Ok(())
}
