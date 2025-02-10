use std::sync::mpsc::Receiver;

pub struct Indexer {}

impl Indexer {
    pub fn new() -> Self {
        Indexer {}
    }

    pub fn run(&mut self, rx: Receiver<Vec<String>>) {
        loop {
            let content = rx.try_recv();
            if content.is_ok() {
            //    println!("{:?}", content.unwrap());
            }
        }
    }
}
