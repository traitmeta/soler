use std::collections::BTreeMap;

#[derive(Debug)]
pub struct WordCounter(BTreeMap<String, u64>);

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
            if *value > 1{
                println!("{}: {}", key, value);
            }
        }
    }
    
}
