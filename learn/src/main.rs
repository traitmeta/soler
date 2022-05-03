use std::env;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;
use learn::word_counter::WordCounter;

fn main() {
    let arguments: Vec<String>  = env::args().collect();
    let filename = &arguments[1];
    println!("Processing file: {}", filename);

    let file = File::open(filename).expect("Could not open file");
    let reader = BufReader::new(file);

    let mut word_counter = WordCounter::new();
    for line in reader.lines(){
        let line = line.expect("Could not read line");
        let words = line.split(" ");
        for word in words{
            if word == ""{
                continue;
            }else{
                word_counter.increment(word);
            }

        }
    }
    word_counter.display();
}
