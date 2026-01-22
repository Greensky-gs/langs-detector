use crate::database::Ratios;
use rand::{Rng,rng};
use regex::Regex;
use serde_json::from_str;
use std::collections::HashMap;
use std::fs;

pub struct Brain {
    pub ratios: Ratios,
}

impl Brain {
    pub fn random(min: f64, max: f64) -> Brain {
        let mut rng_thread = rng();
        if max <= min {
            panic!("Invalid range");
        }
        let mut brain = Brain::new();
        let mut copy: HashMap<String, f32>  = HashMap::new();

        for (k, _v) in &brain.ratios {
            copy.insert(k.to_string(), rng_thread.random_range(min..max) as f32);
        }
        for (k, v) in &copy {
            brain.ratios.entry(k.to_string()).and_modify(|val| *val = *v);
        }

        return brain;
    }
    pub fn new() -> Brain {
        return Brain {
            ratios: vec![
                ("a", 1.0),
                ("b", 1.0),
                ("c", 1.0),
                ("d", 2.0),
                ("f", 1.0),
                ("g", 1.0),
                ("h", 2.5),
                ("i", 1.0),
                ("k", 1.2),
                ("l", 0.5),
                ("m", 0.5),
                ("o", 0.5),
                ("p", 0.5),
                ("q", 0.5),
                ("r", 0.5),
                ("s", 0.5),
                ("t", 0.5),
                ("u", 0.5),
                ("v", 0.5),
                ("w", 2.1),
                ("x", 1.0),
                ("y", 1.8),
                ("z", 1.0),
                ("ç", 800.0),
                ("é", 5.0),
                ("è", 5.0),
                ("ù", 5.0),
                ("à", 5.0),
                ("ê", 8.0)
            ].iter().map(|(a, b)| {(a.to_string(), *b)}).collect()
        }
    }

    pub fn transform(&mut self, into: &Brain) {
        for (k, v) in &(*into).ratios {
            self.ratios.entry(k.to_string()).and_modify(|val| *val = *v).or_insert(*v);
        }
    }
    pub fn write(&self, output: &String) {
        let json = serde_json::to_string(&self.ratios).unwrap();
        fs::write(output, json).expect(&format!("Cannot write at {}", output));
    }
    pub fn load(&mut self, input: &String) {
        let data = fs::read_to_string(input).expect(&format!("Cannot read at {}", input));
        let map: HashMap<String, serde_json::Value> = from_str(&data).unwrap();

        for (k, v) in &map {
            if let serde_json::Value::Number(num) = v {
                let ratio = num.as_f64().unwrap() as f32;
                self.ratios.entry(k.to_string()).and_modify(|val| *val = ratio) .or_insert(ratio);
            }
        }
    } 
    pub fn distance(&self, a: &Ratios, b: &Ratios) -> f64 {
        self.ratios.iter().map(|(k, v)| {
            let val: f64 = (a[k] * v - b[k] * v) as f64;
            val * val
        }).sum::<f64>().sqrt()
    }

    pub fn calculate_ratios(&self, input: &String) -> Ratios {
        let mut ratios: Ratios = HashMap::new();

        for (k, _v) in &self.ratios {
            let re = Regex::new(&format!("({})", k)).unwrap();
            ratios.insert(k.to_string(), ((re.find_iter(input).count() as f64)/ input.len() as f64) as f32);
        }

        return ratios;
    }

    pub fn clone(&self) -> Brain {
        return Brain::from(&self.ratios);
    }
    pub fn mutate(&self) -> Brain {
        let mutation = mutate_ratio(&self.ratios);

        return Brain::from(&mutation);
    }
    pub fn from(ratio: &Ratios) -> Brain {
        return Brain {
            ratios: ratio.clone()
        }
    }
}

pub fn mutate_ratio(ratio: &Ratios) -> Ratios {
    let mut rng_thread = rng();
    let mut muted: Ratios = HashMap::new();

    for (k, v) in ratio {
        let max = v * 1.7;
        let min = v * 0.3;

        muted.insert(k.to_string(), rng_thread.random_range(min..max));
    }

    return muted;
}
