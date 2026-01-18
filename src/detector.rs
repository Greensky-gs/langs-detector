use crate::brain::Brain;
use crate::database::{Database,Lang,Ratios};
use std::collections::HashMap;

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

    pub fn detect(&mut self, input: String) -> String {
        let ratio: Ratios = self.brain.calculate_ratios(&input); 

        let mut table: Vec<(String, f64)> = self.database.vectors.iter().map(|vector| {
            return (vector.name.clone(), self.brain.distance(&vector.ratios, &ratio));
        }).collect();

        table.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut count: HashMap<String, u8> = HashMap::new();

        let mut i: usize = 0;
        let mut max: String = table[0].0.clone();
        while i < 10 && (i as isize) < table.len() as isize {
            count.entry(table[i].0.clone()).and_modify(|v| *v += 1).or_insert(1);

            if count.get(&table[i].0.clone()) > count.get(&max) {
                max = table[i].0.clone();
            }
            i+=1;
        }

        return max;
    }

    pub fn add_entry(&mut self, name: &String, content: &String) {
        let ratio: Ratios = self.brain.calculate_ratios(content);

        let lang = Lang::from(name.to_string(), ratio, content.to_string());

        self.database.add_entry(&lang);
    }
}
