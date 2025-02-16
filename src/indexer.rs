use std::{
    collections::{HashMap, LinkedList},
    sync::mpsc::Receiver,
};

use log::info;

#[derive(Clone, Debug)]
struct Posting {
    doc_id: u32,
    freq: u32,
}

pub struct Indexer {
    dictionary: HashMap<String, (u32, Box<LinkedList<Vec<Posting>>>)>,
}

impl Indexer {
    pub fn new() -> Self {
        Indexer {
            dictionary: HashMap::new(),
        }
    }

    pub fn run(&mut self, rx: Receiver<(Vec<String>, u32)>) {
        loop {
            let get_document = rx.try_recv();
            if let Ok(document) = get_document {
                let (tokens, doc_id) = document;
                let term_freq = Self::calculate_term_frequencies(tokens);
                self.process_document(term_freq, doc_id);
                info!("Current dict {:?}", self.dictionary);
            }
        }
    }

    fn calculate_term_frequencies(tokens: Vec<String>) -> HashMap<String, u32> {
        tokens.into_iter().fold(HashMap::new(), |mut map, term| {
            map.entry(term).and_modify(|freq| *freq += 1).or_insert(1);
            map
        })
    }

    fn process_document(&mut self, term_freq: HashMap<String, u32>, doc_id: u32) {
        for (term, freq) in term_freq {
            self.update_dictionary(term, doc_id, freq);
        }
    }

    /// Extracted: Update the dictionary with a new term frequency
    fn update_dictionary(&mut self, term: String, doc_id: u32, freq: u32) {
        if let Some(posting) = self.dictionary.get(&term) {
            let doc_freq = posting.0 + 1;
            let mut posting_list = *posting.1.clone();
            if let Some(last_node) = posting_list.back_mut() {
                if last_node.capacity() == last_node.len() {
                    let old_cap = last_node.capacity();
                    let mut new_posting = Vec::<Posting>::with_capacity(old_cap * 2);
                    new_posting.push(Posting { doc_id, freq });
                    posting_list.push_back(new_posting);
                    self.dictionary
                        .insert(term, (doc_freq, Box::new(posting_list)));
                } else {
                    last_node.push(Posting { doc_id, freq });
                    self.dictionary
                        .insert(term, (doc_freq, Box::new(posting_list)));
                }
            } else {
                panic!("Should not be possible!!!!!")
            }
        } else {
            let mut posting_list = LinkedList::new();
            let mut cur_posting = Vec::<Posting>::with_capacity(4);
            cur_posting.push(Posting { doc_id, freq });
            posting_list.push_back(cur_posting);
            self.dictionary.insert(term, (1, Box::new(posting_list)));
        }
    }
}

mod tests {
    use crate::indexer::Indexer;

    #[test]
    fn test_calculate_term_frequencies() {
        let tokens = vec!["apple".into(), "banana".into(), "apple".into()];
        let result = Indexer::calculate_term_frequencies(tokens);
        assert_eq!(result["apple"], 2);
        assert_eq!(result["banana"], 1);
    }
}
