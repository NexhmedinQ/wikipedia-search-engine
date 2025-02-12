use std::{
    collections::{HashSet, VecDeque},
    sync::mpsc::Sender, thread::sleep, time::Duration,
};

use log::info;
use regex::Regex;
use reqwest::blocking::{Client, Response};
use scraper::Html;

pub struct Content {
    pub tokens: Vec<String>,
    pub links: Vec<String>,
}

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

    pub fn run(&mut self, tx: Sender<Vec<String>>) {
        loop {
            let request_url = self.url_queue.pop_front().unwrap();
            let request = self
                .http_client
                .get(&request_url)
               // .bearer_auth(api_key)
                .build()
                .unwrap();
            if let Ok(res) = Client::execute(&self.http_client, request) {
                if res.status().is_success() {
                    info!("Request to {} succeeded", request_url);
                    let body = String::from_utf8(res.bytes().unwrap().to_vec()).unwrap();
                    let content = Self::parse(&body);
                    content
                        .links
                        .into_iter()
                        .map(|link| {
                            format!(
                                "https://api.wikimedia.org/core/v1/wikipedia/en/page/{}/html",
                                link
                            )
                        })
                        .filter(|url| !self.visited_urls.contains(url))
                        .for_each(|unvisited_url| self.url_queue.push_back(unvisited_url));
                    let _ = tx.send(content.tokens);
                } else if !res.status().is_client_error() {
                    info!("Server error for request {}", request_url);
                    self.url_queue.push_back(request_url);
                } else {
                    info!("Client error for request {}", request_url);
                }
            } else {
                info!("Request to {} timed out", request_url);
                self.url_queue.push_back(request_url);
            };
            
            sleep(Duration::from_secs(5));
        }
    }

    fn parse(body: &str) -> Content {
        let document = Html::parse_document(body);
        let tree = document.root_element().tree();
        let mut res = String::new();
        let mut stack = vec![tree.root()];
        let mut visited = HashSet::new();
        let link_regex = Regex::new(r"^\.\/[A-Za-z0-9_\.\\-~()]+$").unwrap();
        let mut links = HashSet::new();
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            if visited.insert(node.id()) {
                if let Some(sibling) = node.next_sibling() {
                    if !visited.contains(&sibling.id()) {
                        stack.push(sibling);
                    }
                }
                match node.value() {
                    scraper::Node::Text(text) => {
                        res.push_str(&text);
                    }
                    scraper::Node::Element(element) => {
                        if matches!(element.name(), "sub" | "sup" | "style" | "math") {
                            continue;
                        }
                        if element.name() == "a" {
                            if let Some(link) = element.attr("href") {
                                if link_regex.is_match(link) {
                                    links.insert(link[2..link.len()].to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }

                for child in node.children().rev() {
                    if !visited.contains(&child.id()) {
                        stack.push(child);
                    }
                }
            }
        }
        Content {
            tokens: Self::extract_tokens(&res),
            links: links.into_iter().collect(),
        }
    }

    fn extract_tokens(data: &str) -> Vec<String> {
        let lines = data.lines();
        return lines
            .into_iter()
            .flat_map(|line| line.split_ascii_whitespace())
            .map(|word| Self::clean_word(word))
            .collect();
    }

    fn clean_word(word: &str) -> String {
        word.trim_start_matches(|letter: char| !letter.is_alphanumeric())
            .trim_end_matches(|letter: char| !letter.is_alphanumeric())
            .to_string()
    }
}
