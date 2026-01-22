use crate::brain::Brain;
use crate::database::{Database,Lang,Ratios};
use crate::scrap::get_text_vectors;
use std::collections::HashMap;
use std::fs;

pub const K: u32 = 39;

pub struct Detector {
    database: Database,
    brain: Brain
}

impl Detector {
    pub fn new() -> Detector {
        let mut detector = Detector {
            database: Database::new(),
            brain: Brain::new()
        };

        detector.database.connect();

        detector.database.load();

        return detector;
    }

    pub fn get_least_registered_lang(&self) -> Option<String> {
        if self.database.count.len() == 0 {
            return None;
        }
        let mut min: String = self.database.count.keys().nth(0).unwrap().to_string();
        for (k, v) in &self.database.count {
            if self.database.count.get(&min).unwrap() > v {
                min = k.clone();
            }
        }

        return Some(min);
    }
    pub fn detect(&mut self, input: String) -> Option<String> {
        if self.database.vectors.len() == 0 {
            return None;
        }

        let ratio: Ratios = self.brain.calculate_ratios(&input); 

        let mut table: Vec<(String, f64)> = self.database.vectors.iter().map(|vector| {
            return (vector.name.clone(), self.brain.distance(&vector.ratios, &ratio));
        }).collect();

        table.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut count: HashMap<String, u8> = HashMap::new();

        let mut i: usize = 0;
        let mut max: String = table[0].0.clone();
        while i < K as usize && (i as isize) < table.len() as isize {
            count.entry(table[i].0.clone()).and_modify(|v| *v += 1).or_insert(1);

            if count.get(&table[i].0.clone()) > count.get(&max) {
                max = table[i].0.clone();
            }
            i+=1;
        }

        return Some(max);
    }

    pub fn add_entry(&mut self, name: &String, content: &String) {
        let ratio: Ratios = self.brain.calculate_ratios(content);

        let lang = Lang::from(name.to_string(), ratio, content.to_string());

        self.database.add_entry(&lang);
    }

    pub fn save_brain(&self, output: &String) {
        self.brain.write(output);
    }
    pub fn load_brain(&mut self, input: &String) -> bool {
        if !fs::metadata(input).is_ok() {
            println!("Trying to load from \x1b[90m{}\x1b[0m, but it doesn't exist", input);
            return false;
        }
        self.brain.load(input);
        return true;
    }

    pub fn switch_brain(&mut self, brain: &Brain) {
        self.brain.transform(brain);
    }

    pub fn test(&mut self, repeats: &u64) -> Option<HashMap<String, f64>> {
        let vectors = get_text_vectors(repeats);
        if let None = vectors {
            println!("Cannot test because None was received");
            return None;
        }
        let vectors = vectors.unwrap();

        let mut map: HashMap<String, u64> = HashMap::new();
        for vector in vectors {
            let result = self.detect(vector.1).unwrap();
            let modif = if result == vector.0 {1} else {0};

            map.entry(vector.0).and_modify(|v| *v += modif).or_insert(modif);
        }

        let mut result: HashMap<String, f64> = HashMap::new();
        for (k, v) in &map {
            result.insert(k.to_string(), (*v) as f64 / (*repeats) as f64);
        }
        return Some(result);
    }
}
