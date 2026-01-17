use crate::tables::Lang;
use std::collections::HashMap;
use rand::{rng, Rng};
use regex::Regex;

pub type Importance = HashMap<String, f32>;

pub struct Data {
    pub importance: Importance,
    pub values: Vec<(String, f32)>
}

impl Data {
    pub fn new() -> Data {
        let mut data = Data {
            importance: HashMap::new(),
            values: vec![]
        };

        data.build();
        return data;
    }
    fn add_default(&mut self) {
        let values: Vec<(&str, f32)> = vec![
            ("ç", 800.0),
            ("é", 3.0),
            ("e", 2.0),
            ("u", 1.5),
            ("h", 1.5),
            ("z", 1.5),
            ("w", 2.0),
            ("x", 2.0),
            ("averageWordSize", 0.2),
            ("'", 0.8),
            ("na", 0.8),
            ("no", 0.8),
            ("l", 0.7),
            ("m", 0.7),
            ("o", 0.7),
            ("p", 1.2),
            ("q", 1.2),
            ("r", 1.1),
            ("s", 1.1),
            ("t", 1.1),
            ("u", 1.2),
            ("v", 1.2),
            ("y", 2.0),
            ("z", 2.0)
        ];

        for (s,f) in values {
            self.importance.insert(s.to_string(), f);
        }
    }
    pub fn build(&mut self) {
        self.add_default();
    }

    pub fn random_importance(&mut self) {
        let mut RNG = rng();

        for (k, _v) in self.importance.clone() {
            self.importance.entry(k).and_modify(|l| *l = RNG.random_range(0.0..100.0));
        }
    }

    pub fn mutate(&mut self) -> Importance {
        let mut RNG = rng();

        let mut newImportance: Importance = HashMap::new();

        for (key, v) in self.importance.clone() {
            let min = v * 0.7;
            let max = v * 1.3;

            newImportance.insert(key, RNG.random_range(min..max));
        }

        return newImportance;
    }

    pub fn calc(&mut self, input: &String) -> Importance {
        let mut result: Importance = HashMap::new();

        for (k, v) in &self.importance {
            if k == "averageWordSize" {
                continue;
            }

            let re = Regex::new(&format!("({})", k)).unwrap();
            result.insert(k.to_string(), ((re.find_iter(input).count() as f64) / input.len() as f64) as f32);
        }

        let re = Regex::new(" +").unwrap();
        let words = re.split(input);
        let mut cumulated: usize = 0;

        for word in words {
            cumulated += word.len();
        }

        result.insert("averageWordSize".to_string(), (cumulated as f64 / re.split(input).count() as f64) as f32);

        return result;
    }
}
