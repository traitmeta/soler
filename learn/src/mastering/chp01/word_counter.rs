use std::io::prelude::BufRead;
use std::{collections::BTreeMap, env, fs::File, io::BufReader};

#[derive(Debug)]
pub struct WordCounter(BTreeMap<String, u64>);

impl Default for WordCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl WordCounter {
    pub fn new() -> WordCounter {
        WordCounter(BTreeMap::new())
    }

    pub fn increment(&mut self, word: &str) {
        let key = word.to_string();
        let count = self.0.entry(key).or_insert(0);
        *count += 1;
    }

    pub fn display(&self) {
        for (key, value) in self.0.iter() {
            if *value > 1 {
                println!("{}: {}", key, value);
            }
        }
    }
}


fn count_word() {
    let arguments: Vec<String>  = env::args().collect();
    let filename = &arguments[1];
    println!("Processing file: {}", filename);

    let file = File::open(filename).expect("Could not open file");
    let reader = BufReader::new(file);

    let mut word_counter = WordCounter::new();
    for line in reader.lines(){
        let line = line.expect("Could not read line");
        let words = line.split(' ');
        for word in words{
            if word.is_empty(){
                continue;
            }else{
                word_counter.increment(word);
            }

        }
    }
    word_counter.display();
}
