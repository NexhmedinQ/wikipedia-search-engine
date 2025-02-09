use std::{
    env,
    sync::mpsc::{self, Receiver, Sender},
};
pub mod crawler;
pub mod indexer;
pub mod parser;
fn main() {
    println!("Hello, world!");
    //crate::parser::Parser::new(String::from("example.txt"));
    let (tx, rx): (Sender<Vec<String>>, Receiver<Vec<String>>) = mpsc::channel();
    let crawler_handle = std::thread::spawn(move || {
        let key = env::var("API_KEY").expect("$API_KEY is not set");
        let mut crawler = crate::crawler::Crawler::new(vec![String::from(
            "https://api.wikimedia.org/core/v1/wikipedia/en/page/Web_crawler/html",
        )]);
        crawler.run(tx, &key);
    });

    let indexer_handle = std::thread::spawn(move || {
        let mut indexer = crate::indexer::Indexer::new();
        indexer.run(rx);
    });

    crawler_handle.join();
    indexer_handle.join();
}
