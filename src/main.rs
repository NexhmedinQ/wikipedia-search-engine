use std::sync::mpsc::{self, Receiver, Sender};

use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};
pub mod crawler;
pub mod indexer;

fn main() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {h({l})} | {f}:{L} - {m}{n}",
        )))
        .build();
    let engine = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {h({l})} | {f}:{L} - {m}{n}",
        )))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("wikipedia-search-engine", Box::new(engine)))
        .logger(
            Logger::builder()
                .appender("wikipedia-search-engine")
                .additive(false)
                .build("wikipedia-search-engine", LevelFilter::Info),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();

    let (tx, rx): (Sender<(Vec<String>, u32)>, Receiver<(Vec<String>, u32)>) = mpsc::channel();
    let crawler_handle = std::thread::spawn(move || {
        let mut crawler = crate::crawler::Crawler::new(vec![String::from(
            "https://api.wikimedia.org/core/v1/wikipedia/en/page/Web_crawler/html",
        )]);
        crawler.run(tx);
    });

    let indexer_handle = std::thread::spawn(move || {
        let mut indexer = crate::indexer::Indexer::new();
        indexer.run(rx);
    });

    let _ = crawler_handle.join();
    let _ = indexer_handle.join();
}
