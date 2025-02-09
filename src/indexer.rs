use std::sync::mpsc::Receiver;

pub struct Indexer {}

impl Indexer {
    pub fn new() -> Self {
        Indexer {}
    }

    pub fn run(&mut self, rx: Receiver<Vec<String>>) {
        loop {
            let content = rx.recv().unwrap();
            //println!("{}", content);
        }
    }
}
