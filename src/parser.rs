use std::{
    collections::HashSet,
    fs::File,
    io::Read
};

use regex::Regex;
use scraper::Html;


pub struct Content {
    pub tokens: Vec<String>,
    pub links: Vec<String>
}
pub struct Parser {}

impl Parser {
    pub fn parse(body: &str) -> Content {
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
        Content { tokens: Self::extract_tokens(&res), links: links.into_iter().collect() }
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
