use std::{
    collections::{HashSet, VecDeque},
    sync::mpsc::Sender,
    time::Duration,
};

use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::parser::{self, Parser};
pub struct Crawler {
    visited_urls: HashSet<String>,
    url_queue: VecDeque<String>,
    http_client: Client,
}

impl Crawler {
    pub fn new(starting_urls: Vec<String>) -> Self {
        let mut url_queue = VecDeque::new();
        starting_urls
            .iter()
            .for_each(|value| url_queue.push_back(value.clone()));
        Crawler {
            visited_urls: HashSet::new(),
            url_queue,
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn run(&mut self, tx: Sender<Vec<String>>, api_key: &str) {
        loop {
            let request_url = self.url_queue.pop_front().unwrap();
            let request = self
                .http_client
                .get(&request_url)
                //.bearer_auth(api_key)
                .build()
                .unwrap();
            // TODO: add a handler so we don't crash if the request times out
            let res = Client::execute(&self.http_client, request).unwrap();
            if res.status().is_success() {
                let body = String::from_utf8(res.bytes().unwrap().to_vec()).unwrap();
                let content = Parser::parse(&body);
                let _ = tx.send(content.tokens);
            } else {
                self.url_queue.push_back(request_url);
            }
        }
    }
}
