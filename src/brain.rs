use crate::database::Ratios;
use rand::{Rng,rng};
use regex::Regex;
use std::collections::HashMap;

pub struct Brain {
    ratios: Ratios,
}

impl Brain {
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
                ("à", 5.0)
            ].iter().map(|(a, b)| {(a.to_string(), *b)}).collect()
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
