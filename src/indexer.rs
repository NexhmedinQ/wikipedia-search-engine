use std::{
    collections::{HashMap, LinkedList}, fs::{File, OpenOptions}, sync::mpsc::Receiver
};

use log::info;
use memmap2::{MmapMut, MmapOptions};

#[derive(Clone, Debug)]
struct Posting {
    doc_id: u32,
    freq: u32,
}

struct Term {
    file_offset: usize,
    term: String,
    doc_freq: u32
}

pub struct Indexer {
    in_mem_dictionary: HashMap<String, (u32, Box<LinkedList<Vec<Posting>>>)>,
    disk_posting: Option<MmapMut>,
    in_memory_bytes: u64,
    disk_term_dictionary: Vec<Term>
}

impl Indexer {
    pub fn new() -> Self {
        Indexer {
            in_mem_dictionary: HashMap::new(),
            disk_posting: Option::None,
            in_memory_bytes: 0,
            disk_term_dictionary: Vec::new(),
        }
    }

    pub fn run(&mut self, rx: Receiver<(Vec<String>, u32)>) {
        loop {
            let get_document = rx.try_recv();
            if let Ok(document) = get_document {
                let (tokens, doc_id) = document;
                let term_freq = Self::calculate_term_frequencies(tokens);
                self.process_document(term_freq, doc_id);
                // TODO: change so value is not hardcoded (also needs to increase value significantly)
                if self.in_memory_bytes > 1000 {
                    info!("moving postings to on disk");
                    Self::merge_indices(self);
                }
                info!("Current dict {:?}", self.in_mem_dictionary);
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
        self.in_memory_bytes += term_freq.len() as u64 * 8;  // will clean this up later. For now it is counting size of posting lists (not including pointers and such only data)
        for (term, freq) in term_freq {
            self.update_dictionary(term, doc_id, freq);
        }
    }

    /// Extracted: Update the dictionary with a new term frequency
    fn update_dictionary(&mut self, term: String, doc_id: u32, freq: u32) {
        if let Some(posting) = self.in_mem_dictionary.get(&term) {
            let doc_freq = posting.0 + 1;
            let mut posting_list = *posting.1.clone();
            if let Some(last_node) = posting_list.back_mut() {
                if last_node.capacity() == last_node.len() {
                    let old_cap: usize = last_node.capacity();
                    let mut new_posting = Vec::<Posting>::with_capacity(old_cap * 2);
                    new_posting.push(Posting { doc_id, freq });
                    posting_list.push_back(new_posting);
                    self.in_mem_dictionary
                        .insert(term, (doc_freq, Box::new(posting_list)));
                } else {
                    last_node.push(Posting { doc_id, freq });
                    self.in_mem_dictionary
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
            self.in_mem_dictionary.insert(term, (1, Box::new(posting_list)));
        }
    }

    fn merge_indices(&mut self) {
        // merge on disk and in memory representations
        // TODO : move this code into a new file under an InFileInvertedIndex struct
        if self.disk_posting.is_none() {
            let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("data.mmap").unwrap();
            file.set_len(self.in_memory_bytes).unwrap(); // Allocate 1KB space

            self.disk_posting = Some(unsafe { MmapOptions::new().map_mut(&file).unwrap() });

            let mut sorted_keys: Vec<String> = self.in_mem_dictionary.iter()
            .map(|entry| entry.0.clone())
            .collect();
            // values will all be distinct so no need to have a sorting algo that preserves ordering of equal elements when sorting
            sorted_keys.sort_unstable();
            let mut file_offset: usize = 0;
            for key in sorted_keys.iter() {
                self.disk_term_dictionary.push(Term { file_offset, term: key.clone(), doc_freq: self.in_mem_dictionary.get(key).unwrap().0 });
                let posting = self.in_mem_dictionary.get(key).unwrap();
                let posting_lists = *posting.1.clone();
                posting_lists.iter()
                .flat_map(|posting_list| posting_list.iter())
                .for_each(|posting| {
                    let disk_posting = self.disk_posting.as_mut().unwrap();
                    disk_posting[file_offset..file_offset + 4].copy_from_slice(&posting.doc_id.to_le_bytes());
                    disk_posting[file_offset + 4..file_offset + 8].copy_from_slice(&posting.freq.to_le_bytes());
                    file_offset += 8;
                });

            }
            self.in_mem_dictionary.clear();
            self.in_memory_bytes = 0;
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
