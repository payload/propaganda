#[should_panic(expected = "Bye")]
#[async_std::test]
async fn panic_test() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("{}", panic_info);
        std::process::exit(-1);
    }));

    async_std::task::spawn(async {
        unimplemented!("Bye");
    });
}